pub mod cairo_run;
pub mod cairo_runner;
mod ecdsa;
pub mod ids;
mod instruction_location;
mod memory;
mod memory_segments;
mod range_check;
mod relocatable;
mod run_context;
mod scope_manager;
mod to_felt_or_relocatable;
mod utils;
mod vm_core;

#[cfg(all(feature = "extension-module", feature = "embedded-python"))]
compile_error!("\"extension-module\" is incompatible with \"embedded-python\" as it inhibits linking with cpython");

use cairo_runner::PyCairoRunner;
use pyo3::prelude::*;

#[pymodule]
fn cairo_rs_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyCairoRunner>()?;
    Ok(())
}
