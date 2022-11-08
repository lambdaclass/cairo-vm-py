use crate::{
    relocatable::{PyMaybeRelocatable, PyRelocatable},
    utils::to_py_error,
    vm_core::PyVM,
};
use cairo_rs::{
    cairo_run::write_output,
    hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor,
    serde::deserialize_program::Member,
    types::{program::Program, relocatable::Relocatable},
    vm::{
        errors::{
            cairo_run_errors::CairoRunError, runner_errors::RunnerError, trace_errors::TraceError,
        },
        runners::cairo_runner::{CairoRunner, ExecutionResources},
    },
};
use num_bigint::{BigInt, Sign};
use pyo3::{exceptions::PyTypeError, prelude::*};
use std::iter::zip;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};

const MEMORY_GET_SEGMENT_USED_SIZE_MSG: &str = "Failed to segment used size";
const FAILED_TO_GET_INITIAL_FP: &str = "Failed to get initial segment";

#[pyclass(unsendable)]
#[pyo3(name = "CairoRunner")]
pub struct PyCairoRunner {
    inner: CairoRunner,
    pyvm: PyVM,
    hint_processor: BuiltinHintProcessor,
    hint_locals: HashMap<String, PyObject>,
    struct_types: Rc<HashMap<String, HashMap<String, Member>>>,
}

