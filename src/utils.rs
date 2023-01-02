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

pub fn const_path_to_const_name(constants: &HashMap<String, BigInt>) -> HashMap<String, BigInt> {
    constants
        .iter()
        .map(|(name, value)| {
            let name = name.rsplit('.').next().unwrap_or(name);
            (name.to_string(), value.clone())
        })
        .collect()
}
