use crate::any_box;
use crate::ecdsa::PySignature;
use crate::ids::PyIds;
use crate::pycell;
use crate::run_context::PyRunContext;
use crate::scope_manager::{PyEnterScope, PyExitScope};
use crate::to_felt_or_relocatable::ToFeltOrRelocatableFunc;
use crate::utils::to_py_error;
use crate::{
    memory::PyMemory, memory_segments::PySegmentManager, range_check::PyRangeCheck,
    relocatable::PyRelocatable,
};
use cairo_felt::Felt252;
use cairo_vm::hint_processor::hint_processor_definition::HintProcessor;
use cairo_vm::serde::deserialize_program::Member;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::vm_core::VirtualMachine;
use cairo_vm::{
    hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData,
    vm::errors::vm_errors::VirtualMachineError,
};
use lazy_static::lazy_static;
use num_bigint::BigUint;
use pyo3::{pyclass, pymethods, PyObject, ToPyObject};
use pyo3::{types::PyDict, Python};
use pyo3::{PyCell, PyErr};
use std::any::Any;
use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

const GLOBAL_NAMES: [&str; 18] = [
    "memory",
    "segments",
    "ap",
    "fp",
    "ids",
    "vm_enter_scope",
    "vm_exit_scope",
    "to_felt_or_relocatable",
    "range_check_builtin",
    "ecdsa_builtin",
    "PRIME",
    "__doc__",
    "__annotations__",
    "__package__",
    "__builtins__",
    "__spec__",
    "__loader__",
    "__name__",
];

lazy_static! {
    pub static ref CAIRO_PRIME: BigUint = Felt252::prime();
}

#[derive(Clone)]
#[pyclass(unsendable)]
pub struct PyVM {
    pub(crate) vm: Rc<RefCell<VirtualMachine>>,
    pub(crate) failed_hint_index: Option<usize>,
}

#[pymethods]
impl PyVM {
    #[getter]
    fn run_context(&self) -> PyRunContext {
        let vm = self.vm.borrow();
        PyRunContext::new(vm.get_pc(), vm.get_ap(), vm.get_fp())
    }
}

impl PyVM {
    pub fn new(trace_enabled: bool) -> PyVM {
        PyVM {
            vm: Rc::new(RefCell::new(VirtualMachine::new(trace_enabled))),
            failed_hint_index: None,
        }
    }

    pub(crate) fn get_vm(&self) -> Rc<RefCell<VirtualMachine>> {
        Rc::clone(&self.vm)
    }

