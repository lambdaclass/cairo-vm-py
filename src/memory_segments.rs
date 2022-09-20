//use cairo_rs::vm::vm_memory::memory::Memory;
use cairo_rs::{
    hint_processor::proxies::memory_proxy::{get_memory_proxy, MemoryProxy},
    types::relocatable::Relocatable,
    vm::vm_memory::{memory::Memory, memory_segments::MemorySegmentManager},
};
use lazy_static::lazy_static;
use num_bigint::BigInt;
use pyo3::prelude::*;
use std::{cell::RefCell, rc::Rc};

use crate::relocatable::{PyMaybeRelocatable, PyRelocatable};

lazy_static! {
    pub static ref VM_PRIME: BigInt = BigInt::parse_bytes(
        b"3618502788666131213697322783095070105623107215331596699973092056135872020481",
        10,
    )
    .unwrap();
}
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

    pub fn write_arg(&self, ptr: PyRelocatable, arg: BigInt, py: Python) -> PyResult<PyObject> {
        let result = self.memory.borrow_mut().write_arg(
            &mut self.segment_manager.borrow_mut(),
            &Relocatable::from(ptr),
            &arg,
            Some(&VM_PRIME),
        );
        PyMaybeRelocatable::maybe_relocatable_result_to_py(result, py)
    }
}
