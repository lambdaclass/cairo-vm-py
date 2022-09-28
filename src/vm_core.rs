use crate::ids::PyIds;
use crate::pycell;
use crate::{
    memory::PyMemory, memory_segments::PySegmentManager, relocatable::PyRelocatable,
    utils::to_vm_error,
};
use cairo_rs::any_box;
use cairo_rs::hint_processor::hint_processor_definition::HintReference;
use cairo_rs::hint_processor::proxies::exec_scopes_proxy::ExecutionScopesProxy;
use cairo_rs::serde::deserialize_program::ApTracking;
use cairo_rs::vm::vm_core::VirtualMachine;
use cairo_rs::{
    hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData,
    vm::errors::vm_errors::VirtualMachineError,
};
use num_bigint::BigInt;
use pyo3::{pyclass, pymethods, PyObject, ToPyObject};
use pyo3::{types::PyDict, Python};
use pyo3::{PyAny, PyCell};
use std::any::Any;
use std::collections::HashMap;
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
            vm: Rc::new(RefCell::new(VirtualMachine::new(
                prime,
                Vec::new(),
                trace_enabled,
            ))),
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
        references: &HashMap<String, HintReference>,
        ap_tracking: &ApTracking,
        exec_scopes: &mut ExecutionScopesProxy,
    ) -> Result<(), VirtualMachineError> {
        Python::with_gil(|py| -> Result<(), VirtualMachineError> {
            let memory = PyMemory::new(&self);
            let segments = PySegmentManager::new(&self);
            let ap = PyRelocatable::from(self.vm.borrow().get_ap());
            let fp = PyRelocatable::from(self.vm.borrow().get_fp());
            let ids = PyIds::new(&self, references, ap_tracking);

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

            py.run(&hint_data.code, Some(globals), Some(locals))
                .map_err(to_vm_error)?;

            update_scope_locals(exec_scopes, locals, py);

            Ok(())
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
            proxies::exec_scopes_proxy::{get_exec_scopes_proxy, ExecutionScopesProxy},
        },
        serde::deserialize_program::ApTracking,
        types::{
            exec_scope::ExecutionScopes,
            relocatable::{MaybeRelocatable, Relocatable},
        },
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
                &HashMap::new(),
                &ApTracking::default(),
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
                &HashMap::new(),
                &ApTracking::default(),
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
        let references = HashMap::from([
            (String::from("a"), HintReference::new_simple(2)),
            (String::from("b"), HintReference::new_simple(1)),
        ]);
        vm.vm
            .borrow_mut()
            .memory
            .insert(
                &Relocatable::from((1, 1)),
                &MaybeRelocatable::from(bigint!(2)),
            )
            .unwrap();
        let code = "ids.a = ids.b";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &references,
                &ApTracking::default(),
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
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &HashMap::new(),
                &ApTracking::default(),
                exec_scopes_proxy
            ),
            Ok(())
        );
        let hint_data = HintProcessorData::new_default(code_b.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &HashMap::new(),
                &ApTracking::default(),
                exec_scopes_proxy
            ),
            Ok(())
        );
    }
}
