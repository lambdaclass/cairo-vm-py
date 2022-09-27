mod memory;
mod memory_segments;
mod relocatable;
mod vm_core;
mod utils;

use pyo3::prelude::*;
use vm_core::PyVM;

#[pymodule]
fn cairo_rs_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyVM>()?;
    Ok(())
}
