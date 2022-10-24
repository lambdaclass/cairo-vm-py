use crate::ids::PyIds;
use crate::pycell;
use crate::scope_manager::{PyEnterScope, PyExitScope};
use crate::utils::to_py_error;
use crate::{
    memory::PyMemory, memory_segments::PySegmentManager, range_check::PyRangeCheck,
    relocatable::PyRelocatable, utils::to_vm_error,
};
use cairo_rs::any_box;
use cairo_rs::cairo_run::write_output;
use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor;
use cairo_rs::hint_processor::hint_processor_definition::HintProcessor;
use cairo_rs::types::exec_scope::ExecutionScopes;
use cairo_rs::types::program::Program;
use cairo_rs::types::relocatable::Relocatable;
use cairo_rs::vm::errors::cairo_run_errors::CairoRunError;
use cairo_rs::vm::errors::runner_errors::RunnerError;
use cairo_rs::vm::errors::trace_errors::TraceError;
use cairo_rs::vm::runners::cairo_runner::CairoRunner;
use cairo_rs::vm::vm_core::VirtualMachine;
use cairo_rs::{
    hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData,
    vm::errors::vm_errors::VirtualMachineError,
};
use num_bigint::BigInt;
use pyo3::{pyclass, pymethods, PyObject, ToPyObject};
use pyo3::{types::PyDict, Python};
use pyo3::{PyCell, PyResult};
use std::any::Any;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{cell::RefCell, rc::Rc};

const GLOBAL_NAMES: [&str; 16] = [
    "memory",
    "segments",
    "ap",
    "fp",
    "ids",
    "vm_enter_scope",
    "vm_exit_scope",
    "range_check_builtin",
    "PRIME",
    "__doc__",
    "__annotations__",
    "__package__",
    "__builtins__",
    "__spec__",
    "__loader__",
    "__name__"
];

#[pyclass(unsendable)]
pub struct PyVM {
    pub(crate) vm: Rc<RefCell<VirtualMachine>>,
}

#[pymethods]
impl PyVM {
    #[new]
    pub fn new(prime: BigInt, trace_enabled: bool) -> PyVM {
        PyVM {
            vm: Rc::new(RefCell::new(VirtualMachine::new(prime, trace_enabled))),
        }
    }

    #[pyo3(name = "cairo_run")]
    pub fn cairo_run_py(
        &self,
        path: String,
        entrypoint: String,
        print_output: bool,
        trace_file: Option<&str>,
        memory_file: Option<&str>,
        hint_locals: Option<HashMap<String, PyObject>>,
    ) -> PyResult<()> {
        let path = Path::new(&path);
        let program = Program::new(path, &entrypoint).map_err(to_py_error)?;
        let hint_processor = BuiltinHintProcessor::new_empty();
        let mut cairo_runner = CairoRunner::new(&program, &hint_processor).map_err(to_py_error)?;
        let end = cairo_runner
            .initialize(&mut self.vm.borrow_mut())
            .map_err(to_py_error)?;
        let mut hint_locals = hint_locals.unwrap_or_default();
        self.run_until_pc(&mut cairo_runner, &end, &mut hint_locals)
            .map_err(to_py_error)?;

        self.vm
            .borrow_mut()
            .verify_auto_deductions()
            .map_err(to_py_error)?;

        cairo_runner
            .relocate(&mut self.vm.borrow_mut())
            .map_err(to_py_error)?;

        if print_output {
            write_output(&mut cairo_runner, &mut self.vm.borrow_mut()).map_err(to_py_error)?;
        }

        if let Some(trace_path) = trace_file {
            let trace_path = PathBuf::from(trace_path);
            let relocated_trace = cairo_runner
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
            cairo_rs::cairo_run::write_binary_memory(&cairo_runner.relocated_memory, &memory_path)
                .map_err(|_| to_py_error(CairoRunError::Runner(RunnerError::WriteFail)))?;
        }

        Ok(())
    }
}

impl PyVM {
    pub(crate) fn get_vm(&self) -> Rc<RefCell<VirtualMachine>> {
        Rc::clone(&self.vm)
    }

