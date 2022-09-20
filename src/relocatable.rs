use cairo_rs::{
    bigint,
    hint_processor::hint_processor_utils::bigint_to_usize,
    types::relocatable::{MaybeRelocatable, Relocatable},
};
use num_bigint::BigInt;
use pyo3::prelude::*;

#[derive(FromPyObject, Debug, Clone)]
pub enum PyMaybeRelocatable {
    Int(BigInt),
    RelocatableValue(PyRelocatable),
}

#[pyclass(name = "Relocatable")]
#[derive(Clone, Debug)]
pub struct PyRelocatable {
    index: usize,
    offset: usize,
}

#[pymethods]
impl PyRelocatable {
    #[new]
    pub fn new(tuple: (usize, usize)) -> PyRelocatable {
        PyRelocatable {
            index: tuple.0,
            offset: tuple.1,
        }
    }

    pub fn __add__(&self, value: usize) -> PyRelocatable {
        PyRelocatable {
            index: self.index,
            offset: self.offset + value,
        }
    }

    pub fn __sub__(&self, value: PyMaybeRelocatable, py: Python) -> PyResult<PyObject> {
        match value {
            PyMaybeRelocatable::Int(value) => {
                return Ok(PyMaybeRelocatable::RelocatableValue(PyRelocatable {
                    index: self.index,
                    offset: self.offset - bigint_to_usize(&value).unwrap(),
                })
                .to_object(py));
            }
            PyMaybeRelocatable::RelocatableValue(address) => {
                if self.index == address.index && self.offset >= address.offset {
                    return Ok(
                        PyMaybeRelocatable::Int(bigint!(self.offset - address.offset))
                            .to_object(py),
                    );
                }
                todo!()
            }
        }
    }

    pub fn __repr__(&self) -> String {
        format!("({}, {})", self.index, self.offset)
    }
}

impl From<PyMaybeRelocatable> for MaybeRelocatable {
    fn from(val: PyMaybeRelocatable) -> Self {
        match val {
            PyMaybeRelocatable::RelocatableValue(rel) => {
                MaybeRelocatable::RelocatableValue(Relocatable::from((rel.index, rel.offset)))
            }
            PyMaybeRelocatable::Int(num) => MaybeRelocatable::Int(BigInt::from(num)),
        }
    }
}

impl From<&PyMaybeRelocatable> for MaybeRelocatable {
    fn from(val: &PyMaybeRelocatable) -> Self {
        match val {
            PyMaybeRelocatable::RelocatableValue(rel) => {
                MaybeRelocatable::RelocatableValue(Relocatable::from((rel.index, rel.offset)))
            }
            PyMaybeRelocatable::Int(num) => MaybeRelocatable::Int(num.clone()),
        }
    }
}

impl PyMaybeRelocatable {
    pub fn to_maybe_relocatable(&self) -> MaybeRelocatable {
        MaybeRelocatable::from(self)
    }
}

impl PyRelocatable {
    pub fn to_relocatable(&self) -> Relocatable {
        Relocatable {
            segment_index: self.index,
            offset: self.offset,
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

impl From<Relocatable> for PyMaybeRelocatable {
    fn from(val: Relocatable) -> Self {
        PyMaybeRelocatable::RelocatableValue(val.into())
    }
}
