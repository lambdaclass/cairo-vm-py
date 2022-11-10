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
use pyo3::prelude::*;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[pyclass(unsendable)]
pub struct PyDictManager {
    pub(crate) manager: Rc<RefCell<DictManager>>,
}

#[pyclass(unsendable)]
pub struct PyDictTracker {
    pub(crate) tracker: Rc<RefCell<DictTracker>>,
}

#[pyclass(unsendable)]
pub struct PyDictionary {
    pub(crate) dictionary: Rc<RefCell<Dictionary>>,
}

#[pymethods]
impl PyDictManager {
    #[new]
    pub fn new() -> Self {
        PyDictManager {
            manager: Rc::new(RefCell::new(DictManager::new())),
        }
    }

    pub fn new_dict(
        &mut self,
        segments: &mut PySegmentManager,
        initial_dict: HashMap<BigInt, BigInt>,
        py: Python,
    ) -> PyResult<PyObject> {
        Ok(PyMaybeRelocatable::from(
            self.manager
                .borrow_mut()
                .new_dict(&mut segments.vm.borrow_mut(), initial_dict)
                .map_err(to_py_error)?,
        )
        .to_object(py))
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
                .borrow_mut()
                .new_default_dict(&mut segments.vm.borrow_mut(), &default_value, initial_dict)
                .map_err(to_py_error)?,
        )
        .to_object(py))
    }

    pub fn get_tracker(&mut self, dict_ptr: &PyRelocatable, py: Python) -> PyResult<&mut PyObject> {
        Ok(self
            .manager
            .borrow_mut()
            .get_tracker_mut(&Relocatable {
                segment_index: dict_ptr.segment_index,
                offset: dict_ptr.offset,
            })
            .map_err(to_py_error)?
            .to_object(py))
    }
}
