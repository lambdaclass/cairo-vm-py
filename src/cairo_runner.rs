use crate::{
    instruction_location::InstructionLocation,
    memory::PyMemory,
    memory_segments::PySegmentManager,
    relocatable::{PyMaybeRelocatable, PyRelocatable},
    utils::to_py_error,
    vm_core::PyVM,
};
use bincode::enc::write::Writer;
use cairo_vm::{
    hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor,
    serde::deserialize_program::Member,
    types::{
        program::Program,
        relocatable::{MaybeRelocatable, Relocatable},
    },
    vm::{
        errors::vm_exception::{get_error_attr_value, get_location, get_traceback},
        runners::cairo_runner::{CairoRunner, ExecutionResources},
        security::verify_secure_runner,
    },
};
use pyo3::{
    exceptions::{PyNotImplementedError, PyTypeError, PyValueError},
    prelude::*,
    types::PyIterator,
};
use std::io::{self, Write};
use std::{any::Any, borrow::BorrowMut, collections::HashMap, path::PathBuf, rc::Rc};

pyo3::import_exception!(starkware.cairo.lang.vm.utils, ResourcesError);

const MEMORY_GET_SEGMENT_USED_SIZE_MSG: &str = "Failed to segment used size";
const FAILED_TO_GET_INITIAL_FP: &str = "Failed to get initial segment";

struct FileWriter {
    buf_writer: io::BufWriter<std::fs::File>,
    bytes_written: usize,
}

impl Writer for FileWriter {
    fn write(&mut self, bytes: &[u8]) -> Result<(), bincode::error::EncodeError> {
        self.buf_writer
            .write_all(bytes)
            .map_err(|e| bincode::error::EncodeError::Io {
                inner: e,
                index: self.bytes_written,
            })?;

        self.bytes_written += bytes.len();

        Ok(())
    }
}

