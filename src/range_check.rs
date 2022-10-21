use cairo_rs::vm::{
    errors::vm_errors::VirtualMachineError, runners::builtin_runner::RangeCheckBuiltinRunner,
};

use num_bigint::BigInt;
use pyo3::prelude::*;

#[pyclass(name = "RangeCheck")]
#[derive(Clone, Debug, PartialEq)]
pub struct PyRangeCheck {
    #[pyo3(get)]
    bound: BigInt,
}

#[pymethods]
impl PyRangeCheck {
    #[new]
    pub fn new(value: BigInt) -> Self {
        Self { bound: value }
    }

    pub fn __repr__(&self) -> String {
        format!("Bound: {}", self.bound)
    }
}

impl From<Result<&RangeCheckBuiltinRunner, VirtualMachineError>> for PyRangeCheck {
    fn from(val: Result<&RangeCheckBuiltinRunner, VirtualMachineError>) -> Self {
        match val {
            Ok(range_check_builtin) => PyRangeCheck::from(range_check_builtin),
            Err(_err) => PyRangeCheck::new(BigInt::from(0)),
        }
    }
}

impl From<&RangeCheckBuiltinRunner> for PyRangeCheck {
    fn from(val: &RangeCheckBuiltinRunner) -> Self {
        Self {
            bound: val._bound.clone(),
        }
    }
}

impl ToPyObject for PyRangeCheck {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.clone().into_py(py)
    }
}