    pub(crate) fn execute_hint(
        &self,
        hint_data: &HintProcessorData,
        hint_locals: &mut HashMap<String, PyObject>,
        exec_scopes: &mut ExecutionScopes,
    ) -> Result<(), VirtualMachineError> {
        Python::with_gil(|py| -> Result<(), VirtualMachineError> {
            let memory = PyMemory::new(self);
            let segments = PySegmentManager::new(self);
            let ap = PyRelocatable::from(self.vm.borrow().get_ap());
            let fp = PyRelocatable::from(self.vm.borrow().get_fp());
            let ids = PyIds::new(self, &hint_data.ids_data, &hint_data.ap_tracking);
            let enter_scope = pycell!(py, PyEnterScope::new());
            let exit_scope = pycell!(py, PyExitScope::new());
            let range_check_builtin =
                PyRangeCheck::from(self.vm.borrow().get_range_check_builtin());
            let prime = self.vm.borrow().get_prime().clone();

            // This line imports Python builtins. If not imported, this will run only with Python 3.10
            let mut globals = py
                .import("__main__")
                .map_err(to_vm_error)?
                .dict()
                .copy()
                .map_err(to_vm_error)?;

            add_scope_locals(&mut globals, exec_scopes)?;

            globals
                .set_item("memory", pycell!(py, memory))
                .map_err(to_vm_error)?;
            globals
                .set_item("segments", pycell!(py, segments))
                .map_err(to_vm_error)?;
            globals
                .set_item("ap", pycell!(py, ap))
                .map_err(to_vm_error)?;
            globals
                .set_item("fp", pycell!(py, fp))
                .map_err(to_vm_error)?;
            globals
                .set_item("ids", pycell!(py, ids))
                .map_err(to_vm_error)?;

            globals
                .set_item("vm_enter_scope", enter_scope)
                .map_err(to_vm_error)?;
            globals
                .set_item("vm_exit_scope", exit_scope)
                .map_err(to_vm_error)?;

            globals
                .set_item("range_check_builtin", range_check_builtin)
                .map_err(to_vm_error)?;
            globals.set_item("PRIME", prime).map_err(to_vm_error)?;

            for (name, pyobj) in hint_locals.iter() {
                globals.set_item(name, pyobj).map_err(to_vm_error)?;
            }
            py.run(&hint_data.code, Some(globals), None)
                .map_err(to_vm_error)?;

            update_scope_hint_locals(exec_scopes, hint_locals, globals, py);

            enter_scope.borrow().update_scopes(exec_scopes)?;
            exit_scope.borrow().update_scopes(exec_scopes)
        })?;

        Ok(())
    }

    pub(crate) fn step_hint(
        &self,
        hint_executor: &dyn HintProcessor,
        hint_locals: &mut HashMap<String, PyObject>,
        exec_scopes: &mut ExecutionScopes,
        hint_data_dictionary: &HashMap<usize, Vec<Box<dyn Any>>>,
    ) -> Result<(), VirtualMachineError> {
        let pc_offset = self.vm.borrow().get_pc().offset;

        if let Some(hint_list) = hint_data_dictionary.get(&pc_offset) {
            for hint_data in hint_list.iter() {
                if self.should_run_py_hint(hint_executor, exec_scopes, hint_data)? {
                    let hint_data = hint_data
                        .downcast_ref::<HintProcessorData>()
                        .ok_or(VirtualMachineError::WrongHintData)?;

                    self.execute_hint(hint_data, hint_locals, exec_scopes)?;
                }
            }
        }

        Ok(())
    }

    pub(crate) fn step(
        &self,
        hint_executor: &dyn HintProcessor,
        hint_locals: &mut HashMap<String, PyObject>,
        exec_scopes: &mut ExecutionScopes,
        hint_data_dictionary: &HashMap<usize, Vec<Box<dyn Any>>>,
    ) -> Result<(), VirtualMachineError> {
        self.step_hint(
            hint_executor,
            hint_locals,
            exec_scopes,
            hint_data_dictionary,
        )?;
        self.vm.borrow_mut().step_instruction()
    }

    fn should_run_py_hint(
        &self,
        hint_executor: &dyn HintProcessor,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
    ) -> Result<bool, VirtualMachineError> {
        let mut vm = self.vm.borrow_mut();
        match hint_executor.execute_hint(&mut vm, exec_scopes, hint_data) {
            Ok(()) => Ok(false),
            Err(VirtualMachineError::UnknownHint(_)) => Ok(true),
            Err(e) => Err(e),
        }
    }

    fn run_until_pc(
        &self,
        cairo_runner: &mut CairoRunner,
        address: &Relocatable,
        hint_locals: &mut HashMap<String, PyObject>,
    ) -> Result<(), VirtualMachineError> {
        let references = cairo_runner.get_reference_list();
        let hint_data_dictionary = cairo_runner.get_hint_data_dictionary(&references)?;

        while self.vm.borrow().get_pc() != address {
            self.step(
                cairo_runner.hint_executor,
                hint_locals,
                &mut cairo_runner.exec_scopes,
                &hint_data_dictionary,
            )?;
        }
        Ok(())
    }
}

