use cairo_vm::{
    felt::Felt252, types::errors::math_errors::MathError,
    vm::errors::vm_errors::VirtualMachineError,
};
use num_bigint::BigUint;
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
macro_rules! biguint {
    ($val : expr) => {
        Into::<BigUint>::into($val)
    };
}

#[macro_export]
macro_rules! any_box {
    ($val : expr) => {
        Box::new($val) as Box<dyn Any>
    };
}

pub fn const_path_to_const_name(constants: &HashMap<String, Felt252>) -> HashMap<String, BigUint> {
    constants
        .iter()
        .map(|(name, value)| {
            let name = name.rsplit('.').next().unwrap_or(name);
            (name.to_string(), value.to_biguint())
        })
        .collect()
}

//Tries to convert a biguint value to usize
pub fn biguint_to_usize(biguint: &BigUint) -> Result<usize, VirtualMachineError> {
    biguint
        .try_into()
        .map_err(|_| MathError::Felt252ToUsizeConversion(biguint.into()).into())
}
