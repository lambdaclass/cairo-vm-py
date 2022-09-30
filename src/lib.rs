mod ids;
mod memory;
mod memory_segments;
mod relocatable;
mod scope_manager;
mod utils;
mod vm_core;

use pyo3::prelude::*;
use vm_core::PyVM;

#[pymodule]
fn cairo_rs_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyVM>()?;
    Ok(())
}
