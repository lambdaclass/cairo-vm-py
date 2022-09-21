mod memory;
mod memory_segments;
mod pybigint;
mod relocatable;
pub mod execute_hint;
pub mod utils;

use std::cell::RefCell;
use std::rc::Rc;

use crate::memory::PyMemory;
use crate::memory_segments::PySegmentManager;
use crate::relocatable::PyRelocatable;
use cairo_rs::vm::vm_memory::memory_segments::MemorySegmentManager;
use pyo3::prelude::*;

#[pyfunction]
fn generate_memory_and_segments() -> PyResult<(PyMemory, PySegmentManager)> {
    let py_memory = PyMemory::new();
    let py_segment_manager = PySegmentManager {
        memory: py_memory.memory.clone(),
        segment_manager: Rc::new(RefCell::new(MemorySegmentManager::new())),
    };
    Ok((py_memory, py_segment_manager))
}

#[pymodule]
fn cairo_rs_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyMemory>()?;
    m.add_class::<PyRelocatable>()?;
    m.add_class::<PySegmentManager>()?;
    m.add_function(wrap_pyfunction!(generate_memory_and_segments, m)?)?;
    Ok(())
}
