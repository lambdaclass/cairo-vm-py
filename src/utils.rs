use cairo_rs::vm::errors::vm_errors::VirtualMachineError;
use num_bigint::BigInt;
use pyo3::{pyclass, IntoPy, Py, PyAny, PyErr, Python};
use std::{collections::HashMap, fmt::Display};

pyo3::import_exception!(starkware.cairo.lang.vm.vm_exceptions, VmException);
pyo3::import_exception!(starkware.starknet.core.os, HandlerException);

#[pyclass(unsendable)]
struct HintException {
    #[pyo3(get, set)]
    inner_exc: Py<PyAny>,
}

#[macro_export]
macro_rules! pycell {
    ($py:expr, $val:expr) => {
        PyCell::new($py, $val).map_err(|err| to_vm_error(err, $py))?
    };
}

pub fn to_vm_error(pyerror: PyErr, py: Python) -> VirtualMachineError {
    let value = pyerror.value(py);
    VirtualMachineError::CustomHint(format!("{:?}", value))
}

pub fn to_py_error<T: Display>(error: T) -> PyErr {
    // if let VirtualMachineError::CustomHint(pyerror) = error {
    //     if pyerror.contains("HandlerException") {
    //         VmException::new_err((0, None::<i32>, HintException { inner_exc: error.to_py(py)}, None::<i32>, None::<i32>, [error.to_string()]))
    //     }
    // }
    VmException::new_err((
        0,
        None::<i32>,
        1,
        None::<i32>,
        None::<i32>,
        [error.to_string()],
    ))

    // PyValueError::new_err(format!("{}", error))
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
