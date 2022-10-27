use cairo_rs::vm::errors::vm_errors::VirtualMachineError;
use num_bigint::BigInt;
use pyo3::{exceptions::PyValueError, PyErr};
use std::{collections::HashMap, fmt::Display};

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

fn get_constant_name(const_str: String) -> Result<String, VirtualMachineError> {
    let mut const_name: Vec<String> = const_str.split('.').map(|s| s.to_string()).collect();
    const_name.pop().ok_or(VirtualMachineError::FailedToGetIds)
}

pub fn const_path_to_const_name(
    constants: HashMap<String, BigInt>,
) -> Result<HashMap<String, BigInt>, VirtualMachineError> {
    let mut const_map = HashMap::new();
    for (key, value) in constants {
        const_map.insert(get_constant_name(key)?, value);
    }
    Ok(const_map)
}
