//use cairo_rs::vm::vm_memory::memory::Memory;
use cairo_rs::vm::vm_memory::memory_segments::MemorySegmentManager;
use pyo3::prelude::*;
use std::{cell::RefCell, rc::Rc};

use crate::memory::PyMemory;

#[pyclass(name = "MemorySegmentManager", unsendable)]
pub struct PySegmentManager {
    pub segment_manager: Rc<RefCell<MemorySegmentManager>>,
}

#[pymethods]
impl PySegmentManager {
    #[new]
    fn new() -> PySegmentManager {
        let segments = MemorySegmentManager::new();

        PySegmentManager {
            segment_manager: Rc::new(RefCell::new(segments)),
        }
    }

    pub fn add(&self, pymemory: &PyMemory) -> PyResult<()> {
        self.segment_manager
            .borrow_mut()
            .add(&mut pymemory.memory.borrow_mut().memory_borrow_mut());
        Ok(())
    }
}
