use cairo_rs::{
    bigint,
    hint_processor::hint_processor_utils::bigint_to_usize,
    types::relocatable::{MaybeRelocatable, Relocatable},
};
use num_bigint::BigInt;
use pyo3::{exceptions::PyArithmeticError, prelude::*, pyclass::CompareOp};

const PYRELOCATABLE_COMPARE_ERROR: &str = "Cannot compare Relocatables of different segments";

#[derive(FromPyObject, Debug, Clone, PartialEq, Eq)]
pub enum PyMaybeRelocatable {
    Int(BigInt),
    RelocatableValue(PyRelocatable),
}

#[pyclass(name = "Relocatable")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyRelocatable {
    #[pyo3(get)]
    pub segment_index: isize,
    #[pyo3(get)]
    pub offset: usize,
}

#[pymethods]
impl PyRelocatable {
    #[new]
    pub fn new(tuple: (isize, usize)) -> PyRelocatable {
        PyRelocatable {
            segment_index: tuple.0,
            offset: tuple.1,
        }
    }

    pub fn __add__(&self, value: usize) -> PyRelocatable {
        PyRelocatable {
            segment_index: self.segment_index,
            offset: self.offset + value,
        }
    }

    pub fn __sub__(&self, value: PyMaybeRelocatable, py: Python) -> PyResult<PyObject> {
        match value {
            PyMaybeRelocatable::Int(value) => {
                Ok(PyMaybeRelocatable::RelocatableValue(PyRelocatable {
                    segment_index: self.segment_index,
                    offset: self.offset - bigint_to_usize(&value).unwrap(),
                })
                .to_object(py))
            }
            PyMaybeRelocatable::RelocatableValue(address) => {
                if self.segment_index == address.segment_index && self.offset >= address.offset {
                    return Ok(
                        PyMaybeRelocatable::Int(bigint!(self.offset - address.offset))
                            .to_object(py),
                    );
                }
                todo!()
            }
        }
    }

    pub fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => {
                if self.segment_index == other.segment_index {
                    Ok(self.offset < other.offset)
                } else {
                    Err(PyArithmeticError::new_err(PYRELOCATABLE_COMPARE_ERROR))
                }
            }
            CompareOp::Le => {
                if self.segment_index == other.segment_index {
                    Ok(self.offset <= other.offset)
                } else {
                    Err(PyArithmeticError::new_err(PYRELOCATABLE_COMPARE_ERROR))
                }
            }
            CompareOp::Eq => {
                Ok((self.segment_index, self.offset) == (other.segment_index, other.offset))
            }
            CompareOp::Ne => {
                Ok((self.segment_index, self.offset) != (other.segment_index, other.offset))
            }
            CompareOp::Gt => {
                if self.segment_index == other.segment_index {
                    Ok(self.offset > other.offset)
                } else {
                    Err(PyArithmeticError::new_err(PYRELOCATABLE_COMPARE_ERROR))
                }
            }
            CompareOp::Ge => {
                if self.segment_index == other.segment_index {
                    Ok(self.offset >= other.offset)
                } else {
                    Err(PyArithmeticError::new_err(PYRELOCATABLE_COMPARE_ERROR))
                }
            }
        }
    }

    pub fn __repr__(&self) -> String {
        format!("({}, {})", self.segment_index, self.offset)
    }
}

impl From<PyMaybeRelocatable> for MaybeRelocatable {
    fn from(val: PyMaybeRelocatable) -> Self {
        match val {
            PyMaybeRelocatable::RelocatableValue(rel) => MaybeRelocatable::RelocatableValue(
                Relocatable::from((rel.segment_index, rel.offset)),
            ),
            PyMaybeRelocatable::Int(num) => MaybeRelocatable::Int(num),
        }
    }
}

impl From<&PyMaybeRelocatable> for MaybeRelocatable {
    fn from(val: &PyMaybeRelocatable) -> Self {
        match val {
            PyMaybeRelocatable::RelocatableValue(rel) => MaybeRelocatable::RelocatableValue(
                Relocatable::from((rel.segment_index, rel.offset)),
            ),
            PyMaybeRelocatable::Int(num) => MaybeRelocatable::Int(num.clone()),
        }
    }
}

impl From<MaybeRelocatable> for PyMaybeRelocatable {
    fn from(val: MaybeRelocatable) -> Self {
        match val {
            MaybeRelocatable::RelocatableValue(rel) => PyMaybeRelocatable::RelocatableValue(
                PyRelocatable::new((rel.segment_index, rel.offset)),
            ),
            MaybeRelocatable::Int(num) => PyMaybeRelocatable::Int(num),
        }
    }
}

impl From<&MaybeRelocatable> for PyMaybeRelocatable {
    fn from(val: &MaybeRelocatable) -> Self {
        match val {
            MaybeRelocatable::RelocatableValue(rel) => PyMaybeRelocatable::RelocatableValue(
                PyRelocatable::new((rel.segment_index, rel.offset)),
            ),
            MaybeRelocatable::Int(num) => PyMaybeRelocatable::Int(num.clone()),
        }
    }
}

impl ToPyObject for PyMaybeRelocatable {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            PyMaybeRelocatable::RelocatableValue(address) => address.clone().into_py(py),
            PyMaybeRelocatable::Int(value) => value.clone().into_py(py),
        }
    }
}

impl From<Relocatable> for PyRelocatable {
    fn from(val: Relocatable) -> Self {
        PyRelocatable::new((val.segment_index, val.offset))
    }
}

impl From<&PyRelocatable> for Relocatable {
    fn from(val: &PyRelocatable) -> Self {
        Relocatable::from((val.segment_index, val.offset))
    }
}

impl From<(isize, usize)> for PyRelocatable {
    fn from(val: (isize, usize)) -> Self {
        PyRelocatable::new((val.0, val.1))
    }
}

impl From<Relocatable> for PyMaybeRelocatable {
    fn from(val: Relocatable) -> Self {
        PyMaybeRelocatable::RelocatableValue(val.into())
    }
}

impl From<&Relocatable> for PyMaybeRelocatable {
    fn from(val: &Relocatable) -> Self {
        PyMaybeRelocatable::RelocatableValue(PyRelocatable {
            segment_index: val.segment_index,
            offset: val.offset,
        })
    }
}

impl From<PyRelocatable> for PyMaybeRelocatable {
    fn from(val: PyRelocatable) -> Self {
        PyMaybeRelocatable::RelocatableValue(val)
    }
}

impl From<&BigInt> for PyMaybeRelocatable {
    fn from(val: &BigInt) -> Self {
        PyMaybeRelocatable::Int(val.clone())
    }
}

impl From<BigInt> for PyMaybeRelocatable {
    fn from(val: BigInt) -> Self {
        PyMaybeRelocatable::Int(val)
    }
}
