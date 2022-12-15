use cairo_rs::serde::deserialize_program::Location;
use pyo3::prelude::*;

#[pyclass]
#[pyo3(name = "CairoRunner")]
pub struct VmException {
    pub pc: usize,
    pub inst_location: Option<Location>,
    pub inner_exc: PyErr,
    pub error_attr_value: Option<String>,
}

// Implement new_vm_exception in PyCairoRunner
// to_py_error no longer creates a VmException
// Check if starknet recognizes this pyclass as a VmException
// Add #[get] attribute to attributes
