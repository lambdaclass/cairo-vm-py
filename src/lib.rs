pub mod cairo_run;
pub mod ids;
mod memory;
mod memory_segments;
mod range_check;
mod relocatable;
mod scope_manager;
mod utils;
mod vm_core;
mod run_resource;

use pyo3::prelude::*;
use vm_core::PyVM;
use run_resource::RunResource;

#[pymodule]
fn cairo_rs_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyVM>()?;
    m.add_class::<RunResource>()?;
    Ok(())
}
