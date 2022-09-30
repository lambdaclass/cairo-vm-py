use crate::ids::PyIds;
use crate::pycell;
use crate::scope_manager::{PyEnterScope, PyExitScope};
use crate::{
    memory::PyMemory, memory_segments::PySegmentManager, relocatable::PyRelocatable,
    utils::to_vm_error,
};
use cairo_rs::any_box;
use cairo_rs::hint_processor::proxies::exec_scopes_proxy::ExecutionScopesProxy;
use cairo_rs::vm::vm_core::VirtualMachine;
use cairo_rs::{
    hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData,
    vm::errors::vm_errors::VirtualMachineError,
};
use num_bigint::BigInt;
use pyo3::PyCell;
use pyo3::{pyclass, pymethods, PyObject, ToPyObject};
use pyo3::{types::PyDict, Python};
use std::any::Any;
use std::{cell::RefCell, rc::Rc};

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
}

impl PyVM {
    pub(crate) fn get_vm(&self) -> Rc<RefCell<VirtualMachine>> {
        Rc::clone(&self.vm)
    }

    pub(crate) fn execute_hint(
        &self,
        hint_data: &HintProcessorData,
        exec_scopes: &mut ExecutionScopesProxy,
    ) -> Result<(), VirtualMachineError> {
        Python::with_gil(|py| -> Result<(), VirtualMachineError> {
            let memory = PyMemory::new(&self);
            let segments = PySegmentManager::new(&self);
            let ap = PyRelocatable::from(self.vm.borrow().get_ap());
            let fp = PyRelocatable::from(self.vm.borrow().get_fp());
            let ids = PyIds::new(&self, &hint_data.ids_data, &hint_data.ap_tracking);
            let enter_scope = PyEnterScope::new();
            let exit_scope = PyExitScope::new();

            let locals = get_scope_locals(exec_scopes, py)?;

            let globals = PyDict::new(py);

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
                .set_item("vm_enter_scope", pycell!(py, enter_scope))
                .map_err(to_vm_error)?;
            globals
                .set_item("vm_exit_scope", pycell!(py, exit_scope))
                .map_err(to_vm_error)?;

            py.run(&hint_data.code, Some(globals), Some(locals))
                .map_err(to_vm_error)?;

            update_scope_locals(exec_scopes, locals, py);

            globals
                .get_item("vm_enter_scope")
                .ok_or(VirtualMachineError::CustomHint(
                    "Unexpected Error".to_string(),
                ))?
                .extract::<PyEnterScope>()
                .map_err(to_vm_error)?
                .update_scopes(exec_scopes)?;
            globals
                .get_item("vm_exit_scope")
                .ok_or(VirtualMachineError::CustomHint(
                    "Unexpected Error".to_string(),
                ))?
                .extract::<PyExitScope>()
                .map_err(to_vm_error)?
                .update_scopes(exec_scopes)
        })?;

        Ok(())
    }

    pub(crate) fn step_hint(&self) -> Result<(), VirtualMachineError> {
        todo!()
    }

    pub(crate) fn step(&self) -> Result<(), VirtualMachineError> {
        todo!()
    }
}

pub(crate) fn get_scope_locals<'a>(
    exec_scopes: &ExecutionScopesProxy,
    py: Python<'a>,
) -> Result<&'a PyDict, VirtualMachineError> {
    let locals = PyDict::new(py);
    for (name, elem) in exec_scopes.get_local_variables()? {
        if let Some(pyobj) = elem.downcast_ref::<PyObject>() {
            locals.set_item(name, pyobj).map_err(to_vm_error)?;
        }
    }
    Ok(locals)
}

pub(crate) fn update_scope_locals(
    exec_scopes: &mut ExecutionScopesProxy,
    locals: &PyDict,
    py: Python,
) {
    for (name, elem) in locals {
        exec_scopes.assign_or_update_variable(&name.to_string(), any_box!(elem.to_object(py)));
    }
}

