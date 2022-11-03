use crate::{relocatable::{PyMaybeRelocatable, PyRelocatable}, utils::to_py_error, vm_core::PyVM};
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
use pyo3::prelude::*;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};

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
    pub fn new(path: String, entrypoint: String, layout: Option<String>) -> PyResult<Self> {
        let program = Program::new(Path::new(&path), &entrypoint).map_err(to_py_error)?;
        let cairo_runner =
            CairoRunner::new(&program, layout.unwrap_or_else(|| "plain".to_string()))
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

    pub fn get_execution_resources(&self) -> PyResult<PyExecutionResources> {
        self.inner
            .get_execution_resources(&self.pyvm.vm.borrow())
            .map(PyExecutionResources)
            .map_err(to_py_error)
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

    #[test]
    fn create_cairo_runner() {
        PyCairoRunner::new(
            "cairo_programs/fibonacci.json".to_string(),
            "main".to_string(),
            None,
        )
        .unwrap();
    }

    #[test]
    fn initialize_runner() {
        let mut runner = PyCairoRunner::new(
            "cairo_programs/fibonacci.json".to_string(),
            "main".to_string(),
            None,
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
        )
        .unwrap();
        runner.write_output().unwrap();
    }
}