    pub(crate) fn execute_hint(
        &mut self,
        hint_data: &HintProcessorData,
        hint_locals: &mut HashMap<String, PyObject>,
        exec_scopes: &mut ExecutionScopes,
        constants: &HashMap<String, Felt252>,
        struct_types: Rc<HashMap<String, HashMap<String, Member>>>,
        static_locals: Option<&HashMap<String, PyObject>>,
    ) -> Result<(), PyErr> {
        Python::with_gil(|py| -> Result<(), PyErr> {
            let memory = PyMemory::new(self);
            let segments = PySegmentManager::new(self, memory.clone());
            let ap = PyRelocatable::from((*self.vm).borrow().get_ap());
            let fp = PyRelocatable::from((*self.vm).borrow().get_fp());
            let ids = PyIds::new(
                self,
                &hint_data.ids_data,
                &hint_data.ap_tracking,
                constants,
                struct_types,
            );
            let enter_scope = pycell!(py, PyEnterScope::new());
            let exit_scope = pycell!(py, PyExitScope::new());
            let range_check_builtin =
                PyRangeCheck::from((*self.vm).borrow().get_range_check_builtin());
            let ecdsa_builtin = pycell!(py, PySignature::new());
            let prime: BigUint = CAIRO_PRIME.clone();
            let to_felt_or_relocatable = ToFeltOrRelocatableFunc;

            // This line imports Python builtins. If not imported, this will run only with Python 3.10
            let globals = py.import("__main__")?.dict().copy()?;

            add_scope_locals(globals, exec_scopes)?;

            globals.set_item("memory", pycell!(py, memory))?;
            globals.set_item("segments", pycell!(py, segments))?;
            globals.set_item("ap", pycell!(py, ap))?;
            globals.set_item("fp", pycell!(py, fp))?;
            globals.set_item("ids", pycell!(py, ids))?;
            globals.set_item("vm_enter_scope", enter_scope)?;
            globals.set_item("vm_exit_scope", exit_scope)?;
            globals.set_item("range_check_builtin", range_check_builtin)?;
            globals.set_item("ecdsa_builtin", ecdsa_builtin)?;
            globals.set_item("PRIME", prime)?;
            globals.set_item(
                "to_felt_or_relocatable",
                pycell!(py, to_felt_or_relocatable),
            )?;

            for (name, pyobj) in hint_locals.iter() {
                globals.set_item(name, pyobj)?;
            }

            if let Some(static_locals) = static_locals {
                for (name, pyobj) in static_locals.iter() {
                    globals.set_item(name, pyobj)?;
                }
            }

            py.run(&hint_data.code, Some(globals), None)?;

            update_scope_hint_locals(exec_scopes, hint_locals, static_locals, globals, py);

            if self.vm.borrow_mut().get_signature_builtin().is_ok() {
                ecdsa_builtin
                    .borrow()
                    .update_signature(
                        self.vm
                            .borrow_mut()
                            .get_signature_builtin()
                            .map_err(to_py_error)?,
                    )
                    .map_err(to_py_error)?;
            }
            enter_scope.borrow().update_scopes(exec_scopes)?;
            exit_scope.borrow().update_scopes(exec_scopes)
        })?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn step_hint(
        &mut self,
        hint_executor: &mut dyn HintProcessor,
        hint_locals: &mut HashMap<String, PyObject>,
        exec_scopes: &mut ExecutionScopes,
        hint_data_dictionary: &HashMap<usize, Vec<Box<dyn Any>>>,
        struct_types: Rc<HashMap<String, HashMap<String, Member>>>,
        constants: &HashMap<String, Felt252>,
        static_locals: Option<&HashMap<String, PyObject>>,
    ) -> Result<(), PyErr> {
        let pc_offset = (*self.vm).borrow().get_pc().offset;

        if let Some(hint_list) = hint_data_dictionary.get(&pc_offset) {
            for (hint_index, hint_data) in hint_list.iter().enumerate() {
                if self
                    .should_run_py_hint(
                        hint_executor,
                        exec_scopes,
                        hint_data,
                        constants,
                        hint_index,
                    )
                    .map_err(to_py_error)?
                {
                    let hint_data = hint_data
                        .downcast_ref::<HintProcessorData>()
                        .ok_or_else(|| {
                            VirtualMachineError::Hint(
                                hint_index,
                                Box::new(HintError::WrongHintData),
                            )
                        })
                        .map_err(to_py_error)?;

                    if let Err(hint_error) = self.execute_hint(
                        hint_data,
                        hint_locals,
                        exec_scopes,
                        constants,
                        Rc::clone(&struct_types),
                        static_locals,
                    ) {
                        self.failed_hint_index = Some(hint_index);
                        return Err(hint_error);
                    }
                }
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn step(
        &mut self,
        hint_executor: &mut dyn HintProcessor,
        hint_locals: &mut HashMap<String, PyObject>,
        exec_scopes: &mut ExecutionScopes,
        hint_data_dictionary: &HashMap<usize, Vec<Box<dyn Any>>>,
        struct_types: Rc<HashMap<String, HashMap<String, Member>>>,
        constants: &HashMap<String, Felt252>,
        static_locals: Option<&HashMap<String, PyObject>>,
    ) -> Result<(), PyErr> {
        self.step_hint(
            hint_executor,
            hint_locals,
            exec_scopes,
            hint_data_dictionary,
            struct_types,
            constants,
            static_locals,
        )?;
        self.vm.borrow_mut().step_instruction().map_err(to_py_error)
    }

    fn should_run_py_hint(
        &mut self,
        hint_executor: &mut dyn HintProcessor,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
        hint_index: usize,
    ) -> Result<bool, VirtualMachineError> {
        let mut vm = self.vm.borrow_mut();
        match hint_executor.execute_hint(&mut vm, exec_scopes, hint_data, constants) {
            Ok(()) => Ok(false),
            Err(HintError::UnknownHint(_)) => Ok(true),
            Err(e) => {
                self.failed_hint_index = Some(hint_index);
                Err(VirtualMachineError::Hint(hint_index, Box::new(e)))
            }
        }
    }
}

pub(crate) fn add_scope_locals(
    globals: &PyDict,
    exec_scopes: &ExecutionScopes,
) -> Result<(), PyErr> {
    for (name, elem) in exec_scopes.get_local_variables().map_err(to_py_error)? {
        if let Some(pyobj) = elem.downcast_ref::<PyObject>() {
            globals.set_item(name, pyobj)?;
        }
    }
    Ok(())
}

pub(crate) fn update_scope_hint_locals(
    exec_scopes: &mut ExecutionScopes,
    hint_locals: &mut HashMap<String, PyObject>,
    static_locals: Option<&HashMap<String, PyObject>>,
    globals: &PyDict,
    py: Python,
) {
    let static_local_names = static_locals
        .map(|locals| locals.keys().collect::<Vec<&String>>())
        .unwrap_or_default();
    for (name, elem) in globals {
        let name = name.to_string();
        if !GLOBAL_NAMES.contains(&name.as_str()) && !static_local_names.contains(&&name) {
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
    use super::*;
    use crate::{biguint, relocatable::PyMaybeRelocatable, vm_core::PyVM};
    use cairo_felt::Felt252;
    use cairo_vm::{
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
    };
    use pyo3::{PyObject, Python, ToPyObject};
    use std::{any::Any, collections::HashMap, rc::Rc};

    #[test]
    fn execute_print_hint() {
        let mut vm = PyVM::new(false);
        let code = "print(ap)";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut ExecutionScopes::new(),
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
    }

    #[test]
    fn set_memory_item_hint() {
        let mut vm = PyVM::new(false);
        let code = "print(ap)";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut ExecutionScopes::new(),
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
    }

    #[test]
    fn ids_hint() {
        let mut vm = PyVM::new(false);
        for _ in 0..2 {
            vm.vm.borrow_mut().add_memory_segment();
        }
        let references = HashMap::from([
            (String::from("a"), HintReference::new_simple(2)),
            (String::from("b"), HintReference::new_simple(1)),
        ]);
        vm.vm
            .borrow_mut()
            .insert_value(Relocatable::from((1, 1)), &MaybeRelocatable::from(2))
            .unwrap();
        let code = "ids.a = ids.b";
        let hint_data = HintProcessorData::new_default(code.to_string(), references);
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut ExecutionScopes::new(),
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
        assert_eq!(
            vm.vm.borrow().get_maybe(&Relocatable::from((1, 2))),
            Some(MaybeRelocatable::from(2))
        );
    }

    #[test]
    // Test the availability of cairo constants in ids
    fn const_ids() {
        let mut vm = PyVM::new(false);

        let constants = HashMap::from([(String::from("CONST"), Felt252::new(1))]);

        let mut exec_scopes = ExecutionScopes::new();
        let code_1 = "assert(ids.CONST != 2)";
        let hint_data = HintProcessorData::new_default(code_1.to_string(), HashMap::new());

        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &constants,
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());

        let code_2 = "assert(ids.CONST == 1)";
        let hint_data = HintProcessorData::new_default(code_2.to_string(), HashMap::new());

        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &constants,
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
    }

    #[test]
    // This test is analogous to the `test_step_for_preset_memory` unit test in the cairo-rs crate.
    fn test_step_with_no_hint() {
        let mut vm = PyVM::new(false);

        for _ in 0..2 {
            vm.vm.borrow_mut().add_memory_segment();
        }

        let mut hint_processor = BuiltinHintProcessor::new_empty();

        vm.vm.borrow_mut().set_pc(Relocatable::from((0, 0)));
        vm.vm.borrow_mut().set_ap(2);
        vm.vm.borrow_mut().set_fp(2);

        vm.vm
            .borrow_mut()
            .insert_value(Relocatable::from((0, 0)), 2345108766317314046)
            .unwrap();
        vm.vm
            .borrow_mut()
            .insert_value(Relocatable::from((1, 0)), Relocatable::from((2, 0)))
            .unwrap();
        vm.vm
            .borrow_mut()
            .insert_value(Relocatable::from((1, 1)), Relocatable::from((3, 0)))
            .unwrap();

        assert!(vm
            .step(
                &mut hint_processor,
                &mut HashMap::new(),
                &mut ExecutionScopes::new(),
                &HashMap::new(),
                Rc::new(HashMap::new()),
                &HashMap::new(),
                None,
            )
            .is_ok());
    }

    #[test]
    fn test_step_with_print_hint() {
        let mut vm = PyVM::new(false);

        for _ in 0..2 {
            vm.vm.borrow_mut().add_memory_segment();
        }

        let mut hint_processor = BuiltinHintProcessor::new_empty();

        vm.vm.borrow_mut().set_pc(Relocatable::from((0, 0)));
        vm.vm.borrow_mut().set_ap(2);
        vm.vm.borrow_mut().set_fp(2);

        vm.vm
            .borrow_mut()
            .insert_value(Relocatable::from((0, 0)), 2345108766317314046)
            .unwrap();
        vm.vm
            .borrow_mut()
            .insert_value(Relocatable::from((1, 0)), Relocatable::from((2, 0)))
            .unwrap();
        vm.vm
            .borrow_mut()
            .insert_value(Relocatable::from((1, 1)), Relocatable::from((3, 0)))
            .unwrap();

        let code = "print(ap)";
        let hint_proc_data = HintProcessorData::new_default(code.to_string(), HashMap::new());

        let mut hint_data = HashMap::new();
        hint_data.insert(0, hint_proc_data);

        assert!(vm
            .step(
                &mut hint_processor,
                &mut HashMap::new(),
                &mut ExecutionScopes::new(),
                &HashMap::new(),
                Rc::new(HashMap::new()),
                &HashMap::new(),
                None,
            )
            .is_ok());
    }

    #[test]
    fn scopes_hint() {
        let mut vm = PyVM::new(false);
        for _ in 0..2 {
            vm.vm.borrow_mut().add_memory_segment();
        }

        let mut exec_scopes = ExecutionScopes::new();
        let code_a = "num = 6";
        let code_b = "assert(num == 6)";
        let hint_data = HintProcessorData::new_default(code_a.to_string(), HashMap::new());

        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
        let hint_data = HintProcessorData::new_default(code_b.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
    }

    #[test]
    fn scopes_hint_modify() {
        let mut vm = PyVM::new(false);
        for _ in 0..2 {
            vm.vm.borrow_mut().add_memory_segment();
        }

        let mut exec_scopes = ExecutionScopes::new();
        let code_a = "num = 6";
        let code_b = "assert(num == 6)";
        let code_c = "num = num + 3";
        let code_d = "assert(num == 9)";
        let hint_data = HintProcessorData::new_default(code_a.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
        let hint_data = HintProcessorData::new_default(code_b.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
        let hint_data = HintProcessorData::new_default(code_c.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
        let hint_data = HintProcessorData::new_default(code_d.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
    }

    #[test]
    fn modify_hint_locals() {
        let mut vm = PyVM::new(false);
        let code = "word = word[::-1]
print(word)";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        let word = Python::with_gil(|py| -> PyObject { "fruity".to_string().to_object(py) });
        let mut hint_locals = HashMap::from([("word".to_string(), word)]);
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut hint_locals,
                &mut ExecutionScopes::new(),
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
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
        let mut vm = PyVM::new(false);
        let mut exec_scopes = ExecutionScopes::new();
        let code = "vm_exit_scope()";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        let x = vm.execute_hint(
            &hint_data,
            &mut HashMap::new(),
            &mut exec_scopes,
            &HashMap::new(),
            Rc::new(HashMap::new()),
            None,
        );
        assert!(x.is_err());
    }

    #[test]
    fn enter_scope_empty_hint() {
        let mut vm = PyVM::new(false);
        let mut exec_scopes = ExecutionScopes::new();
        let code = "vm_enter_scope()";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
        assert_eq!(exec_scopes.data.len(), 2)
    }

    #[test]
    fn enter_exit_scope_same_hint() {
        let mut vm = PyVM::new(false);
        let mut exec_scopes = ExecutionScopes::new();
        let code = "vm_enter_scope()
vm_exit_scope()";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
        assert_eq!(exec_scopes.data.len(), 1);
    }

    #[test]
    fn enter_exit_scope_separate_hints() {
        let mut vm = PyVM::new(false);
        let mut exec_scopes = ExecutionScopes::new();
        let code_a = "vm_enter_scope()";
        let code_b = "vm_exit_scope()";
        let hint_data = HintProcessorData::new_default(code_a.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
        assert_eq!(exec_scopes.data.len(), 2);
        let hint_data = HintProcessorData::new_default(code_b.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
        assert_eq!(exec_scopes.data.len(), 1)
    }

    #[test]
    fn enter_exit_enter_scope_same_hint() {
        let mut vm = PyVM::new(false);
        let mut exec_scopes = ExecutionScopes::new();
        let code = "vm_enter_scope()
vm_exit_scope()
vm_enter_scope()";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
        assert_eq!(exec_scopes.data.len(), 2)
    }

    #[test]
    fn list_comprehension() {
        let mut vm = PyVM::new(false);
        let mut exec_scopes = ExecutionScopes::new();
        let code = "lista_a = [1,2,3]
lista_b = [lista_a[k] for k in range(2)]";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
    }

    #[test]
    fn enter_scope_non_empty_hint() {
        let mut vm = PyVM::new(false);
        let mut exec_scopes = ExecutionScopes::new();
        let code_a = "vm_enter_scope({'n': 12})";
        let code_b = "assert(n == 12)";
        let hint_data = HintProcessorData::new_default(code_a.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
        let hint_data = HintProcessorData::new_default(code_b.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
        assert_eq!(exec_scopes.data.len(), 2);
        assert!(exec_scopes.data[0].is_empty());
    }

    #[test]
    fn access_relocatable_segment_index() {
        let mut vm = PyVM::new(false);
        let mut exec_scopes = ExecutionScopes::new();
        let code = "assert(ap.segment_index == 1)";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
    }

    #[test]
    fn to_felt_or_relocatable_number() {
        let mut vm = PyVM::new(false);
        let mut exec_scopes = ExecutionScopes::new();
        let code = "felt = to_felt_or_relocatable(456)";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
        Python::with_gil(|py| {
            assert_eq!(
                exec_scopes
                    .get::<PyObject>("felt")
                    .unwrap()
                    .extract::<PyMaybeRelocatable>(py)
                    .unwrap(),
                PyMaybeRelocatable::from(biguint!(456_u32))
            );
        });
    }

    #[test]
    fn to_felt_or_relocatable_list_should_fail() {
        let mut vm = PyVM::new(false);
        let mut exec_scopes = ExecutionScopes::new();
        let code = "felt = to_felt_or_relocatable([1,2,3])";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_err());
    }

    #[test]
    fn to_felt_or_relocatable_relocatable() {
        let mut vm = PyVM::new(false);
        let mut exec_scopes = ExecutionScopes::new();
        let code = "ids.test_value = to_felt_or_relocatable(ids.relocatable)";
        vm.vm.borrow_mut().add_memory_segment();
        vm.vm.borrow_mut().add_memory_segment();
        //insert ids.relocatable
        vm.vm
            .borrow_mut()
            .insert_value(Relocatable::from((1, 0)), Relocatable::from((2, 0)))
            .unwrap();
        let ids = HashMap::from([
            ("relocatable".to_string(), HintReference::new_simple(0)),
            ("test_value".to_string(), HintReference::new_simple(1)),
        ]);
        let hint_data = HintProcessorData::new_default(code.to_string(), ids);
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok());
        //Check the value of ids.test_value
        assert_eq!(
            vm.vm
                .borrow()
                .get_relocatable(Relocatable::from((1, 1)))
                .unwrap(),
            Relocatable::from((2, 0))
        );
    }

    #[test]
    fn test_get_range() {
        let mut pyvm = PyVM::new(false);
        let mut exec_scopes = ExecutionScopes::new();
        let code = "assert(memory.get_range(ids.address, 3) == [1,2,7])";

        let ids = HashMap::from([("address".to_string(), HintReference::new_simple(0))]);

        for _ in 0..3 {
            pyvm.vm.borrow_mut().add_memory_segment();
        }

        pyvm.vm
            .borrow_mut()
            .insert_value(Relocatable::from((1, 0)), Relocatable::from((2, 0)))
            .unwrap();

        pyvm.vm
            .borrow_mut()
            .insert_value(Relocatable::from((2, 0)), 1)
            .unwrap();

        pyvm.vm
            .borrow_mut()
            .insert_value(Relocatable::from((2, 1)), 2)
            .unwrap();

        pyvm.vm
            .borrow_mut()
            .insert_value(Relocatable::from((2, 2)), 7)
            .unwrap();

        let hint_data = HintProcessorData::new_default(code.to_string(), ids);
        assert!(pyvm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok())
    }

    #[test]
    fn test_segments_memory_get_range() {
        let mut pyvm = PyVM::new(false);
        let code = "assert(segments.memory.get_range(ids.address, 2) == [9,12])";

        let ids = HashMap::from([("address".to_string(), HintReference::new_simple(0))]);

        for _ in 0..3 {
            pyvm.vm.borrow_mut().add_memory_segment();
        }

        pyvm.vm
            .borrow_mut()
            .insert_value(Relocatable::from((1, 0)), Relocatable::from((2, 0)))
            .unwrap();

        pyvm.vm
            .borrow_mut()
            .insert_value(Relocatable::from((2, 0)), 9)
            .unwrap();

        pyvm.vm
            .borrow_mut()
            .insert_value(Relocatable::from((2, 1)), 12)
            .unwrap();

        let hint_data = HintProcessorData::new_default(code.to_string(), ids);
        assert!(pyvm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut ExecutionScopes::new(),
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_ok())
    }

    #[test]
    fn run_hint_with_static_locals() {
        let mut vm = PyVM::new(false);
        let static_locals = HashMap::from([(
            "__number_max".to_string(),
            Python::with_gil(|py| -> PyObject { 90.to_object(py) }),
        )]);
        let code = "number = __number_max";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        let mut exec_scopes = ExecutionScopes::new();
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                Some(&static_locals),
            )
            .is_ok());
        let number = Python::with_gil(|py| -> usize {
            exec_scopes.data[0]
                .get("number")
                .unwrap()
                .downcast_ref::<PyObject>()
                .unwrap()
                .extract::<usize>(py)
                .unwrap()
        });
        assert_eq!(number, 90)
    }

    #[test]
    fn run_hint_with_static_locals_shouldnt_change_its_value() {
        let mut vm = PyVM::new(false);
        let static_locals = HashMap::from([(
            "__number_max".to_string(),
            Python::with_gil(|py| -> PyObject { 90.to_object(py) }),
        )]);
        let code = "__number_max = 15";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        let mut exec_scopes = ExecutionScopes::new();
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                Some(&static_locals),
            )
            .is_ok());
        let number = Python::with_gil(|py| -> usize {
            static_locals
                .get("__number_max")
                .unwrap()
                .extract::<usize>(py)
                .unwrap()
        });
        assert_eq!(number, 90)
    }

    #[test]
    fn run_hint_with_static_locals_shouldnt_affect_scope_or_hint_locals() {
        let mut vm = PyVM::new(false);
        let static_locals = HashMap::from([(
            "__number_max".to_string(),
            Python::with_gil(|py| -> PyObject { 90.to_object(py) }),
        )]);
        let code = "assert(__number_max == 90)";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        let mut exec_scopes = ExecutionScopes::new();
        let mut hint_locals = HashMap::new();
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut hint_locals,
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                Some(&static_locals),
            )
            .is_ok());
        assert!(exec_scopes.data[0].is_empty());
        assert!(hint_locals.is_empty())
    }

    #[test]
    fn should_run_py_hint_nonsense_data_should_fail() {
        let mut vm = PyVM::new(false);
        let hint_data: Box<dyn Any + 'static> = Box::new("nonsense");
        let mut hint_processor = BuiltinHintProcessor::new_empty();
        assert!(vm
            .should_run_py_hint(
                &mut hint_processor,
                &mut ExecutionScopes::new(),
                &hint_data,
                &HashMap::new(),
                0
            )
            .is_err());
    }

    #[test]
    fn run_context() {
        let vm = PyVM::new(false);

        let run_context = vm.run_context();
        assert_eq!(run_context.pc(), (0, 0).into());
        assert_eq!(run_context.ap(), (1, 0).into());
        assert_eq!(run_context.fp(), (1, 0).into());
    }
}
