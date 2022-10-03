mod ids;
pub mod cairo_run;
mod memory;
mod memory_segments;
mod relocatable;
mod scope_manager;
mod utils;
mod vm_core;

use pyo3::prelude::*;
use vm_core::PyVM;
use cairo_run::cairo_run_py;

#[pymodule]
fn cairo_rs_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyVM>()?;
    m.add_function(wrap_pyfunction!(cairo_run_py, m)?)?;
    Ok(())
}
