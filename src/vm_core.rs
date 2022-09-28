use crate::pycell;
use crate::{
    memory::PyMemory, memory_segments::PySegmentManager, relocatable::PyRelocatable,
    utils::to_vm_error,
};
use cairo_rs::vm::vm_core::VirtualMachine;
use cairo_rs::{
    hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData,
    vm::errors::vm_errors::VirtualMachineError,
};
use num_bigint::BigInt;
use pyo3::PyCell;
use pyo3::{pyclass, pymethods};
use pyo3::{types::PyDict, Python};
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
    ) -> Result<(), VirtualMachineError> {
        Python::with_gil(|py| -> Result<(), VirtualMachineError> {
            let memory = PyMemory::new(&self);
            let segments = PySegmentManager::new(&self);
            let ap = PyRelocatable::from(self.vm.borrow().get_ap());
            let fp = PyRelocatable::from(self.vm.borrow().get_fp());

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

            py.run(&hint_data.code, Some(globals), None)
                .map_err(to_vm_error)?;

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

#[cfg(test)]
mod test {
    use crate::vm_core::PyVM;
    use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData;
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
        assert_eq!(vm.execute_hint(&hint_data), Ok(()));
    }

    #[test]
    fn set_memory_item_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        let code = "print(ap)";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(vm.execute_hint(&hint_data), Ok(()));
    }
}
