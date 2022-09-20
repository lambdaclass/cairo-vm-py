//use cairo_rs::vm::vm_memory::memory::Memory;
use cairo_rs::{
    hint_processor::proxies::memory_proxy::{get_memory_proxy, MemoryProxy},
    vm::vm_memory::{memory::Memory, memory_segments::MemorySegmentManager},
};
use pyo3::prelude::*;
use std::{cell::RefCell, rc::Rc};

use crate::relocatable::PyRelocatable;

#[pyclass(name = "MemorySegmentManager", unsendable)]
pub struct PySegmentManager {
    pub segment_manager: Rc<RefCell<MemorySegmentManager>>,
    pub memory: Rc<RefCell<MemoryProxy>>,
}

#[pymethods]
impl PySegmentManager {
    #[new]
    fn new() -> PySegmentManager {
        PySegmentManager {
            segment_manager: Rc::new(RefCell::new(MemorySegmentManager::new())),
            memory: Rc::new(RefCell::new(get_memory_proxy(&Rc::new(RefCell::new(
                Memory::new(),
            ))))),
        }
    }

    pub fn add(&self) -> PyResult<PyRelocatable> {
        Ok(self
            .memory
            .borrow_mut()
            .add_segment(&mut self.segment_manager.borrow_mut())
            .into())
    }
}
