pub mod cairo_run;
pub mod cairo_runner;
pub mod ids;
mod memory;
mod memory_segments;
mod range_check;
mod relocatable;
mod scope_manager;
mod utils;
mod vm_core;

use cairo_runner::PyCairoRunner;
use relocatable::to_felt_or_relocatable;
use pyo3::prelude::*;

#[pymodule]
fn cairo_rs_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyCairoRunner>()?;
    m.add_function(wrap_pyfunction!(to_felt_or_relocatable))?;
    Ok(())
}
