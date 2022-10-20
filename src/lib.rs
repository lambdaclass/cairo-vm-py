pub mod cairo_run;
pub mod ids;
mod memory;
mod memory_segments;
mod range_check;
mod relocatable;
mod scope_manager;
mod utils;
mod vm_core;

use cairo_run::cairo_run_py;
use pyo3::prelude::*;
use vm_core::PyVM;

#[pymodule]
fn cairo_rs_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyVM>()?;
    m.add_function(wrap_pyfunction!(cairo_run_py, m)?)?;
    Ok(())
}