pub(crate) fn add_scope_locals<'a>(
    globals: &PyDict,
    exec_scopes: &ExecutionScopes,
) -> Result<(), VirtualMachineError> {
    for (name, elem) in exec_scopes.get_local_variables()? {
        if let Some(pyobj) = elem.downcast_ref::<PyObject>() {
            globals.set_item(name, pyobj).map_err(to_vm_error)?;
        }
    }
    Ok(())
}

pub(crate) fn update_scope_hint_locals(
    exec_scopes: &mut ExecutionScopes,
    hint_locals: &mut HashMap<String, PyObject>,
    globals: &PyDict,
    py: Python,
) {
    for (name, elem) in globals {
        let name = name.to_string();
        if !GLOBAL_NAMES.contains(&&name.as_str()) {
            if hint_locals.keys().cloned().any(|x| x == name) {
                hint_locals.insert(name, elem.to_object(py));
            } else {
                exec_scopes.assign_or_update_variable(&name, any_box!(elem.to_object(py)));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::vm_core::PyVM;
    use cairo_rs::{
        bigint,
        hint_processor::{
            builtin_hint_processor::builtin_hint_processor_definition::{
                BuiltinHintProcessor, HintProcessorData,
            },
            hint_processor_definition::HintReference,
        },
        types::{
            exec_scope::ExecutionScopes,
            relocatable::{MaybeRelocatable, Relocatable},
        },
        vm::errors::{exec_scope_errors::ExecScopeError, vm_errors::VirtualMachineError},
    };
    use num_bigint::{BigInt, Sign};
    use pyo3::{PyObject, Python, ToPyObject};
    use std::collections::HashMap;

    #[test]
    fn execute_print_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        let code = "print(ap)";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(&hint_data, &mut HashMap::new(), &mut ExecutionScopes::new()),
            Ok(())
        );
    }

    #[test]
    fn set_memory_item_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        let code = "print(ap)";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(&hint_data, &mut HashMap::new(), &mut ExecutionScopes::new()),
            Ok(())
        );
    }

    #[test]
    fn ids_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        for _ in 0..2 {
            vm.vm.borrow_mut().add_memory_segment();
        }
        let references = HashMap::from([
            (String::from("a"), HintReference::new_simple(2)),
            (String::from("b"), HintReference::new_simple(1)),
        ]);
        vm.vm
            .borrow_mut()
            .insert_value(
                &Relocatable::from((1, 1)),
                &MaybeRelocatable::from(bigint!(2usize)),
            )
            .unwrap();
        let code = "ids.a = ids.b";
        let hint_data = HintProcessorData::new_default(code.to_string(), references);
        assert_eq!(
            vm.execute_hint(&hint_data, &mut HashMap::new(), &mut ExecutionScopes::new()),
            Ok(())
        );
        assert_eq!(
            vm.vm.borrow().get_maybe(&Relocatable::from((1, 2))),
            Ok(Some(MaybeRelocatable::from(bigint!(2))))
        );
    }

    #[test]
    // This test is analogous to the `test_step_for_preset_memory` unit test in the cairo-rs crate.
    fn test_step_with_no_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );

        for _ in 0..2 {
            vm.vm.borrow_mut().add_memory_segment();
        }

        let hint_processor = BuiltinHintProcessor::new_empty();

        vm.vm.borrow_mut().set_pc(Relocatable::from((0, 0)));
        vm.vm.borrow_mut().set_ap(2);
        vm.vm.borrow_mut().set_fp(2);

        vm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((0, 0)), bigint!(2345108766317314046_u64))
            .unwrap();
        vm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((1, 0)), &Relocatable::from((2, 0)))
            .unwrap();
        vm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((1, 1)), &Relocatable::from((3, 0)))
            .unwrap();

        assert_eq!(
            vm.step(
                &hint_processor,
                &mut HashMap::new(),
                &mut ExecutionScopes::new(),
                &HashMap::new()
            ),
            Ok(())
        );
    }

    #[test]
    fn test_step_with_print_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );

        for _ in 0..2 {
            vm.vm.borrow_mut().add_memory_segment();
        }

        let hint_processor = BuiltinHintProcessor::new_empty();

        vm.vm.borrow_mut().set_pc(Relocatable::from((0, 0)));
        vm.vm.borrow_mut().set_ap(2);
        vm.vm.borrow_mut().set_fp(2);

        vm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((0, 0)), bigint!(2345108766317314046_u64))
            .unwrap();
        vm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((1, 0)), &Relocatable::from((2, 0)))
            .unwrap();
        vm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((1, 1)), &Relocatable::from((3, 0)))
            .unwrap();

        let code = "print(ap)";
        let hint_proc_data = HintProcessorData::new_default(code.to_string(), HashMap::new());

        let mut hint_data = HashMap::new();
        hint_data.insert(0, hint_proc_data);

        assert_eq!(
            vm.step(
                &hint_processor,
                &mut HashMap::new(),
                &mut ExecutionScopes::new(),
                &HashMap::new()
            ),
            Ok(())
        );
    }

    #[test]
    fn scopes_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        for _ in 0..2 {
            vm.vm.borrow_mut().add_memory_segment();
        }

        let mut exec_scopes = ExecutionScopes::new();
        let code_a = "num = 6";
        let code_b = "assert(num == 6)";
        let hint_data = HintProcessorData::new_default(code_a.to_string(), HashMap::new());

        assert_eq!(
            vm.execute_hint(&hint_data, &mut HashMap::new(), &mut exec_scopes),
            Ok(())
        );
        let hint_data = HintProcessorData::new_default(code_b.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(&hint_data, &mut HashMap::new(), &mut exec_scopes),
            Ok(())
        );
    }

    #[test]
    fn scopes_hint_modify() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        for _ in 0..2 {
            vm.vm.borrow_mut().add_memory_segment();
        }

        let mut exec_scopes = ExecutionScopes::new();
        let code_a = "num = 6";
        let code_b = "assert(num == 6)";
        let code_c = "num = num + 3";
        let code_d = "assert(num == 9)";
        let hint_data = HintProcessorData::new_default(code_a.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(&hint_data, &mut HashMap::new(), &mut exec_scopes),
            Ok(())
        );
        let hint_data = HintProcessorData::new_default(code_b.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(&hint_data, &mut HashMap::new(), &mut exec_scopes),
            Ok(())
        );
        let hint_data = HintProcessorData::new_default(code_c.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(&hint_data, &mut HashMap::new(), &mut exec_scopes),
            Ok(())
        );
        let hint_data = HintProcessorData::new_default(code_d.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(&hint_data, &mut HashMap::new(), &mut exec_scopes),
            Ok(())
        );
    }

    #[test]
    fn modify_hint_locals() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        let code = "word = word[::-1]
