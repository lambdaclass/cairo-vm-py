use crate::{
    memory_segments::PySegmentManager,
    relocatable::{PyMaybeRelocatable, PyRelocatable},
    utils::to_py_error,
};
use cairo_rs::{
    hint_processor::builtin_hint_processor::dict_manager::{DictManager, DictTracker, Dictionary},
    types::relocatable::Relocatable,
};
use num_bigint::BigInt;
use pyo3::{exceptions::PyAttributeError, prelude::*};
use std::collections::HashMap;

#[pyclass]
pub struct PyDictManager {
    pub manager: DictManager,
}

#[pyclass]
pub struct PyDictTracker {
    pub tracker: DictTracker,
}

#[pyclass]
pub struct PyDictionary {
    pub dictionary: Dictionary,
}

#[pymethods]
impl PyDictManager {
    #[new]
    pub fn new() -> Self {
        PyDictManager {
            manager: DictManager::new(),
        }
    }

    pub fn new_dict(
        &mut self,
        segments: &mut PySegmentManager,
        initial_dict: HashMap<BigInt, BigInt>,
        py: Python,
    ) -> PyResult<PyObject> {
        let res = Ok(PyMaybeRelocatable::from(
            self.manager
                .new_dict(&mut segments.vm.borrow_mut(), initial_dict)
                .map_err(to_py_error)?,
        )
        .to_object(py));
        res
    }

    pub fn new_default_dict(
        &mut self,
        segments: &mut PySegmentManager,
        default_value: BigInt,
        initial_dict: Option<HashMap<BigInt, BigInt>>,
        py: Python,
    ) -> PyResult<PyObject> {
        Ok(PyMaybeRelocatable::from(
            self.manager
                .new_default_dict(&mut segments.vm.borrow_mut(), &default_value, initial_dict)
                .map_err(to_py_error)?,
        )
        .to_object(py))
    }

    pub fn get_tracker(&mut self, dict_ptr: &PyRelocatable) -> PyResult<PyDictTracker> {
        Ok(PyDictTracker {
            tracker: self
                .manager
                .get_tracker_mut(&Relocatable {
                    segment_index: dict_ptr.segment_index,
                    offset: dict_ptr.offset,
                })
                .map_err(to_py_error)?
                .clone(),
        })
    }
}

#[pymethods]
impl PyDictTracker {
    pub fn __getattr__(&self, name: &str, py: Python) -> PyResult<PyObject> {
        if name == "current_ptr" {
            return Ok(PyMaybeRelocatable::from(&self.tracker.current_ptr).to_object(py));
        }

        if name == "data" {
            return Ok(PyDictionary {
                dictionary: self.tracker.data.clone(),
            }
            .into_py(py));
        }

        Err(PyAttributeError::new_err(name.to_string()))
    }

    pub fn __setattr__(&mut self, name: &str, val: PyRelocatable) -> PyResult<()> {
        if name == "current_ptr" {
            self.tracker.current_ptr.offset = val.offset;
            return Ok(());
        }
        Err(PyAttributeError::new_err(name.to_string()))
    }
}

#[pymethods]
impl PyDictionary {
    #[getter]
    pub fn __getitem__(&self, key: &BigInt) -> Option<&BigInt> {
        self.dictionary.get(key)
    }
}