#[cfg(test)]
mod test {
    use crate::vm_core::PyVM;
    use cairo_rs::{
        bigint,
        hint_processor::{
            builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData,
            hint_processor_definition::HintReference,
            proxies::exec_scopes_proxy::get_exec_scopes_proxy,
        },
        types::{
            exec_scope::ExecutionScopes,
            relocatable::{MaybeRelocatable, Relocatable},
        },
        vm::errors::{exec_scope_errors::ExecScopeError, vm_errors::VirtualMachineError},
    };
    use num_bigint::{BigInt, Sign};
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
            vm.execute_hint(
                &hint_data,
                &mut get_exec_scopes_proxy(&mut ExecutionScopes::new())
            ),
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
            vm.execute_hint(
                &hint_data,
                &mut get_exec_scopes_proxy(&mut ExecutionScopes::new())
            ),
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
        vm.vm
            .borrow_mut()
            .memory
            .insert(
                &Relocatable::from((1, 1)),
                &MaybeRelocatable::from(bigint!(2)),
            )
            .unwrap();
        let code = "ids.a = ids.b";
        let mut hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        hint_data.ids_data = HashMap::from([
            (String::from("a"), HintReference::new_simple(2)),
            (String::from("b"), HintReference::new_simple(1)),
        ]);
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut get_exec_scopes_proxy(&mut ExecutionScopes::new())
            ),
            Ok(())
        );
        assert_eq!(
            vm.vm.borrow().memory.get(&Relocatable::from((1, 2))),
            Ok(Some(&MaybeRelocatable::from(bigint!(2))))
        );
    }

    #[test]
    fn scopes_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        let mut exec_scopes = ExecutionScopes::new();
        let exec_scopes_proxy = &mut get_exec_scopes_proxy(&mut exec_scopes);
        let code_a = "num = 6";
        let code_b = "assert(num == 6)";
        let hint_data = HintProcessorData::new_default(code_a.to_string(), HashMap::new());
        assert_eq!(vm.execute_hint(&hint_data, exec_scopes_proxy), Ok(()));
        let exec_scopes_proxy = &mut get_exec_scopes_proxy(&mut exec_scopes);
        let hint_data = HintProcessorData::new_default(code_b.to_string(), HashMap::new());
        assert_eq!(vm.execute_hint(&hint_data, exec_scopes_proxy), Ok(()));
    }

    #[test]
    fn exit_main_scope_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        let mut exec_scopes = ExecutionScopes::new();
        let exec_scopes_proxy = &mut get_exec_scopes_proxy(&mut exec_scopes);
        let code = "vm_exit_scope()";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(&hint_data, exec_scopes_proxy),
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
        let exec_scopes_proxy = &mut get_exec_scopes_proxy(&mut exec_scopes);
        let code = "vm_enter_scope()";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(vm.execute_hint(&hint_data, exec_scopes_proxy), Ok(()));
        assert_eq!(exec_scopes.data.len(), 2)
    }

    #[test]
    fn enter_exit_scope_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        let mut exec_scopes = ExecutionScopes::new();
        let exec_scopes_proxy = &mut get_exec_scopes_proxy(&mut exec_scopes);
        let code = "vm_enter_scope()
vm_exit_scope()";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(vm.execute_hint(&hint_data, exec_scopes_proxy), Ok(()));
        assert_eq!(exec_scopes.data.len(), 1)
    }

    #[test]
    fn enter_scope_non_empty_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        let mut exec_scopes = ExecutionScopes::new();
        let exec_scopes_proxy = &mut get_exec_scopes_proxy(&mut exec_scopes);
        let code_a = "vm_enter_scope({'n': 12})";
        let code_b = "assert(n == 12)";
        let hint_data = HintProcessorData::new_default(code_a.to_string(), HashMap::new());
        assert_eq!(vm.execute_hint(&hint_data, exec_scopes_proxy), Ok(()));
        let exec_scopes_proxy = &mut get_exec_scopes_proxy(&mut exec_scopes);
        let hint_data = HintProcessorData::new_default(code_b.to_string(), HashMap::new());
        assert_eq!(vm.execute_hint(&hint_data, exec_scopes_proxy), Ok(()));
        assert_eq!(exec_scopes.data.len(), 2);
        assert!(exec_scopes.data[0].is_empty());
    }
}
