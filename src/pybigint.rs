use cairo_rs::bigint_str;
use num_bigint::BigInt;
use pyo3::{prelude::*, types::PyDict};

#[pyclass]
#[derive(Debug)]
pub struct PyBigInt {
    pub value: String,
}

impl ToPyObject for PyBigInt {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let number_string = self.value.clone();
        let pystring = number_string.into_py(py);
        let locals = PyDict::new(py);
        locals.set_item("pystring", pystring).unwrap();
        let result = py.eval("int(pystring)", None, Some(locals)).unwrap();
        result.to_object(py)
    }
}

impl PyBigInt {
    pub fn new(value: &str) -> Self {
        PyBigInt {
            value: value.to_string(),
        }
    }
}

impl<'a> FromPyObject<'a> for PyBigInt {
    fn extract(ob: &'a PyAny) -> PyResult<Self> {
        let ob_as_string = ob.to_string();
        Ok(PyBigInt {
            value: ob_as_string,
        })
    }
}

impl From<PyBigInt> for BigInt {
    fn from(py_object: PyBigInt) -> Self {
        bigint_str!(py_object.value.as_bytes())
    }
}

impl From<&PyBigInt> for BigInt {
    fn from(py_object: &PyBigInt) -> Self {
        bigint_str!(py_object.value.as_bytes())
    }
}

impl From<BigInt> for PyBigInt {
    fn from(bi: BigInt) -> Self {
        PyBigInt {
            value: bi.to_string(),
        }
    }
}

impl From<&BigInt> for PyBigInt {
    fn from(bi: &BigInt) -> Self {
        PyBigInt {
            value: bi.to_string(),
        }
    }
}

impl From<usize> for PyBigInt {
    fn from(value: usize) -> Self {
        PyBigInt {
            value: value.to_string(),
        }
    }
}
