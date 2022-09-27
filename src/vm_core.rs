use cairo_rs::vm::{runners::builtin_runner::BuiltinRunner, vm_core::VirtualMachine};
use num_bigint::BigInt;
use pyo3::{pyclass, pymethods};
use std::{cell::RefCell, rc::Rc};
use crate::{utils::to_vm_error, relocatable::PyRelocatable, memory::PyMemory, memory_segments::PySegmentManager};
use cairo_rs::{hint_processor::{builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData, proxies::{vm_proxy::VMProxy, exec_scopes_proxy::ExecutionScopesProxy}}, vm::errors::vm_errors::VirtualMachineError};
use pyo3::{Python, types::PyDict};
use pyo3::PyCell;
use crate::pycell;

#[pyclass(unsendable)]
pub struct PyVM {
    vm: Rc<RefCell<VirtualMachine>>,
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
    fn execute_hint(
        &self,
        hint_data: &HintProcessorData,
    ) -> Result<(), VirtualMachineError> {
        Python::with_gil(|py| -> Result<(), VirtualMachineError> {
                let memory = PyMemory::new(&self);
                let segments = PySegmentManager::new(&self);
                let globals = PyDict::new(py);
                let ap =  PyRelocatable::new((1, self.vm.borrow().run_context.ap));
                let fp =  PyRelocatable::new((1, self.vm.borrow().run_context.fp));

                globals.set_item("memory", pycell!(py, memory)).map_err(to_vm_error)?;
                globals.set_item("segments", pycell!(py, segments)).map_err(to_vm_error)?;
                globals.set_item("ap", pycell!(py, ap)).map_err(to_vm_error)?;
                globals.set_item("fp", pycell!(py, fp)).map_err(to_vm_error)?;

                py.run(&hint_data.code, Some(globals), None).map_err(to_vm_error)?;
                Ok(())
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use cairo_rs::{hint_processor::{builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData}};
    use num_bigint::{BigInt, Sign};
    use crate::vm_core::PyVM;

    #[test]
    fn execute_hint() {
        let mut vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        // let code = "print(ap)";
        let code = r#"print("hello")"#;
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(vm.execute_hint(&hint_data), Ok(()));
    }
}

