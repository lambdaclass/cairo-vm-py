use crate::{
    relocatable::{PyMaybeRelocatable, PyRelocatable},
    utils::to_py_error,
    vm_core::PyVM,
};
use cairo_rs::{
    cairo_run::write_output,
    hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor,
    serde::deserialize_program::Member,
    types::{
        program::Program,
        relocatable::{MaybeRelocatable, Relocatable},
    },
    vm::{
        errors::{
            cairo_run_errors::CairoRunError, runner_errors::RunnerError, trace_errors::TraceError,
        },
        runners::cairo_runner::{CairoRunner, ExecutionResources},
    },
};
use num_bigint::{BigInt, Sign};
use pyo3::{
    exceptions::{PyNotImplementedError, PyTypeError},
    prelude::*,
    types::PyIterator,
};
use std::iter::zip;
use std::{any::Any, collections::HashMap, path::PathBuf, rc::Rc};

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
        program: String,
        entrypoint: String,
        layout: Option<String>,
        proof_mode: bool,
    ) -> PyResult<Self> {
        let program = Program::from_reader(program.as_bytes(), &entrypoint).map_err(to_py_error)?;
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

    pub fn initialize_segments(&mut self) {
        self.inner
            .initialize_segments(&mut self.pyvm.vm.borrow_mut(), None)
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

    pub fn get_builtins_final_stack(&self, stack_ptr: PyRelocatable) -> PyResult<PyRelocatable> {
        let mut stack_ptr = Relocatable::from(&stack_ptr);
        let mut stop_ptrs = Vec::new();
        let mut stop_ptr;

        for (_, runner) in self.pyvm.vm.borrow().get_builtin_runners() {
            (stack_ptr, stop_ptr) = runner
                .final_stack(&self.pyvm.vm.borrow(), stack_ptr)
                .map_err(to_py_error)?;
            stop_ptrs.push(stop_ptr);
        }

        for ((_, runner), stop_ptr) in zip(
            self.pyvm.vm.borrow_mut().get_builtin_runners_as_mut(),
            stop_ptrs,
        ) {
            runner.set_stop_ptr(stop_ptr);
        }

        Ok(stack_ptr.into())
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

    #[allow(clippy::too_many_arguments)]
    pub fn run_from_entrypoint(
        &mut self,
        entrypoint: &PyAny,
        args: Vec<&PyAny>,
        typed_args: Option<bool>,
        verify_secure: Option<bool>,
        apply_modulo_to_args: Option<bool>,
    ) -> PyResult<()> {
        enum Either {
            MaybeRelocatable(MaybeRelocatable),
            VecMaybeRelocatable(Vec<MaybeRelocatable>),
        }

        impl Either {
            pub fn as_any(&self) -> &dyn Any {
                match self {
                    Self::MaybeRelocatable(x) => x as &dyn Any,
                    Self::VecMaybeRelocatable(x) => x as &dyn Any,
                }
            }
        }

        let entrypoint = if let Ok(x) = entrypoint.extract::<usize>() {
            x
        } else if entrypoint.extract::<String>().is_ok() {
            return Err(PyNotImplementedError::new_err(()));
        } else {
            return Err(PyTypeError::new_err("entrypoint must be int or str"));
        };

        let mut processed_args = Vec::new();
        for arg in args {
            let arg_box = if let Ok(x) = arg.extract::<PyMaybeRelocatable>() {
                Either::MaybeRelocatable(x.into())
            } else if let Ok(x) = arg.extract::<Vec<PyMaybeRelocatable>>() {
                Either::VecMaybeRelocatable(x.into_iter().map(|x| x.into()).collect())
            } else {
                return Err(PyTypeError::new_err("Argument has unsupported type."));
            };

            processed_args.push(arg_box);
        }

        let vm = self.pyvm.get_vm();

        let mut vm = vm.borrow_mut();

        self.inner
            .run_from_entrypoint(
                entrypoint,
                processed_args.iter().map(|x| x.as_any()).collect(),
                typed_args.unwrap_or(false),
                verify_secure.unwrap_or(true),
                apply_modulo_to_args.unwrap_or(true),
                &mut vm,
                &self.hint_processor,
            )
            .map_err(to_py_error)
    }

    /// Inserts a value into a memory address given by a Relocatable value.
    pub fn insert(&self, key: &PyRelocatable, value: PyMaybeRelocatable) -> PyResult<()> {
        self.pyvm
            .get_vm()
            .borrow_mut()
            .insert_value(&key.into(), value)
            .map_err(to_py_error)
    }

    // Initialize all the builtins and segments.
    pub fn initialize_function_runner(&mut self) -> PyResult<()> {
        self.inner
            .initialize_function_runner(&mut self.pyvm.vm.borrow_mut())
            .map_err(to_py_error)
    }

    pub fn gen_arg(
        &self,
        py: Python,
        arg: Py<PyAny>,
        apply_modulo_to_args: bool,
    ) -> PyResult<PyObject> {
        Ok(
            PyMaybeRelocatable::from(match PyIterator::from_object(py, &arg) {
                Ok(iterator) => {
                    let segment_ptr = MaybeRelocatable::RelocatableValue(
                        self.pyvm.vm.borrow_mut().add_memory_segment(),
                    );
                    self.write_arg(
                        py,
                        segment_ptr.clone().into(),
                        iterator.to_object(py),
                        apply_modulo_to_args,
                    )?;
                    segment_ptr
                }
                _ => {
                    let mut value: MaybeRelocatable = arg.extract::<PyMaybeRelocatable>(py)?.into();
                    if apply_modulo_to_args {
                        value = value
                            .mod_floor(self.pyvm.vm.borrow().get_prime())
                            .map_err(to_py_error)?;
                    }
                    value
                }
            })
            .to_object(py),
        )
    }

    #[args(apply_modulo_to_args = true)]
    pub fn write_arg(
        &self,
        py: Python<'_>,
        ptr: PyMaybeRelocatable,
        arg: Py<PyAny>,
        apply_modulo_to_args: bool,
    ) -> PyResult<PyObject> {
        let ptr: MaybeRelocatable = ptr.into();

        let arg_iter = PyIterator::from_object(py, &arg)?;
        let mut data = Vec::new();
        for value in arg_iter {
            data.push(
                self.gen_arg(py, value?.to_object(py), apply_modulo_to_args)?
                    .extract::<PyMaybeRelocatable>(py)?
                    .into(),
            );
        }

        self.pyvm
            .vm
            .borrow_mut()
            .load_data(&ptr, data)
            .map(|x| PyMaybeRelocatable::from(x).to_object(py))
            .map_err(to_py_error)
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
    use cairo_rs::bigint;
    use std::fs;
    use std::ops::Add;

    use super::*;
    use crate::relocatable::PyMaybeRelocatable::RelocatableValue;

    #[test]
    fn create_cairo_runner() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        PyCairoRunner::new(program, "main".to_string(), None, false).unwrap();
    }

    #[test]
    fn initialize_runner() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(program, "main".to_string(), None, false).unwrap();
        runner.initialize().unwrap();
    }

    #[test]
    fn runner_relocate() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(program, "main".to_string(), None, false).unwrap();
        runner.relocate().unwrap();
    }

    #[test]
    fn get_output() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            "main".to_string(),
            Some("small".to_string()),
            false,
        )
        .unwrap();
        runner.get_output().unwrap();
    }

    #[test]
    fn write_output() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            "main".to_string(),
            Some("small".to_string()),
            false,
        )
        .unwrap();
        runner.write_output().unwrap();
    }

    #[test]
    fn get_ap() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let runner = PyCairoRunner::new(
            program,
            "main".to_string(),
            Some("small".to_string()),
            false,
        )
        .unwrap();
        assert_eq!(runner.get_ap().unwrap(), PyRelocatable::from((1, 0)));
    }

    #[test]
    fn add_segment() {
        let path = "cairo_programs/get_builtins_initial_stack.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
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
        let path = "cairo_programs/get_builtins_initial_stack.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
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
        let path = "cairo_programs/get_builtins_initial_stack.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            "main".to_string(),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        runner.cairo_run_py(false, None, None, None).unwrap();

        let expected_output = PyRelocatable::from((1, 8));

        let final_stack = PyRelocatable::from((1, 9));
        assert_eq!(
            runner.get_builtins_final_stack(final_stack).unwrap(),
            expected_output
        );
    }

    #[test]
    fn final_stack_when_not_using_builtins() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            "main".to_string(),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        runner.cairo_run_py(false, None, None, None).unwrap();

        let expected_output = PyRelocatable::from((1, 0));

        let final_stack = PyRelocatable::from((1, 0));
        assert_eq!(
            runner.get_builtins_final_stack(final_stack).unwrap(),
            expected_output
        );
    }

    #[test]
    fn final_stack_when_using_two_builtins() {
        let path = "cairo_programs/final_stack.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner =
            PyCairoRunner::new(program, "main".to_string(), Some("all".to_string()), false)
                .unwrap();

        runner.cairo_run_py(false, None, None, None).unwrap();

        // Insert os_context in the VM's stack:
        //  * range_check segment base in (1, 41)
        //  * bitwise segment base in (1, 41)
        runner
            .insert(
                &(1, 41).into(),
                PyMaybeRelocatable::RelocatableValue(PyRelocatable::new((2, 0))),
            )
            .unwrap();

        runner
            .insert(
                &(1, 42).into(),
                PyMaybeRelocatable::RelocatableValue(PyRelocatable::new((3, 0))),
            )
            .unwrap();

        let expected_output = PyRelocatable::from((1, 40));
        let final_stack = PyRelocatable::from((1, 42));

        assert_eq!(
            runner.get_builtins_final_stack(final_stack).unwrap(),
            expected_output
        );
    }

    #[test]
    fn get_segment_used_size_of_segment_0() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(program, "main".to_string(), None, false).unwrap();
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
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(program, "main".to_string(), None, false).unwrap();
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
    fn run_from_entrypoint_without_args() {
        let path = "cairo_programs/not_main.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            "main".to_string(),
            Some("plain".to_string()),
            false,
        )
        .unwrap();

        // Without `runner.initialize()`, an uninitialized error is returned.
        // With `runner.initialize()`, an invalid memory assignment is returned...
        //   Maybe it has to do with `initialize_main_entrypoint()` called from `initialize()`?
        runner.initialize_segments();

        Python::with_gil(|py| {
            runner
                .run_from_entrypoint(
                    py.eval("0", None, None).unwrap(),
                    vec![],
                    Some(false),
                    None,
                    None,
                )
                .unwrap();
        });
    }

    #[test]
    fn run_from_entrypoint_with_one_typed_arg() {
        // One arg (typed)
        //   value
    }

    #[test]
    fn run_from_entrypoint_with_one_typed_vec_arg() {
        // One arg (typed)
        //   vec
    }

    #[test]
    fn run_from_entrypoint_with_multiple_untyped_args() {
        // Multiple args (no typed)
        // Test that `PyCairoRunner::insert()` inserts values correctly.
    }

    #[test]
    fn insert() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let runner = PyCairoRunner::new(program, "main".to_string(), None, true).unwrap();

        (*runner.pyvm.get_vm()).borrow_mut().add_memory_segment();
        runner
            .insert(&(0, 0).into(), PyMaybeRelocatable::Int(bigint!(3)))
            .unwrap();
        runner
            .insert(&(0, 1).into(), PyMaybeRelocatable::Int(bigint!(4)))
            .unwrap();
        runner
            .insert(&(0, 2).into(), PyMaybeRelocatable::Int(bigint!(5)))
            .unwrap();
        assert_eq!(
            runner
                .pyvm
                .get_vm()
                .borrow()
                .get_continuous_range(&(0, 0).into(), 3),
            Ok(vec![
                bigint!(3).into(),
                bigint!(4).into(),
                bigint!(5).into(),
            ]),
        )
    }

    /// Test that `PyCairoRunner::insert()` fails when it should.
    #[test]
    fn insert_duplicate() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let runner = PyCairoRunner::new(program, "main".to_string(), None, true).unwrap();

        (*runner.pyvm.get_vm()).borrow_mut().add_memory_segment();
        runner
            .insert(&(0, 0).into(), PyMaybeRelocatable::Int(bigint!(3)))
            .unwrap();
        runner
            .insert(&(0, 1).into(), PyMaybeRelocatable::Int(bigint!(4)))
            .unwrap();
        runner
            .insert(&(0, 0).into(), PyMaybeRelocatable::Int(bigint!(5)))
            .expect_err("insertion succeeded when it should've failed");
        assert_eq!(
            runner
                .pyvm
                .get_vm()
                .borrow()
                .get_continuous_range(&(0, 0).into(), 2),
            Ok(vec![bigint!(3).into(), bigint!(4).into(),]),
        );
    }

    #[test]
    fn get_initial_fp_test() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
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

    #[test]
    fn initialize_function_runner() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner =
            PyCairoRunner::new(program, "main".to_string(), Some("all".to_string()), false)
                .unwrap();

        runner.initialize_function_runner().unwrap();

        let expected_output: Vec<(&str, Vec<PyMaybeRelocatable>)> = vec![
            (
                "output",
                vec![RelocatableValue(PyRelocatable {
                    segment_index: 2,
                    offset: 0,
                })],
            ),
            (
                "pedersen",
                vec![RelocatableValue(PyRelocatable {
                    segment_index: 3,
                    offset: 0,
                })],
            ),
            (
                "range_check",
                vec![RelocatableValue(PyRelocatable {
                    segment_index: 4,
                    offset: 0,
                })],
            ),
            (
                "bitwise",
                vec![RelocatableValue(PyRelocatable {
                    segment_index: 5,
                    offset: 0,
                })],
            ),
            (
                "ec_op",
                vec![RelocatableValue(PyRelocatable {
                    segment_index: 6,
                    offset: 0,
                })],
            ),
        ];

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
    fn write_arg_test() {
        Python::with_gil(|py| {
            let path = "cairo_programs/fibonacci.json".to_string();
            let program = fs::read_to_string(path).unwrap();
            let runner =
                PyCairoRunner::new(program, "main".to_string(), Some("all".to_string()), false)
                    .unwrap();

            let ptr = runner.add_segment();
            runner
                .write_arg(
                    py,
                    PyMaybeRelocatable::RelocatableValue(ptr),
                    py.eval("[1, 2, [3, 4], [5, 6]]", None, None)
                        .unwrap()
                        .to_object(py),
                    true,
                )
                .unwrap();

            let vm_ref = runner.pyvm.get_vm();
            let vm_ref = vm_ref.borrow();

            assert_eq!(
                vm_ref
                    .get_maybe(&Relocatable::from((0, 0)))
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(1),
            );
            assert_eq!(
                vm_ref
                    .get_maybe(&Relocatable::from((0, 1)))
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(2),
            );

            let relocatable = vm_ref
                .get_maybe(&Relocatable::from((0, 2)))
                .unwrap()
                .unwrap()
                .get_relocatable()
                .unwrap()
                .clone();

            assert_eq!(
                vm_ref
                    .get_maybe(&relocatable)
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(3),
            );
            assert_eq!(
                vm_ref
                    .get_maybe(&relocatable.clone().add(1_i32))
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(4),
            );
            assert!(vm_ref.get_maybe(&relocatable.add(2_i32)).unwrap().is_none());

            let relocatable = vm_ref
                .get_maybe(&Relocatable::from((0, 3)))
                .unwrap()
                .unwrap()
                .get_relocatable()
                .unwrap()
                .clone();

            assert_eq!(
                vm_ref
                    .get_maybe(&relocatable)
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(5),
            );
            assert_eq!(
                vm_ref
                    .get_maybe(&relocatable.clone().add(1_i32))
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(6),
            );
            assert!(vm_ref.get_maybe(&relocatable.add(2_i32)).unwrap().is_none());

            assert!(vm_ref
                .get_maybe(&Relocatable::from((0, 4)))
                .unwrap()
                .is_none());
        });
    }
}