impl FileWriter {
    fn new(buf_writer: io::BufWriter<std::fs::File>) -> Self {
        Self {
            buf_writer,
            bytes_written: 0,
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buf_writer.flush()
    }
}

#[pyclass(unsendable)]
#[pyo3(name = "CairoRunner")]
pub struct PyCairoRunner {
    inner: CairoRunner,
    pub(crate) pyvm: PyVM,
    hint_processor: BuiltinHintProcessor,
    hint_locals: HashMap<String, PyObject>,
    struct_types: Rc<HashMap<String, HashMap<String, Member>>>,
    static_locals: Option<HashMap<String, PyObject>>,
}

#[pymethods]
impl PyCairoRunner {
    #[new]
    #[pyo3(signature = (program, entrypoint="__main__.main".to_string(), layout="plain".to_string(), proof_mode=false))]
    pub fn new(
        program: String,
        entrypoint: Option<String>,
        layout: Option<String>,
        proof_mode: bool,
    ) -> PyResult<Self> {
        let program =
            Program::from_bytes(program.as_bytes(), entrypoint.as_deref()).map_err(to_py_error)?;
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
            pyvm: PyVM::new(true),
            hint_processor: BuiltinHintProcessor::new_empty(),
            hint_locals: HashMap::new(),
            struct_types: Rc::new(struct_types),
            static_locals: None,
        })
    }

    #[pyo3(name = "cairo_run")]
    pub fn cairo_run_py(
        &mut self,
        print_output: bool,
        trace_file: Option<&str>,
        memory_file: Option<&str>,
        hint_locals: Option<HashMap<String, PyObject>>,
        static_locals: Option<HashMap<String, PyObject>>,
        entrypoint: Option<&str>,
    ) -> PyResult<()> {
        if let Some(entrypoint) = entrypoint {
            self.inner
                .borrow_mut()
                .set_entrypoint(Some(entrypoint))
                .map_err(to_py_error)?;
        }

        let end = self.initialize()?;

        if let Some(locals) = hint_locals {
            self.hint_locals = locals
        }

        self.static_locals = static_locals;

        if trace_file.is_none() {
            (*self.pyvm.vm).borrow_mut().disable_trace();
        }
        if let Err(error) = self.run_until_pc(&end, None) {
            return Err(self.as_vm_exception(error));
        }

        self.inner
            .end_run(
                false,
                false,
                &mut (*self.pyvm.vm).borrow_mut(),
                &mut self.hint_processor,
            )
            .map_err(to_py_error)?;
        (*self.pyvm.vm)
            .borrow()
            .verify_auto_deductions()
            .map_err(to_py_error)?;
        self.inner
            .read_return_values(&mut (*self.pyvm.vm).borrow_mut())
            .map_err(to_py_error)?;
        verify_secure_runner(&self.inner, true, &mut (*self.pyvm.vm).borrow_mut())
            .map_err(to_py_error)?;

        self.relocate()?;

        if print_output {
            self.write_output()?;
        }

        if let Some(trace_path) = trace_file {
            let trace_path = PathBuf::from(trace_path);
            let vm_ref = self.pyvm.vm.borrow();
            let relocated_trace = vm_ref.get_relocated_trace().map_err(to_py_error)?;

            let trace_file = std::fs::File::create(trace_path)?;

            let mut trace_writer = FileWriter::new(io::BufWriter::new(trace_file));

            cairo_vm::cairo_run::write_encoded_trace(relocated_trace, &mut trace_writer)
                .map_err(to_py_error)?;
            trace_writer.flush()?;
        }

        if let Some(memory_path) = memory_file {
            let memory_file = std::fs::File::create(memory_path)?;
            let mut memory_writer = FileWriter::new(io::BufWriter::new(memory_file));

            cairo_vm::cairo_run::write_encoded_memory(
                &self.inner.relocated_memory,
                &mut memory_writer,
            )
            .map_err(to_py_error)?;
            memory_writer.flush()?;
        }

        Ok(())
    }

    pub fn initialize(&mut self) -> PyResult<PyRelocatable> {
        self.inner
            .initialize(&mut (*self.pyvm.vm).borrow_mut())
            .map(PyRelocatable::from)
            .map_err(to_py_error)
    }

    pub fn initialize_segments(&mut self) {
        self.inner
            .initialize_segments(&mut (*self.pyvm.vm).borrow_mut(), None)
    }

    pub fn run_until_pc(
        &mut self,
        address: &PyRelocatable,
        run_resources_n_steps: Option<usize>,
    ) -> PyResult<()> {
        let references = self.inner.get_reference_list();
        let hint_data_dictionary = self
            .inner
            .get_hint_data_dictionary(&references, &mut self.hint_processor)
            .map_err(to_py_error)?;

        let address = Into::<Relocatable>::into(address);
        let constants = self.inner.get_constants().clone();
        let mut steps_left = run_resources_n_steps.unwrap_or(1); // default value
        while self.pyvm.vm.borrow().get_pc() != address && steps_left > 0 {
            self.pyvm.step(
                &mut self.hint_processor,
                &mut self.hint_locals,
                &mut self.inner.exec_scopes,
                &hint_data_dictionary,
                Rc::clone(&self.struct_types),
                &constants,
                self.static_locals.as_ref(),
            )?;
            // Consume step
            if run_resources_n_steps.is_some() {
                steps_left -= 1;
            }
        }
        if self.pyvm.vm.borrow().get_pc() != address {
            return Err(ResourcesError::new_err(PyValueError::new_err(
                "Error: Execution reached the end of the program.",
            )));
        }
        Ok(())
    }

    pub fn mark_as_accessed(&mut self, address: PyRelocatable, size: usize) -> PyResult<()> {
        (*self.pyvm.vm)
            .borrow_mut()
            .mark_address_range_as_accessed((&address).into(), size)
            .map_err(to_py_error)
    }

    pub fn relocate(&mut self) -> PyResult<()> {
        self.inner
            .relocate(&mut (*self.pyvm.vm).borrow_mut())
            .map_err(to_py_error)
    }

    pub fn write_output(&mut self) -> PyResult<()> {
        let mut buffer = String::new();
        (*self.pyvm.vm)
            .borrow_mut()
            .write_output(&mut buffer)
            .map_err(to_py_error)?;
        println!("Program Output:\n{}", buffer);
        Ok(())
    }

    pub fn add_segment(&self) -> PyRelocatable {
        (*self.pyvm.vm).borrow_mut().add_memory_segment().into()
    }

    pub fn get_program_builtins_initial_stack(&self, py: Python) -> PyObject {
        (*self.pyvm.vm)
            .borrow_mut()
            .get_builtin_runners()
            .iter()
            .filter(|builtin_runner| {
                self.inner
                    .get_program_builtins()
                    .iter()
                    .any(|name| &name.name() == &builtin_runner.name())
            })
            .flat_map(|builtin_runner| {
                builtin_runner
                    .initial_stack()
                    .into_iter()
                    .map(Into::<PyMaybeRelocatable>::into)
                    .collect::<Vec<PyMaybeRelocatable>>()
            })
            .collect::<Vec<PyMaybeRelocatable>>()
            .to_object(py)
    }

    pub fn get_builtins_initial_stack(&self, py: Python) -> PyObject {
        (*self.pyvm.vm)
            .borrow_mut()
            .get_builtin_runners()
            .iter()
            .map(|builtin_runner| {
                builtin_runner
                    .initial_stack()
                    .into_iter()
                    .map(Into::<PyMaybeRelocatable>::into)
                    .collect::<Vec<PyMaybeRelocatable>>()
            })
            .collect::<Vec<Vec<PyMaybeRelocatable>>>()
            .to_object(py)
    }

    pub fn get_builtins_final_stack(&self, stack_ptr: PyRelocatable) -> PyResult<PyRelocatable> {
        Ok(self
            .inner
            .get_builtins_final_stack(
                &mut (*self.pyvm.vm).borrow_mut(),
                Relocatable::from(&stack_ptr),
            )
            .map_err(to_py_error)?
            .into())
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

    #[getter]
    pub fn initial_fp(&self) -> PyResult<PyRelocatable> {
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

    #[allow(unused)]
    #[allow(clippy::too_many_arguments)]
    pub fn run_from_entrypoint(
        &mut self,
        py: Python,
        entrypoint: &PyAny,
        args: Py<PyAny>,
        hint_locals: Option<HashMap<String, PyObject>>,
        static_locals: Option<HashMap<String, PyObject>>,
        typed_args: Option<bool>,
        verify_secure: Option<bool>,
        run_resources: Option<PyRunResources>,
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

        if let Some(locals) = hint_locals {
            self.hint_locals = locals
        }

        self.static_locals = static_locals;

        let entrypoint = if let Ok(x) = entrypoint.extract::<usize>() {
            x
        } else if entrypoint.extract::<String>().is_ok() {
            return Err(PyNotImplementedError::new_err(()));
        } else {
            return Err(PyTypeError::new_err("entrypoint must be int or str"));
        };

        let stack = if typed_args.unwrap_or(false) {
            let args = self
                .gen_typed_args(py, args.to_object(py))
                .map_err(to_py_error)?;

            let mut stack = Vec::new();
            for arg in args.extract::<Vec<&PyAny>>(py)? {
                let arg: MaybeRelocatable = arg.extract::<PyMaybeRelocatable>()?.into();
                stack.push(arg)
            }
            stack
        } else {
            let mut processed_args = Vec::new();
            for arg in args.extract::<Vec<&PyAny>>(py)? {
                let arg_box = if let Ok(x) = arg.extract::<PyMaybeRelocatable>() {
                    Either::MaybeRelocatable(x.into())
                } else if let Ok(x) = arg.extract::<Vec<PyMaybeRelocatable>>() {
                    Either::VecMaybeRelocatable(x.into_iter().map(|x| x.into()).collect())
                } else {
                    return Err(PyTypeError::new_err("Argument has unsupported type."));
                };

                processed_args.push(arg_box);
            }
            let processed_args: Vec<&dyn Any> = processed_args.iter().map(|x| x.as_any()).collect();
            let mut stack = Vec::new();
            for arg in processed_args {
                stack.push(
                    (*self.pyvm.vm)
                        .borrow_mut()
                        .gen_arg(arg)
                        .map_err(to_py_error)?,
                );
            }

            stack
        };

        let return_fp = MaybeRelocatable::from(0);

        let end = self
            .inner
            .initialize_function_entrypoint(
                &mut (*self.pyvm.vm).borrow_mut(),
                entrypoint,
                stack,
                return_fp,
            )
            .map_err(to_py_error)?;

        self.inner
            .initialize_vm(&mut (*self.pyvm.vm).borrow_mut())
            .map_err(to_py_error)?;

        if let Err(error) = self.run_until_pc(
            &end.into(),
            run_resources.and_then(|resource| resource.n_steps),
        ) {
            return Err(self.as_vm_exception(error));
        }

        self.inner
            .end_run(
                true,
                false,
                &mut (*self.pyvm.vm).borrow_mut(),
                &mut self.hint_processor,
            )
            .map_err(to_py_error)?;

        if verify_secure.unwrap_or(true) {
            verify_secure_runner(&self.inner, false, &mut (*self.pyvm.vm).borrow_mut())
                .map_err(to_py_error)?;
        }

        Ok(())
    }

    /// Inserts a value into a memory address given by a Relocatable value.
    pub fn insert(&self, key: &PyRelocatable, value: PyMaybeRelocatable) -> PyResult<()> {
        (*self.pyvm.vm)
            .borrow_mut()
            .insert_value(key.into(), value)
            .map_err(to_py_error)
    }

    // Initialize all the builtins and segments.
    pub fn initialize_function_runner(&mut self) -> PyResult<()> {
        self.inner
            .initialize_function_runner(&mut (*self.pyvm.vm).borrow_mut())
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
                        (*self.pyvm.vm).borrow_mut().add_memory_segment(),
                    );
                    self.write_arg(
                        py,
                        segment_ptr.clone().into(),
                        iterator.to_object(py),
                        apply_modulo_to_args,
                    )?;
                    segment_ptr
                }
                _ => arg.extract::<PyMaybeRelocatable>(py)?.into(),
            })
            .to_object(py),
        )
    }

    #[pyo3(signature = (ptr, arg, apply_modulo_to_args = true))]
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

        let pointer = ptr
            .get_relocatable()
            .ok_or_else(|| PyValueError::new_err("Cannot write to a non-relocatable pointer."))?;
        (*self.pyvm.vm)
            .borrow_mut()
            .load_data(pointer, &data)
            .map(|x| PyMaybeRelocatable::from(x).to_object(py))
            .map_err(to_py_error)
    }

    /// Return a value from memory given its address.
    pub fn get(&self, py: Python, key: &PyRelocatable) -> Option<PyObject> {
        self.pyvm
            .vm
            .borrow()
            .get_maybe(key)
            .map(|x| PyMaybeRelocatable::from(x).to_object(py))
    }

    /// Return a list of values from memory given an initial address and a length.
    pub fn get_range(&self, py: Python, key: &PyRelocatable, size: usize) -> PyResult<PyObject> {
        let pointer: Relocatable = key.into();

        Ok(self
            .pyvm
            .vm
            .borrow()
            .get_continuous_range(pointer, size)
            .map_err(to_py_error)?
            .into_iter()
            .map(PyMaybeRelocatable::from)
            .collect::<Vec<_>>()
            .to_object(py))
    }

    /**  Converts typed arguments to cairo friendly ones
    The args received should be an iterable with an __annotations__ attribute with a values method
    which returns an iterable containing the types of each of the elements in args
    These types should de TypePointer, TypeFelt or TypeStruct
    This method is meant to process starknet's current typed arguments structure and shouldnt be used in any other case
    **/
    fn gen_typed_args(&self, py: Python<'_>, args: Py<PyAny>) -> PyResult<PyObject> {
        let args_iter = PyIterator::from_object(py, &args)?;
        let annotations_values = args
            .getattr(py, "__annotations__")?
            .call_method0(py, "values")?;

        let annotation_values = PyIterator::from_object(py, &annotations_values);

        let mut cairo_args = Vec::new();
        for (value, field_type) in std::iter::zip(args_iter, annotation_values?) {
            let type_str = format!("{:?}", field_type?);
            let type_str = type_str
                .rsplit('.')
                .next()
                .ok_or_else(|| PyTypeError::new_err("gen_typed_args: Failed to get arg type"))?
                .trim_end_matches("'>");

            if type_str == "TypePointer" || type_str == "TypeFelt" {
                cairo_args.push(self.gen_arg(py, value?.to_object(py), true)?)
            } else if type_str == "TypeStruct" {
                cairo_args.extend(self.gen_typed_args(py, value?.to_object(py)));
            } else {
                return Err(PyValueError::new_err(format!(
                    "Failed to generate typed arguments: {type_str:?} is not supported"
                )));
            }
        }

        Ok(cairo_args.to_object(py))
    }

    /// Add (or replace if already present) a custom hash builtin.
    /// Returns a Relocatable with the new hash builtin base.
    pub fn add_additional_hash_builtin(&self) -> PyRelocatable {
        let mut vm = (*self.pyvm.vm).borrow_mut();
        self.inner.add_additional_hash_builtin(&mut vm).into()
    }

    #[getter]
    fn segments(&self) -> PySegmentManager {
        PySegmentManager::new(&self.pyvm, PyMemory::new(&self.pyvm))
    }

    #[getter]
    pub fn memory(&self) -> PyMemory {
        PyMemory::new(&self.pyvm)
    }

    #[getter]
    pub fn vm(&self) -> PyVM {
        self.pyvm.clone()
    }

    #[getter]
    pub fn vm_memory(&self) -> PyMemory {
        PyMemory::new(&self.pyvm)
    }
}