#[pymethods]
impl PyCairoRunner {
    #[new]
    pub fn new(
        path: String,
        entrypoint: String,
        layout: Option<String>,
        proof_mode: bool,
    ) -> PyResult<Self> {
        let program = Program::from_file(Path::new(&path), &entrypoint).map_err(to_py_error)?;
        let cairo_runner = CairoRunner::new(
            &program,
            &layout.unwrap_or_else(|| "plain".to_string()),
            proof_mode,
        )
        .map_err(to_py_error)?;

        let struct_types = program
            .identifiers
            .iter()
            .filter_map(|(path, identifier)| match identifier.type_.as_deref() {
                Some("struct") => Some((path.to_string(), identifier.members.clone().unwrap())),
                _ => None,
            })
            .collect();

        Ok(PyCairoRunner {
            inner: cairo_runner,
            pyvm: PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                true,
            ),
            hint_processor: BuiltinHintProcessor::new_empty(),
            hint_locals: HashMap::new(),
            struct_types: Rc::new(struct_types),
        })
    }

    #[pyo3(name = "cairo_run")]
    pub fn cairo_run_py(
        &mut self,
        print_output: bool,
        trace_file: Option<&str>,
        memory_file: Option<&str>,
        hint_locals: Option<HashMap<String, PyObject>>,
    ) -> PyResult<()> {
        let end = self.initialize()?;
        if let Some(locals) = hint_locals {
            self.hint_locals = locals
        }
        if trace_file.is_none() {
            self.pyvm.vm.borrow_mut().disable_trace();
        }
        self.run_until_pc(&end)?;

        self.pyvm
            .vm
            .borrow_mut()
            .verify_auto_deductions()
            .map_err(to_py_error)?;

        self.relocate()?;

        if print_output {
            self.write_output()?;
        }

        if let Some(trace_path) = trace_file {
            let trace_path = PathBuf::from(trace_path);
            let relocated_trace = self
                .inner
                .relocated_trace
                .as_ref()
                .ok_or(CairoRunError::Trace(TraceError::TraceNotEnabled))
                .map_err(to_py_error)?;

            match cairo_rs::cairo_run::write_binary_trace(relocated_trace, &trace_path) {
                Ok(()) => (),
                Err(_e) => {
                    return Err(CairoRunError::Runner(RunnerError::WriteFail)).map_err(to_py_error)
                }
            }
        }

        if let Some(memory_path) = memory_file {
            let memory_path = PathBuf::from(memory_path);
            cairo_rs::cairo_run::write_binary_memory(&self.inner.relocated_memory, &memory_path)
                .map_err(|_| to_py_error(CairoRunError::Runner(RunnerError::WriteFail)))?;
        }

        Ok(())
    }

    pub fn initialize(&mut self) -> PyResult<PyRelocatable> {
        self.inner
            .initialize(&mut self.pyvm.vm.borrow_mut())
            .map(PyRelocatable::from)
            .map_err(to_py_error)
    }

    pub fn run_until_pc(&mut self, address: &PyRelocatable) -> PyResult<()> {
        let references = self.inner.get_reference_list();
        let hint_data_dictionary = self
            .inner
            .get_hint_data_dictionary(&references, &self.hint_processor)
            .map_err(to_py_error)?;

        let address = Into::<Relocatable>::into(address);
        let constants = self.inner.get_constants().clone();
        while self.pyvm.vm.borrow().get_pc() != &address {
            self.pyvm
                .step(
                    &self.hint_processor,
                    &mut self.hint_locals,
                    &mut self.inner.exec_scopes,
                    &hint_data_dictionary,
                    Rc::clone(&self.struct_types),
                    &constants,
                )
                .map_err(to_py_error)?;
        }
        Ok(())
    }

    pub fn mark_as_accessed(&mut self, address: PyRelocatable, size: usize) -> PyResult<()> {
        self.inner
            .mark_as_accessed((&address).into(), size)
            .map_err(to_py_error)
    }

    pub fn relocate(&mut self) -> PyResult<()> {
        self.inner
            .relocate(&mut self.pyvm.vm.borrow_mut())
            .map_err(to_py_error)
    }

    pub fn get_output(&mut self) -> PyResult<String> {
        self.inner
            .get_output(&mut self.pyvm.vm.borrow_mut())
            .map_err(to_py_error)
    }

    pub fn write_output(&mut self) -> PyResult<()> {
        write_output(&mut self.inner, &mut self.pyvm.vm.borrow_mut()).map_err(to_py_error)
    }

    pub fn add_segment(&self) -> PyRelocatable {
        self.pyvm.vm.borrow_mut().add_memory_segment().into()
    }

    pub fn get_builtins_initial_stack(&self, py: Python) -> PyObject {
        self.pyvm
            .vm
            .borrow_mut()
            .get_builtin_runners()
            .iter()
            .map(|(builtin_name, builtin_runner)| {
                (
                    builtin_name,
                    builtin_runner
                        .initial_stack()
                        .into_iter()
                        .map(Into::<PyMaybeRelocatable>::into)
                        .collect::<Vec<PyMaybeRelocatable>>(),
                )
            })
            .collect::<Vec<(&String, Vec<PyMaybeRelocatable>)>>()
            .to_object(py)
    }

    pub fn get_builtins_final_stack(&self, stack_ptr: PyRelocatable) -> PyRelocatable {
        let mut stack_ptr = Relocatable::from(&stack_ptr);
        let mut stop_ptrs = Vec::new();
        let mut stop_ptr;

        for (_, runner) in self.pyvm.vm.borrow().get_builtin_runners() {
            (stack_ptr, stop_ptr) = runner
                .final_stack(&self.pyvm.vm.borrow(), stack_ptr)
                .unwrap();
            stop_ptrs.push(stop_ptr);
        }

        for ((_, runner), stop_ptr) in zip(
            self.pyvm.vm.borrow_mut().get_builtin_runners_as_mut(),
            stop_ptrs,
        ) {
            runner.set_stop_ptr(stop_ptr);
        }

        stack_ptr.into()
    }

    pub fn get_execution_resources(&self) -> PyResult<PyExecutionResources> {
        self.inner
            .get_execution_resources(&self.pyvm.vm.borrow())
            .map(PyExecutionResources)
            .map_err(to_py_error)
    }

    pub fn get_ap(&self) -> PyResult<PyRelocatable> {
        Ok(PyRelocatable::from(self.pyvm.vm.borrow().get_ap()))
    }

    pub fn get_initial_fp(&self) -> PyResult<PyRelocatable> {
        Ok(PyRelocatable::from(
            self.inner
                .get_initial_fp()
                .ok_or_else(|| PyTypeError::new_err(FAILED_TO_GET_INITIAL_FP))?,
        ))
    }

    pub fn get_return_values(&self, n_ret: usize, py: Python) -> PyResult<PyObject> {
        let return_values = self
            .pyvm
            .get_vm()
            .borrow()
            .get_return_values(n_ret)
            .map_err(|err| pyo3::exceptions::PyException::new_err(format!("{err}")))?
            .into_iter()
            .map(|maybe_reloc| maybe_reloc.into())
            .collect::<Vec<PyMaybeRelocatable>>()
            .to_object(py);
        Ok(return_values)
    }

    pub fn get_segment_used_size(&self, index: usize, py: Python) -> PyResult<PyObject> {
        Ok(self
            .pyvm
            .vm
            .borrow()
            .get_segment_used_size(index)
            .ok_or_else(|| PyTypeError::new_err(MEMORY_GET_SEGMENT_USED_SIZE_MSG))?
            .to_object(py))
    }
}

#[pyclass]
pub struct PyExecutionResources(ExecutionResources);

#[pymethods]
impl PyExecutionResources {
    #[getter]
    fn n_steps(&self) -> usize {
        self.0.n_steps
    }

    #[getter]
    fn n_memory_holes(&self) -> usize {
        self.0.n_memory_holes
    }

