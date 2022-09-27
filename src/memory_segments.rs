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
