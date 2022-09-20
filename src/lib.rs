mod memory;
mod memory_segments;
mod relocatable;

use crate::memory::PyMemory;
use crate::memory_segments::PySegmentManager;
use crate::relocatable::PyRelocatable;
use pyo3::prelude::*;

#[pymodule]
fn cairo_rs_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyMemory>()?;
    m.add_class::<PyRelocatable>()?;
    m.add_class::<PySegmentManager>()?;
    Ok(())
}