    #[getter]
    fn a(&self) -> Vec<(String, usize)> {
        self.0.builtin_instance_counter.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::relocatable::PyMaybeRelocatable::RelocatableValue;

    #[test]
    fn create_cairo_runner() {
        PyCairoRunner::new(
            "cairo_programs/fibonacci.json".to_string(),
            "main".to_string(),
            None,
            false,
        )
        .unwrap();
    }

    #[test]
    fn initialize_runner() {
        let mut runner = PyCairoRunner::new(
            "cairo_programs/fibonacci.json".to_string(),
            "main".to_string(),
            None,
            false,
        )
        .unwrap();
        runner.initialize().unwrap();
    }

    #[test]
    fn runner_relocate() {
        let mut runner = PyCairoRunner::new(
            "cairo_programs/fibonacci.json".to_string(),
            "main".to_string(),
            None,
            false,
        )
        .unwrap();
        runner.relocate().unwrap();
    }

    #[test]
    fn get_output() {
        let mut runner = PyCairoRunner::new(
            "cairo_programs/fibonacci.json".to_string(),
            "main".to_string(),
            Some("small".to_string()),
            false,
        )
        .unwrap();
        runner.get_output().unwrap();
    }

    #[test]
    fn write_output() {
        let mut runner = PyCairoRunner::new(
            "cairo_programs/fibonacci.json".to_string(),
            "main".to_string(),
            Some("small".to_string()),
            false,
        )
        .unwrap();
        runner.write_output().unwrap();
    }

    #[test]
    fn get_ap() {
        let runner = PyCairoRunner::new(
            "cairo_programs/fibonacci.json".to_string(),
            "main".to_string(),
            Some("small".to_string()),
            false,
        )
        .unwrap();
        assert_eq!(runner.get_ap().unwrap(), PyRelocatable::from((1, 0)));
    }

    #[test]
    fn add_segment() {
        let mut runner = PyCairoRunner::new(
            "cairo_programs/get_builtins_initial_stack.json".to_string(),
            "main".to_string(),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        runner.cairo_run_py(false, None, None, None).unwrap();
        let new_segment = runner.add_segment();
        assert_eq!(
            new_segment,
            PyRelocatable {
                segment_index: 5,
                offset: 0
            }
        );
        let new_segment = runner.add_segment();
        assert_eq!(
            new_segment,
            PyRelocatable {
                segment_index: 6,
                offset: 0
            }
        );
    }

    #[test]
    fn get_builtins_initial_stack() {
        let mut runner = PyCairoRunner::new(
            "cairo_programs/get_builtins_initial_stack.json".to_string(),
            "main".to_string(),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        runner.cairo_run_py(false, None, None, None).unwrap();

        let expected_output: Vec<(&str, Vec<PyMaybeRelocatable>)> = vec![(
            "range_check",
            vec![RelocatableValue(PyRelocatable {
                segment_index: 2,
                offset: 0,
            })],
        )];

        Python::with_gil(|py| {
            assert_eq!(
                runner
                    .get_builtins_initial_stack(py)
                    .extract::<Vec<(&str, Vec<PyMaybeRelocatable>)>>(py)
                    .unwrap(),
                expected_output
            );
        });
    }

    #[test]
    fn get_builtins_final_stack() {
        let mut runner = PyCairoRunner::new(
            "cairo_programs/get_builtins_initial_stack.json".to_string(),
            "main".to_string(),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        runner.cairo_run_py(false, None, None, None).unwrap();

        let expected_output = PyRelocatable::from((1, 8));

        Python::with_gil(|_py| {
            let final_stack = PyRelocatable::from((1, 9));
            assert_eq!(
                runner.get_builtins_final_stack(final_stack),
                expected_output
            );
        });
    }

    #[test]
    fn get_segment_used_size_of_segment_0() {
        let mut runner = PyCairoRunner::new(
            "cairo_programs/fibonacci.json".to_string(),
            "main".to_string(),
            None,
            false,
        )
        .unwrap();
        runner.cairo_run_py(false, None, None, None).unwrap();
        Python::with_gil(|py| {
            assert_eq!(
                24,
                runner
                    .get_segment_used_size(0, py)
                    .unwrap()
                    .extract::<usize>(py)
                    .unwrap()
            )
        });
    }

    #[test]
    fn get_segment_used_size_of_segment_2() {
        let mut runner = PyCairoRunner::new(
            "cairo_programs/fibonacci.json".to_string(),
            "main".to_string(),
            None,
            false,
        )
        .unwrap();
        runner.cairo_run_py(false, None, None, None).unwrap();
        Python::with_gil(|py| {
            assert_eq!(
                0,
                runner
                    .get_segment_used_size(2, py)
                    .unwrap()
                    .extract::<usize>(py)
                    .unwrap()
            )
        });
    }
    #[test]
    fn get_initial_fp_test() {
        let mut runner = PyCairoRunner::new(
            "cairo_programs/fibonacci.json".to_string(),
            "main".to_string(),
            Some(String::from("all")),
            false,
        )
        .unwrap();
        runner.cairo_run_py(false, None, None, None).unwrap();
        assert_eq! {
            PyRelocatable::from((1,2)),
            runner.get_initial_fp().unwrap()
        };
    }
}
