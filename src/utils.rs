use cairo_rs::vm::errors::vm_errors::VirtualMachineError;
use felt::{Felt, FeltOps};
use num_bigint::BigInt;
use pyo3::{exceptions::PyValueError, PyErr};
use std::{collections::HashMap, fmt::Display};

#[macro_export]
macro_rules! pycell {
    ($py:expr, $val:expr) => {
        PyCell::new($py, $val)?
    };
}
pub fn to_py_error<T: Display>(error: T) -> PyErr {
    PyValueError::new_err(error.to_string())
}

#[macro_export]
macro_rules! bigint {
    ($val : expr) => {
        Into::<BigInt>::into($val)
    };
}

pub fn const_path_to_const_name(constants: &HashMap<String, Felt>) -> HashMap<String, BigInt> {
    constants
        .iter()
        .map(|(name, value)| {
            let name = name.rsplit('.').next().unwrap_or(name);
            (name.to_string(), value.to_bigint())
        })
        .collect()
}

//Tries to convert a BigInt value to usize
pub fn bigint_to_usize(bigint: &BigInt) -> Result<usize, VirtualMachineError> {
    bigint
        .try_into()
        .map_err(|_| VirtualMachineError::BigintToUsizeFail)
}