pyo3::import_exception!(starkware.cairo.lang.vm.vm_exceptions, VmException);

impl PyCairoRunner {
    fn as_vm_exception(&self, error: PyErr) -> PyErr {
        let pc = self.pyvm.vm.borrow().get_pc().offset;
        let instruction_location = get_location(pc, &self.inner, self.pyvm.failed_hint_index)
            .map(InstructionLocation::from);
        let error_attribute = get_error_attr_value(pc, &self.inner, &self.pyvm.vm.borrow());
        let traceback = get_traceback(&self.pyvm.vm.borrow(), &self.inner);
        VmException::new_err((
            PyRelocatable::from((0, pc)),
            instruction_location,
            error,
            error_attribute,
            traceback,
        ))
    }
}

#[derive(Clone, FromPyObject)]
pub struct PyRunResources {
    n_steps: Option<usize>,
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
    fn builtin_instance_counter(&self) -> HashMap<String, usize> {
        let mut instance_counters = self.0.builtin_instance_counter.clone();
        // replace the builtin name with "<name>_builtin" as expected in the Starknet code.
        for builtin_name in self.0.builtin_instance_counter.keys() {
            if let Some((key, counter)) = instance_counters.remove_entry(builtin_name) {
                instance_counters
                    .entry(format!("{key}_builtin").to_string())
                    .or_insert(counter);
            }
        }

        instance_counters
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::biguint;
    use crate::relocatable::PyMaybeRelocatable::RelocatableValue;
    use cairo_felt::Felt252;
    use num_bigint::BigUint;
    use std::env::temp_dir;
    use std::fs;

    use type_samples::*;

    pub mod type_samples {
        use super::*;
        /* First we need to create a structure that behaves similarly to starknet's typed args
        This means we need:
        A: An iterable object
        B: An object that has an __annotations__ attribute
        C: The __annotations__  attribute should have a values method
        D: Values must return an iterable object containing the arg's type for each of the elements in args
        F: This iterable object must yield the following format string when format!("{:?}") is applied to it:
            **.Type or **.Type'>
        Where Type can be either TypeFelt, TypePointer or TypeStruct
        */

        // We first create the iterable pyclass (A)
        #[pyclass(unsendable)]
        pub struct MyIterator {
            pub iter: Box<dyn Iterator<Item = PyObject>>,
            pub types: Vec<PyType>,
        }

        // This pyclass implements the iterator dunder methods __iter__ and __next__
        // We then implement a __getattr__ that will allow us to call Object.__annotations__ (B)
        // This method returns a second object, so that we can then implement the values() method

        #[pymethods]
        impl MyIterator {
            #[getter]
            pub fn __annotations__(&self) -> PyResult<Annotations> {
                Ok(Annotations(self.types.clone()))
            }
            pub fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
                slf
            }
            pub fn __next__(mut slf: PyRefMut<Self>) -> Option<PyObject> {
                slf.iter.next()
            }
        }
        #[pyclass(unsendable)]
        pub struct Annotations(Vec<PyType>);

        // We implement the values method (C), which in turn returns another object so that we can override its representation
        #[pymethods]
        impl Annotations {
            pub fn values(&self) -> PyResult<Vec<PyType>> {
                Ok(self.0.clone())
            }
        }

        #[pyclass]
        #[derive(Clone)]
        pub enum PyType {
            TypePointer,
            TypeFelt,
            TypeStruct,
            // this value is added to test invalid types
            BigInt,
        }

        #[pyclass]
        #[derive(Clone)]
        pub struct TypeFelt;

        // We override the __repr__ method, so that we can customize the string we get when calling format!({:?}) (F)
        #[pymethods]
        impl TypeFelt {
            fn __repr__(&self) -> String {
                "TypeFelt".to_string()
            }
        }

        #[pyclass]
        #[derive(Clone)]
        pub struct TypePointer;

        // We override the __repr__ method, so that we can customize the string we get when calling format!({:?}) (F)
        #[pymethods]
        impl TypePointer {
            fn __repr__(&self) -> String {
                "TypePointer".to_string()
            }
        }

        #[pyclass]
        #[derive(Clone)]
        pub struct TypeStruct;

        // We override the __repr__ method, so that we can customize the string we get when calling format!({:?}) (F)
        #[pymethods]
        impl TypeStruct {
            fn __repr__(&self) -> String {
                "TypeStruct".to_string()
            }
        }
    }