print(word)";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        let word = Python::with_gil(|py| -> PyObject { "fruity".to_string().to_object(py) });
        let mut hint_locals = HashMap::from([("word".to_string(), word)]);
        assert_eq!(
            vm.execute_hint(&hint_data, &mut hint_locals, &mut ExecutionScopes::new()),
            Ok(())
        );
        let word_res = Python::with_gil(|py| -> String {
            hint_locals
                .get("word")
                .unwrap()
                .extract::<String>(py)
                .unwrap()
        });
        assert_eq!(word_res, "ytiurf".to_string())
    }

    #[test]
    fn exit_main_scope_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        let mut exec_scopes = ExecutionScopes::new();
        let code = "vm_exit_scope()";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(&hint_data, &mut HashMap::new(), &mut exec_scopes),
            Err(VirtualMachineError::MainScopeError(
                ExecScopeError::ExitMainScopeError
            ))
        );
    }

    #[test]
    fn enter_scope_empty_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        let mut exec_scopes = ExecutionScopes::new();
        let code = "vm_enter_scope()";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(&hint_data, &mut HashMap::new(), &mut exec_scopes),
            Ok(())
        );
        assert_eq!(exec_scopes.data.len(), 2)
    }

    #[test]
    fn enter_exit_scope_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        let mut exec_scopes = ExecutionScopes::new();
        let code = "vm_enter_scope()
vm_exit_scope()";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(&hint_data, &mut HashMap::new(), &mut exec_scopes),
            Ok(())
        );
        assert_eq!(exec_scopes.data.len(), 1)
    }

    #[test]
    fn list_bug() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        let mut exec_scopes = ExecutionScopes::new();
        let code = "lista_a = [1,2,3]
lista_b = [lista_a[k] for k in range(2)]";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(&hint_data, &mut HashMap::new(), &mut exec_scopes),
            Ok(())
        );
    }

    #[test]
    fn enter_scope_non_empty_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        let mut exec_scopes = ExecutionScopes::new();
        let code_a = "vm_enter_scope({'n': 12})";
        let code_b = "assert(n == 12)";
        let hint_data = HintProcessorData::new_default(code_a.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(&hint_data, &mut HashMap::new(), &mut exec_scopes),
            Ok(())
        );
        let hint_data = HintProcessorData::new_default(code_b.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(&hint_data, &mut HashMap::new(), &mut exec_scopes),
            Ok(())
        );
        assert_eq!(exec_scopes.data.len(), 2);
        assert!(exec_scopes.data[0].is_empty());
    }

    #[test]
    fn access_relocatable_segment_index() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        let mut exec_scopes = ExecutionScopes::new();
        let code = "assert(ap.segment_index == 1)";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(&hint_data, &mut HashMap::new(), &mut exec_scopes),
            Ok(())
        );
    }
}
