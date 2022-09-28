use crate::{relocatable::PyRelocatable, vm_core::PyVM};
use cairo_rs::vm::vm_core::VirtualMachine;
use pyo3::prelude::*;
use std::{cell::RefCell, rc::Rc};

#[pyclass(name = "MemorySegmentManager", unsendable)]
pub struct PySegmentManager {
    vm: Rc<RefCell<VirtualMachine>>,
}

#[pymethods]
impl PySegmentManager {
    #[new]
    pub fn new(vm: &PyVM) -> PySegmentManager {
        PySegmentManager { vm: vm.get_vm() }
    }

    pub fn add(&self) -> PyResult<PyRelocatable> {
        Ok(self.vm.borrow_mut().add_memory_segment().into())
    }
}

#[cfg(test)]
mod test {
    use super::PySegmentManager;
    use crate::vm_core::PyVM;
    use num_bigint::{BigInt, Sign};

    #[test]
    fn add_segment_test() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        let segments = PySegmentManager::new(&vm);
        assert!(segments.add().is_ok());
    }
}