    #[test]
    fn create_cairo_runner() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        PyCairoRunner::new(program, Some("main".to_string()), None, false).unwrap();
    }

    #[test]
    fn initialize_runner() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner =
            PyCairoRunner::new(program, Some("main".to_string()), None, false).unwrap();
        runner.initialize().unwrap();
    }

    #[test]
    fn runner_relocate() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner =
            PyCairoRunner::new(program, Some("main".to_string()), None, false).unwrap();
        let address = runner.initialize().unwrap();
        runner.run_until_pc(&address, None).unwrap();
        runner.relocate().unwrap();
    }

    #[test]
    fn write_output() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
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
            Some("main".to_string()),
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
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();
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
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();

        let expected_output: Vec<PyMaybeRelocatable> = vec![RelocatableValue(PyRelocatable {
            segment_index: 2,
            offset: 0,
        })];

        Python::with_gil(|py| {
            assert_eq!(
                runner
                    .get_program_builtins_initial_stack(py)
                    .extract::<Vec<PyMaybeRelocatable>>(py)
                    .unwrap(),
                expected_output
            );
        });
    }

    #[test]
    fn get_builtins_initial_stack_two_builtins() {
        let path = "cairo_programs/keccak_copy_inputs.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("all_cairo".to_string()),
            false,
        )
        .unwrap();

        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();

        let expected_output: Vec<PyMaybeRelocatable> = vec![
            RelocatableValue(PyRelocatable {
                segment_index: 2,
                offset: 0,
            }),
            RelocatableValue(PyRelocatable {
                segment_index: 3,
                offset: 0,
            }),
        ];

        Python::with_gil(|py| {
            assert_eq!(
                runner
                    .get_program_builtins_initial_stack(py)
                    .extract::<Vec<PyMaybeRelocatable>>(py)
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
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();

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
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();

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
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("all_cairo".to_string()),
            false,
        )
        .unwrap();

        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();

        assert_eq!(runner.pyvm.vm.borrow().get_ap(), Relocatable::from((1, 41)));
        assert_eq!(
            runner
                .pyvm
                .vm
                .borrow()
                .get_maybe(&Relocatable::from((1, 40)))
                .unwrap(),
            MaybeRelocatable::from((3, 20))
        );
        assert_eq!(
            runner
                .pyvm
                .vm
                .borrow()
                .get_maybe(&Relocatable::from((1, 39)))
                .unwrap(),
            MaybeRelocatable::from((2, 0))
        );

        let expected_output = PyRelocatable::from((1, 39));
        let final_stack = PyRelocatable::from((1, 41));

        assert_eq!(
            runner.get_builtins_final_stack(final_stack).unwrap(),
            expected_output
        );
    }

    #[test]
    fn get_segment_used_size_of_segment_0() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner =
            PyCairoRunner::new(program, Some("main".to_string()), None, false).unwrap();
        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();
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
        let mut runner =
            PyCairoRunner::new(program, Some("main".to_string()), None, false).unwrap();
        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();
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
    fn failed_to_get_segment_used_size() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner =
            PyCairoRunner::new(program, Some("main".to_string()), None, false).unwrap();
        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();

        Python::with_gil(|py| assert!(runner.get_segment_used_size(100, py).is_err()));
    }

    #[test]
    fn cairo_run_py_with_hint_locals() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let hint_locals = Some(HashMap::from([(
            "__find_element_max_size".to_string(),
            Python::with_gil(|py| -> PyObject { 100.to_object(py) }),
        )]));
        let mut runner =
            PyCairoRunner::new(program, Some("main".to_string()), None, false).unwrap();

        runner
            .cairo_run_py(false, None, None, hint_locals, None, None)
            .unwrap();

        Python::with_gil(|py| assert!(runner.get_segment_used_size(100, py).is_err()));
    }

    #[test]
    fn run_from_entrypoint_without_args() {
        let path = "cairo_programs/not_main.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("plain".to_string()),
            false,
        )
        .unwrap();

        runner
            .initialize_function_runner()
            .expect("Failed to initialize function runner");

        Python::with_gil(|py| {
            runner
                .run_from_entrypoint(
                    py,
                    py.eval("0", None, None).unwrap(),
                    Vec::<&PyAny>::new().to_object(py),
                    None,
                    None,
                    Some(false),
                    None,
                    None,
                    None,
                )
                .unwrap();
        });
    }

    #[test]
    fn run_from_entrypoint_with_false_apply_module_to_args_and_false_typed_args() {
        let path = "cairo_programs/not_main.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("plain".to_string()),
            false,
        )
        .unwrap();

        runner
            .initialize_function_runner()
            .expect("Failed to initialize function runner");

        Python::with_gil(|py| {
            let args = vec![
                Into::<PyMaybeRelocatable>::into(MaybeRelocatable::from((0, 0))).to_object(py),
                Into::<PyMaybeRelocatable>::into(MaybeRelocatable::from((0, 1))).to_object(py),
            ]
            .to_object(py);

            runner
                .run_from_entrypoint(
                    py,
                    py.eval("0", None, None).unwrap(),
                    args,
                    None,
                    None,
                    Some(false),
                    None,
                    None,
                    Some(false),
                )
                .unwrap();
        });
    }

    #[test]
    fn run_from_entrypoint_with_false_apply_module_to_args_and_true_typed_args() {
        let path = "cairo_programs/not_main.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("plain".to_string()),
            false,
        )
        .unwrap();

        runner
            .initialize_function_runner()
            .expect("Failed to initialize function runner");

        Python::with_gil(|py| {
            let args = MyIterator {
                iter: Box::new(
                    vec![
                        PyMaybeRelocatable::from(biguint!(0_u32)).to_object(py),
                        PyMaybeRelocatable::from(biguint!(2_u32)).to_object(py),
                    ]
                    .into_iter(),
                ),
                types: vec![PyType::TypeFelt, PyType::TypeFelt],
            }
            .into_py(py);

            runner
                .run_from_entrypoint(
                    py,
                    py.eval("0", None, None).unwrap(),
                    args,
                    None,
                    None,
                    Some(true),
                    None,
                    None,
                    Some(false),
                )
                .unwrap();
        });
    }

    #[test]
    fn error_when_run_from_entrypoint_with_invalid_args() {
        let path = "cairo_programs/not_main.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("plain".to_string()),
            false,
        )
        .unwrap();

        runner
            .initialize_function_runner()
            .expect("Failed to initialize function runner");

        Python::with_gil(|py| {
            // invalid arguments
            let args =
                vec![vec![vec![
                    Into::<PyMaybeRelocatable>::into(MaybeRelocatable::from((0, 0))).to_object(py),
                ]
                .to_object(py)]
                .to_object(py)]
                .to_object(py);

            assert!(runner
                .run_from_entrypoint(
                    py,
                    py.eval("0", None, None).unwrap(),
                    args,
                    None,
                    None,
                    Some(false),
                    None,
                    None,
                    None,
                )
                .is_err());
        });
    }

    #[test]
    fn run_from_entrypoint_with_string_name() {
        let path = "cairo_programs/not_main.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("plain".to_string()),
            false,
        )
        .unwrap();

        runner
            .initialize_function_runner()
            .expect("Failed to initialize function runner");

        Python::with_gil(|py| {
            let result = runner.run_from_entrypoint(
                py,
                py.eval("'not_main'", None, None).unwrap(),
                Vec::<&PyAny>::new().to_object(py),
                None,
                None,
                Some(false),
                None,
                None,
                None,
            );
            // using a named entrypoint in run_from_entrypoint is not implemented yet
            assert_eq!(
                format!("{:?}", result),
                format!("{:?}", Err::<(), PyErr>(PyNotImplementedError::new_err(())))
            );
        });
    }

    #[test]
    fn run_from_entrypoint_limited_resources() {
        let path = "cairo_programs/not_main.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("plain".to_string()),
            false,
        )
        .unwrap();

        runner
            .initialize_function_runner()
            .expect("Failed to initialize function runner");
        let pc_before_run = runner.pyvm.vm.borrow().get_pc();

        Python::with_gil(|py| {
            let result = runner.run_from_entrypoint(
                py,
                py.eval("0", None, None).unwrap(),
                Vec::<&PyAny>::new().to_object(py),
                None,
                None,
                Some(false),
                None,
                Some(PyRunResources { n_steps: Some(0) }),
                None,
            );
            assert!(result.is_err());
            assert!(format!("{:?}", result).contains("Execution reached the end of the program."));
        });

        let pc_after_run = runner.pyvm.vm.borrow().get_pc();

        // As the run_resurces provide 0 steps, no steps should have been run
        // To check this, we check that the pc hasnt changed after "running" the vm
        assert_eq!(pc_before_run, pc_after_run);
    }

    #[test]
    fn run_from_entrypoint_with_invalid_entrypoint() {
        let path = "cairo_programs/not_main.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("plain".to_string()),
            false,
        )
        .unwrap();

        runner
            .initialize_function_runner()
            .expect("Failed to initialize function runner");

        Python::with_gil(|py| {
            let result = runner.run_from_entrypoint(
                py,
                py.eval("[]", None, None).unwrap(),
                Vec::<&PyAny>::new().to_object(py),
                None,
                None,
                Some(false),
                None,
                None,
                None,
            );
            assert_eq!(
                format!("{:?}", result),
                format!(
                    "{:?}",
                    Err::<(), PyErr>(PyTypeError::new_err("entrypoint must be int or str"))
                )
            );
        });
    }

    #[test]
    fn run_from_entrypoint_without_args_set_hint_locals() {
        let path = "cairo_programs/not_main.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("plain".to_string()),
            false,
        )
        .unwrap();

        runner.initialize_segments();

        Python::with_gil(|py| {
            runner
                .run_from_entrypoint(
                    py,
                    py.eval("0", None, None).unwrap(),
                    Vec::<&PyAny>::new().to_object(py),
                    Some(HashMap::from([(
                        String::from("syscall_handler"),
                        1.to_object(py),
                    )])),
                    None,
                    Some(false),
                    None,
                    None,
                    None,
                )
                .unwrap();
            assert!(!runner.hint_locals.is_empty());
            assert_eq!(
                runner
                    .hint_locals
                    .get("syscall_handler")
                    .unwrap()
                    .extract::<usize>(py)
                    .unwrap(),
                1
            )
        });
    }

    #[test]
    fn run_from_entrypoint_without_args_set_static_locals() {
        let path = "cairo_programs/not_main.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("plain".to_string()),
            false,
        )
        .unwrap();

        runner.initialize_segments();

        Python::with_gil(|py| {
            runner
                .run_from_entrypoint(
                    py,
                    py.eval("0", None, None).unwrap(),
                    Vec::<&PyAny>::new().to_object(py),
                    None,
                    Some(HashMap::from([(
                        String::from("__keccak_max_size"),
                        100.to_object(py),
                    )])),
                    Some(false),
                    None,
                    None,
                    None,
                )
                .unwrap();
            assert!(!runner.static_locals.as_ref().unwrap().is_empty());
            assert_eq!(
                runner
                    .static_locals
                    .as_ref()
                    .unwrap()
                    .get("__keccak_max_size")
                    .unwrap()
                    .extract::<usize>(py)
                    .unwrap(),
                100
            )
        });
    }

    #[test]
    fn run_from_entrypoint_with_one_typed_arg() {
        let test_fails_with_zero = |value: usize, py: &Python| {
            let path = "cairo_programs/assert_not_zero.json".to_string();
            let program = fs::read_to_string(path).unwrap();
            let mut runner = PyCairoRunner::new(
                program,
                Some("main".to_string()),
                Some("plain".to_string()),
                false,
            )
            .unwrap();

            runner.initialize_segments();

            let args = MyIterator {
                iter: Box::new(
                    vec![PyMaybeRelocatable::from(biguint!(value)).to_object(*py)].into_iter(),
                ),
                types: vec![PyType::TypeFelt],
            };
            runner.run_from_entrypoint(
                *py,
                py.eval("0", None, None).unwrap(),
                args.into_py(*py),
                None,
                None,
                Some(true),
                None,
                None,
                None,
            )
        };
        Python::with_gil(|py| {
            // program fails if argument is zero
            assert!(test_fails_with_zero(0, &py).is_err());
            // but doesn't with nonzero argument
            assert!(test_fails_with_zero(1, &py).is_ok());
            assert!(test_fails_with_zero(2, &py).is_ok());
        });
    }

    #[test]
    fn run_from_entrypoint_with_multiple_untyped_args() {
        let path = "cairo_programs/array_sum.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("plain".to_string()),
            false,
        )
        .unwrap();

        runner.initialize_segments();

        Python::with_gil(|py| {
            let array = vec![
                PyMaybeRelocatable::from(biguint!(1_u32)).to_object(py),
                PyMaybeRelocatable::from(biguint!(2_u32)).to_object(py),
                PyMaybeRelocatable::from(biguint!(4_u32)).to_object(py),
                PyMaybeRelocatable::from(biguint!(8_u32)).to_object(py),
            ];
            let size = PyMaybeRelocatable::from(biguint!(array.len()));
            let args = vec![array.into_py(py), size.to_object(py)];
            let result = runner.run_from_entrypoint(
                py,
                py.eval("7", None, None).unwrap(),
                args.into_py(py),
                None,
                None,
                Some(false),
                None,
                None,
                None,
            );

            assert!(result.is_ok());

            let return_value: MaybeRelocatable = runner
                .get_return_values(1, py)
                .unwrap()
                .extract::<Vec<PyMaybeRelocatable>>(py)
                .expect("failed to get return value")
                .first()
                .expect("there's no return value")
                .into();
            assert_eq!(return_value, MaybeRelocatable::from(15));
        });
    }

    #[test]
    fn insert() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let runner = PyCairoRunner::new(program, Some("main".to_string()), None, true).unwrap();

        (*runner.pyvm.get_vm()).borrow_mut().add_memory_segment();
        runner
            .insert(&(0, 0).into(), PyMaybeRelocatable::Int(biguint!(3_u32)))
            .unwrap();
        runner
            .insert(&(0, 1).into(), PyMaybeRelocatable::Int(biguint!(4_u32)))
            .unwrap();
        runner
            .insert(&(0, 2).into(), PyMaybeRelocatable::Int(biguint!(5_u32)))
            .unwrap();
        assert_eq!(
            (*runner.pyvm.get_vm())
                .borrow()
                .get_continuous_range((0, 0).into(), 3),
            Ok(vec![3.into(), 4.into(), 5.into(),]),
        )
    }

    /// Test that `PyCairoRunner::insert()` fails when it should.
    #[test]
    fn insert_duplicate() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let runner = PyCairoRunner::new(program, Some("main".to_string()), None, true).unwrap();

        (*runner.pyvm.get_vm()).borrow_mut().add_memory_segment();
        runner
            .insert(&(0, 0).into(), PyMaybeRelocatable::Int(biguint!(3_u32)))
            .unwrap();
        runner
            .insert(&(0, 1).into(), PyMaybeRelocatable::Int(biguint!(4_u32)))
            .unwrap();
        runner
            .insert(&(0, 0).into(), PyMaybeRelocatable::Int(biguint!(5_u32)))
            .expect_err("insertion succeeded when it should've failed");
        assert_eq!(
            (*runner.pyvm.get_vm())
                .borrow()
                .get_continuous_range((0, 0).into(), 2),
            Ok(vec![3.into(), 4.into(),]),
        );
    }

    #[test]
    fn get_initial_fp_test() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some(String::from("all_cairo")),
            false,
        )
        .unwrap();
        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();
        assert_eq! {
            PyRelocatable::from((1,2)),
            runner.initial_fp().unwrap()
        };
    }

    #[test]
    fn failed_to_get_initial_fp_test() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some(String::from("all_cairo")),
            false,
        )
        .unwrap();

        assert!(runner.initial_fp().is_err());
    }

    #[test]
    fn initialize_function_runner() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("all_cairo".to_string()),
            false,
        )
        .unwrap();

        runner
            .initialize_function_runner()
            .expect("Failed to initialize function runner");

        let expected_output: Vec<Vec<PyMaybeRelocatable>> = vec![
            vec![RelocatableValue(PyRelocatable {
                segment_index: 2,
                offset: 0,
            })],
            vec![RelocatableValue(PyRelocatable {
                segment_index: 3,
                offset: 0,
            })],
            vec![RelocatableValue(PyRelocatable {
                segment_index: 4,
                offset: 0,
            })],
            vec![RelocatableValue(PyRelocatable {
                segment_index: 5,
                offset: 0,
            })],
            vec![RelocatableValue(PyRelocatable {
                segment_index: 6,
                offset: 0,
            })],
            vec![RelocatableValue(PyRelocatable {
                segment_index: 7,
                offset: 0,
            })],
            vec![RelocatableValue(PyRelocatable {
                segment_index: 8,
                offset: 0,
            })],
            vec![RelocatableValue(PyRelocatable {
                segment_index: 9,
                offset: 0,
            })],
        ];

        Python::with_gil(|py| {
            assert_eq!(
                runner
                    .get_builtins_initial_stack(py)
                    .extract::<Vec<Vec<PyMaybeRelocatable>>>(py)
                    .unwrap(),
                expected_output
            );
        });
    }

    #[test]
    fn program_builtins_initial_stack_are_empty_when_no_program_builtins() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("all_cairo".to_string()),
            false,
        )
        .unwrap();

        runner
            .initialize_function_runner()
            .expect("Failed to initialize function runner");

        let expected_output: Vec<Vec<PyMaybeRelocatable>> = vec![];

        Python::with_gil(|py| {
            assert_eq!(
                runner
                    .get_program_builtins_initial_stack(py)
                    .extract::<Vec<Vec<PyMaybeRelocatable>>>(py)
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
            let runner = PyCairoRunner::new(
                program,
                Some("main".to_string()),
                Some("all_cairo".to_string()),
                false,
            )
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
            let vm_ref = (*vm_ref).borrow();

            assert_eq!(
                vm_ref
                    .get_maybe(&Relocatable::from((0, 0)))
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &Felt252::new(1),
            );
            assert_eq!(
                vm_ref
                    .get_maybe(&Relocatable::from((0, 1)))
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &Felt252::new(2),
            );

            let relocatable = vm_ref
                .get_maybe(&Relocatable::from((0, 2)))
                .unwrap()
                .get_relocatable()
                .unwrap();

            assert_eq!(
                vm_ref
                    .get_maybe(&relocatable)
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &Felt252::new(3),
            );
            assert_eq!(
                vm_ref
                    .get_maybe(&(&relocatable + 1))
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &Felt252::new(4),
            );
            assert!(vm_ref.get_maybe(&(&relocatable + 2)).is_none());

            let relocatable = vm_ref
                .get_maybe(&Relocatable::from((0, 3)))
                .unwrap()
                .get_relocatable()
                .unwrap();

            assert_eq!(
                vm_ref
                    .get_maybe(&relocatable)
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &Felt252::new(5),
            );
            assert_eq!(
                vm_ref
                    .get_maybe(&(&relocatable + 1))
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &Felt252::new(6),
            );
            assert!(vm_ref.get_maybe(&(&relocatable + 2)).is_none());

            assert!(vm_ref.get_maybe(&Relocatable::from((0, 4))).is_none());
        });
    }

    #[test]
    fn run_find_element_with_max_size() {
        let path = "cairo_programs/find_element.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("all_cairo".to_string()),
            false,
        )
        .unwrap();
        assert!(runner
            .cairo_run_py(
                false,
                None,
                None,
                None,
                Some(HashMap::from([(
                    "__find_element_max_size".to_string(),
                    Python::with_gil(|py| -> PyObject { 100.to_object(py) }),
                )])),
                None,
            )
            .is_ok());
    }

    #[test]
    fn run_find_element_with_max_size_low_size() {
        let path = "cairo_programs/find_element.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("all_cairo".to_string()),
            false,
        )
        .unwrap();
        assert!(runner
            .cairo_run_py(
                false,
                None,
                None,
                None,
                Some(HashMap::from([(
                    "__find_element_max_size".to_string(),
                    Python::with_gil(|py| -> PyObject { 1.to_object(py) }),
                )])),
                None
            )
            .is_err());
    }

    #[test]
    fn set_entrypoint() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner =
            PyCairoRunner::new(program, None, Some("small".to_string()), false).unwrap();

        runner
            .cairo_run_py(false, None, None, None, None, Some("main"))
            .expect("Call to PyCairoRunner::cairo_run_py() failed.");
    }

    #[test]
    fn set_bad_entrypoint_on_new() {
        let path = "cairo_programs/fibonacci.json".to_string();

        let program = fs::read_to_string(path).unwrap();
        let result = PyCairoRunner::new(
            program,
            Some(" non-existent entrypoint".to_string()),
            Some("small".to_string()),
            false,
        );

        assert!(result.is_err());
    }

    /// Test that `PyCairoRunner::get()` works as intended.
    #[test]
    fn get() {
        Python::with_gil(|py| {
            let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
            let mut runner = PyCairoRunner::new(
                program,
                Some("main".to_string()),
                Some("small".to_string()),
                false,
            )
            .unwrap();

            runner
                .cairo_run_py(false, None, None, None, None, None)
                .expect("Call to PyCairoRunner::cairo_run_py");

            let mut ap = runner.get_ap().unwrap();
            ap.offset -= 1;
            assert_eq!(
                runner
                    .get(py, &ap)
                    .map(|x| MaybeRelocatable::from(x.extract::<PyMaybeRelocatable>(py).unwrap())),
                Some(MaybeRelocatable::from(144)),
            );
        });
    }

    /// Test that `PyCairoRunner::get_range()` works as intended.
    #[test]
    fn get_range() {
        Python::with_gil(|py| {
            let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
            let runner = PyCairoRunner::new(
                program,
                Some("main".to_string()),
                Some("small".to_string()),
                false,
            )
            .unwrap();

            let ptr = {
                let mut vm = (*runner.pyvm.vm).borrow_mut();
                let ptr = vm.add_memory_segment();
                vm.load_data(ptr, &vec![1.into(), 2.into(), 3.into(), 4.into(), 5.into()])
                    .unwrap();

                ptr
            };

            assert_eq!(
                runner
                    .get_range(py, &PyRelocatable::from(ptr), 5)
                    .unwrap()
                    .extract::<Vec<PyMaybeRelocatable>>(py)
                    .unwrap()
                    .into_iter()
                    .map(MaybeRelocatable::from)
                    .collect::<Vec<_>>(),
                vec![1.into(), 2.into(), 3.into(), 4.into(), 5.into(),],
            );
        });
    }

    /// Test that add_additional_hash_builtin() returns successfully.
    #[test]
    fn add_additional_hash_builtin() {
        Python::with_gil(|_| {
            let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
            let runner = PyCairoRunner::new(
                program,
                Some("main".to_string()),
                Some("small".to_string()),
                false,
            )
            .unwrap();

            let expected_relocatable = PyRelocatable {
                segment_index: 0,
                offset: 0,
            };
            let relocatable = runner.add_additional_hash_builtin();
            assert_eq!(expected_relocatable, relocatable);

            let mut vm = (*runner.pyvm.vm).borrow_mut();
            // Check that the segment exists by writing to it.
            vm.insert_value(Relocatable::from((0, 0)), MaybeRelocatable::from(42))
                .expect("memory insert failed");
        });
    }

    #[test]
    fn gen_typed_args_type_felt() {
        //For documentation on how this test works see test submodule type_samples
        let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
        let runner = PyCairoRunner::new(program, None, None, false).unwrap();
        Python::with_gil(|py| {
            // We create an iterable object containing elements which match the type we defined in (F), thus fullfilling (D)
            let arg = MyIterator {
                iter: Box::new(
                    vec![
                        PyMaybeRelocatable::from(biguint!(0_u32)).to_object(py),
                        PyMaybeRelocatable::from(biguint!(2_u32)).to_object(py),
                    ]
                    .into_iter(),
                ),
                types: vec![PyType::TypeFelt, PyType::TypeFelt],
            };
            let stack = runner.gen_typed_args(py, arg.into_py(py)).unwrap();
            let stack = stack.extract::<Vec<PyMaybeRelocatable>>(py).unwrap();

            // We compare the output of gen_typed_args to our expected cairo-firendly arguments
            assert_eq!(
                stack,
                vec![
                    PyMaybeRelocatable::from(biguint!(0_u32)),
                    PyMaybeRelocatable::from(biguint!(2_u32)),
                ]
            );
        })
    }

    #[test]
    fn gen_typed_args_type_pointer() {
        //For documentation on how this test works see test submodule type_samples

        let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
        let runner = PyCairoRunner::new(program, None, None, false).unwrap();
        Python::with_gil(|py| {
            let arg = MyIterator {
                iter: Box::new(
                    vec![
                        Into::<PyMaybeRelocatable>::into(MaybeRelocatable::from((0, 0)))
                            .to_object(py),
                        Into::<PyMaybeRelocatable>::into(MaybeRelocatable::from((0, 1)))
                            .to_object(py),
                    ]
                    .into_iter(),
                ),
                types: vec![PyType::TypePointer, PyType::TypePointer],
            };

            let stack = runner.gen_typed_args(py, arg.into_py(py)).unwrap();
            let stack = stack.extract::<Vec<PyMaybeRelocatable>>(py).unwrap();
            assert_eq!(
                stack,
                vec![
                    MaybeRelocatable::from((0, 0)).into(),
                    MaybeRelocatable::from((0, 1)).into(),
                ]
            );
        })
    }

    #[test]
    fn segments() {
        let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
        let runner = PyCairoRunner::new(program, None, None, false).unwrap();

        let segments = runner.segments();

        Python::with_gil(|py| {
            let segment = segments.add().expect("Could not add segemnt.");
            segments
                .write_arg(
                    py,
                    segment.clone().into(),
                    py.eval("[1, 2, 3, 4]", None, None).unwrap().to_object(py),
                    false,
                )
                .unwrap();

            let get_value = |addr: &PyRelocatable, offset| {
                let addr = addr.__add__(offset);
                runner
                    .get(py, &addr)
                    .map(|x| x.extract::<BigUint>(py))
                    .transpose()
                    .expect("Could not convert value to a biguint")
            };
            assert_eq!(get_value(&segment, 0), Some(biguint!(1_u32)));
            assert_eq!(get_value(&segment, 1), Some(biguint!(2_u32)));
            assert_eq!(get_value(&segment, 2), Some(biguint!(3_u32)));
            assert_eq!(get_value(&segment, 3), Some(biguint!(4_u32)));
            assert_eq!(get_value(&segment, 4), None);
        });
    }

    #[test]
    fn gen_typed_args_type_struct() {
        //For documentation on how this test works see test submodule type_samples

        let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
        let runner = PyCairoRunner::new(program, None, None, false).unwrap();
        Python::with_gil(|py| {
            let arg = MyIterator {
                iter: Box::new(
                    vec![
                        MyIterator {
                            iter: Box::new(
                                vec![
                                    Into::<PyMaybeRelocatable>::into(MaybeRelocatable::from((
                                        0, 0,
                                    )))
                                    .to_object(py),
                                    Into::<PyMaybeRelocatable>::into(MaybeRelocatable::from((
                                        0, 1,
                                    )))
                                    .to_object(py),
                                ]
                                .into_iter(),
                            ),
                            types: vec![PyType::TypePointer, PyType::TypePointer],
                        }
                        .into_py(py),
                        MyIterator {
                            iter: Box::new(
                                vec![
                                    Into::<PyMaybeRelocatable>::into(MaybeRelocatable::from((
                                        0, 0,
                                    )))
                                    .to_object(py),
                                    Into::<PyMaybeRelocatable>::into(MaybeRelocatable::from((
                                        0, 1,
                                    )))
                                    .to_object(py),
                                ]
                                .into_iter(),
                            ),
                            types: vec![PyType::TypePointer, PyType::TypePointer],
                        }
                        .into_py(py),
                    ]
                    .into_iter(),
                ),
                types: vec![PyType::TypeStruct, PyType::TypeStruct],
            };

            let stack = runner.gen_typed_args(py, arg.into_py(py)).unwrap();
            let stack = stack.extract::<Vec<Py<PyAny>>>(py).unwrap();
            for value in stack.iter() {
                let stack = value.extract::<Vec<PyMaybeRelocatable>>(py).unwrap();
                assert_eq!(
                    stack,
                    vec![
                        MaybeRelocatable::from((0, 0)).into(),
                        MaybeRelocatable::from((0, 1)).into(),
                    ]
                );
            }
        })
    }

    #[test]
    fn error_when_gen_typed_args_with_invalid_type() {
        //For documentation on how this test works see test submodule type_samples

        let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
        let runner = PyCairoRunner::new(program, None, None, false).unwrap();
        Python::with_gil(|py| {
            let arg = MyIterator {
                iter: Box::new(
                    vec![
                        PyMaybeRelocatable::from(biguint!(0_u32)).to_object(py),
                        PyMaybeRelocatable::from(biguint!(2_u32)).to_object(py),
                    ]
                    .into_iter(),
                ),
                types: vec![PyType::BigInt, PyType::BigInt],
            };

            assert!(runner.gen_typed_args(py, arg.into_py(py)).is_err());
        })
    }

    #[test]
    fn memory() {
        let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
        let runner = PyCairoRunner::new(program, None, None, false).unwrap();

        let memory = runner.memory();

        Python::with_gil(|py| {
            let segment = runner.add_segment();

            let set_value = |addr: &PyRelocatable, offset, value: BigUint| {
                let addr = addr.__add__(offset);
                memory
                    .__setitem__(&addr, PyMaybeRelocatable::Int(value))
                    .expect("Could not insert value into memory.");
            };
            let get_value = |addr: &PyRelocatable, offset| {
                let addr = addr.__add__(offset);
                memory
                    .__getitem__(&addr, py)
                    .map(|x| x.extract::<BigUint>(py))
                    .transpose()
                    .expect("Could not convert value to a biguint")
            };

            set_value(&segment, 0, biguint!(1_u32));
            set_value(&segment, 1, biguint!(2_u32));
            set_value(&segment, 2, biguint!(3_u32));
            set_value(&segment, 3, biguint!(4_u32));

            assert_eq!(get_value(&segment, 0), Some(biguint!(1_u32)));
            assert_eq!(get_value(&segment, 1), Some(biguint!(2_u32)));
            assert_eq!(get_value(&segment, 2), Some(biguint!(3_u32)));
            assert_eq!(get_value(&segment, 3), Some(biguint!(4_u32)));
            assert_eq!(get_value(&segment, 4), None);
        });
    }

    #[test]
    fn vm() {
        let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
        let runner = PyCairoRunner::new(program, None, None, false).unwrap();

        let vm = runner.vm();
        assert_eq!(vm.vm.as_ptr(), runner.pyvm.vm.as_ptr());
    }

    #[test]
    fn vm_memory() {
        let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
        let runner = PyCairoRunner::new(program, None, None, false).unwrap();

        let memory = runner.vm_memory();

        Python::with_gil(|py| {
            let segment = runner.add_segment();

            let set_value = |addr: &PyRelocatable, offset, value: BigUint| {
                let addr = addr.__add__(offset);
                memory
                    .__setitem__(&addr, PyMaybeRelocatable::Int(value))
                    .expect("Could not insert value into memory.");
            };
            let get_value = |addr: &PyRelocatable, offset| {
                let addr = addr.__add__(offset);
                memory
                    .__getitem__(&addr, py)
                    .map(|x| x.extract::<BigUint>(py))
                    .transpose()
                    .expect("Could not convert value to a biguint")
            };

            set_value(&segment, 0, biguint!(1_u32));
            set_value(&segment, 1, biguint!(2_u32));
            set_value(&segment, 2, biguint!(3_u32));
            set_value(&segment, 3, biguint!(4_u32));

            assert_eq!(get_value(&segment, 0), Some(biguint!(1_u32)));
            assert_eq!(get_value(&segment, 1), Some(biguint!(2_u32)));
            assert_eq!(get_value(&segment, 2), Some(biguint!(3_u32)));
            assert_eq!(get_value(&segment, 3), Some(biguint!(4_u32)));
            assert_eq!(get_value(&segment, 4), None);
        });
    }

    #[test]
    fn cairo_run_with_trace_file() {
        let path = String::from("cairo_programs/fibonacci.json");
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        let trace_path = temp_dir().join("fibonacci.trace");
        let trace_path = trace_path.to_str().unwrap();

        _ = fs::remove_file(trace_path);

        runner
            .cairo_run_py(false, Some(trace_path), None, None, None, None)
            .expect("Call to PyCairoRunner::cairo_run_py() failed.");

        // We simply check if file exists
        assert!(fs::canonicalize(trace_path).is_ok());

        _ = fs::remove_file(trace_path);
    }

    #[test]
    fn cairo_run_with_nonexistent_trace_file() {
        let path = String::from("cairo_programs/fibonacci.json");
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        let trace_path = "cairo_programs";

        let result = runner.cairo_run_py(false, Some(trace_path), None, None, None, None);

        assert!(result.is_err());
    }

    #[test]
    fn cairo_run_with_memory_file() {
        let path = String::from("cairo_programs/fibonacci.json");
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        let memory_path = temp_dir().join("fibonacci.memory");
        let memory_path = memory_path.to_str().unwrap();

        _ = fs::remove_file(memory_path);

        runner
            .cairo_run_py(false, None, Some(memory_path), None, None, None)
            .expect("Call to PyCairoRunner::cairo_run_py() failed.");

        // We simply check if file exists
        assert!(fs::canonicalize(memory_path).is_ok());

        _ = fs::remove_file(memory_path);
    }

    #[test]
    fn cairo_run_with_nonexistent_memory_file() {
        let path = String::from("cairo_programs/fibonacci.json");
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        let memory_path = "cairo_programs";

        let result = runner.cairo_run_py(false, None, Some(memory_path), None, None, None);

        assert!(result.is_err());
    }

    #[test]
    fn get_execution_resources() {
        let path = String::from("cairo_programs/array_sum.json");
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        let result = runner.cairo_run_py(false, None, None, None, None, None);

        assert!(result.is_ok());

        let exec_res = runner.get_execution_resources().unwrap();

        // n_steps is 0 because trace is disabled when trace_file is None
        assert_eq!(exec_res.n_steps(), 0);
        assert_eq!(exec_res.n_memory_holes(), 0);
        assert_eq!(
            exec_res.builtin_instance_counter(),
            HashMap::from([("output_builtin".to_string(), 1)])
        );
    }

    #[test]
    fn mark_as_accessed_run_not_finished() {
        let path = String::from("cairo_programs/fibonacci.json");
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();
        assert!(runner.mark_as_accessed((0, 0).into(), 3).is_err());
    }

    #[test]
    fn get_return_values_ok() {
        let path = String::from("cairo_programs/fibonacci.json");
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        runner
            .cairo_run_py(false, None, None, None, None, None)
            .expect("Call to PyCairoRunner::cairo_run_py() failed.");

        Python::with_gil(|py| {
            let addresses = runner
                .get_return_values(1, py)
                .unwrap()
                .extract::<Vec<PyMaybeRelocatable>>(py)
                .unwrap();
            assert_eq!(addresses.len(), 1);

            let result = match &addresses[0] {
                PyMaybeRelocatable::Int(value) => Ok(value),
                PyMaybeRelocatable::RelocatableValue(r) => Err(r),
            };
            let expected = biguint!(144_u32);
            assert_eq!(result, Ok(&expected) as Result<&BigUint, &PyRelocatable>);
        });
    }

    #[test]
    fn get_return_values_out_of_bounds() {
        let path = String::from("cairo_programs/fibonacci.json");
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        runner
            .cairo_run_py(true, None, None, None, None, None)
            .expect("Call to PyCairoRunner::cairo_run_py() failed.");

        Python::with_gil(|py| {
            let oob_error = runner.get_return_values(100, py);

            assert!(oob_error.is_err());
        });
    }

    #[test]
    fn cairo_run_with_ecdsa_builtin() {
        let path = String::from("cairo_programs/ecdsa.json");
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("all_cairo".to_string()),
            false,
        )
        .unwrap();

        assert!(runner
            .cairo_run_py(false, None, None, None, None, None)
            .is_ok());
    }
}
