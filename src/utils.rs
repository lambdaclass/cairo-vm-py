use std::fmt::Display;
use cairo_rs::vm::errors::vm_errors::VirtualMachineError;
use pyo3::{exceptions::PyValueError, PyErr};

#[macro_export]
macro_rules! pycell {
    ($py:expr, $val:expr) => {
        PyCell::new($py, $val).map_err(to_vm_error)?
    };
}

pub fn to_vm_error(pyerror: PyErr) -> VirtualMachineError {
    VirtualMachineError::CustomHint(format!("{}", pyerror))
}

pub fn to_py_error<T: Display>(error: T) -> PyErr {
    PyValueError::new_err(format!("{}", error))
}
