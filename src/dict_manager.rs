use crate::{
    ids::PyTypedId,
    memory_segments::PySegmentManager,
    relocatable::{PyMaybeRelocatable, PyRelocatable},
    utils::to_py_error,
};
use cairo_rs::{
    hint_processor::builtin_hint_processor::dict_manager::DictManager,
    types::relocatable::Relocatable,
};
use num_bigint::BigInt;
use pyo3::{exceptions::PyKeyError, prelude::*};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[pyclass(unsendable)]
pub struct PyDictManager {
    pub manager: Rc<RefCell<DictManager>>,
}

#[pyclass(unsendable)]
pub struct PyDictTracker {
    pub manager: Rc<RefCell<DictManager>>,
    pub key: Relocatable,
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
        &self,
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

    pub fn get_tracker(&mut self, dict_ptr: &PyTypedId) -> PyResult<PyDictTracker> {
        let ptr_addr = dict_ptr.hint_value.clone();
        self.manager
            .borrow()
            .get_tracker(&ptr_addr)
            .map_err(to_py_error)?;
        Ok(PyDictTracker {
            manager: self.manager.clone(),
            key: ptr_addr,
        })
    }
}

#[pymethods]
impl PyDictTracker {
    #[getter]
    pub fn get_current_ptr(&self, py: Python) -> PyResult<PyObject> {
        Ok(PyRelocatable::from(
            self.manager
                .borrow_mut()
                .get_tracker_mut(&self.key)
                .map_err(to_py_error)?
                .current_ptr
                .clone(),
        )
        .into_py(py))
    }

    #[getter]
    pub fn get_data(&self, py: Python) -> PyObject {
        PyDictTracker {
            manager: self.manager.clone(),
            key: self.key.clone(),
        }
        .into_py(py)
    }

    #[setter]
    pub fn set_current_ptr(&mut self, val: PyRelocatable) -> PyResult<()> {
        self.manager
            .borrow_mut()
            .get_tracker_mut(&self.key)
            .map_err(to_py_error)?
            .current_ptr
            .offset = val.offset;
        self.key = Relocatable {
            segment_index: val.segment_index,
            offset: val.offset,
        };
        Ok(())
    }

    #[getter]
    pub fn __getitem__(&self, key: PyMaybeRelocatable, py: Python) -> PyResult<PyObject> {
        match key {
            PyMaybeRelocatable::Int(key) => Ok(PyMaybeRelocatable::from(
                self.manager
                    .borrow_mut()
                    .get_tracker_mut(&self.key)
                    .map_err(to_py_error)?
                    .get_value(&key)
                    .map_err(to_py_error)?,
            )
            .to_object(py)),
            PyMaybeRelocatable::RelocatableValue(_) => Err(PyKeyError::new_err(key.to_object(py))),
        }
    }

    #[setter]
    pub fn __setitem__(
        &mut self,
        key: PyMaybeRelocatable,
        val: PyMaybeRelocatable,
    ) -> PyResult<()> {
        match (key, val) {
            (PyMaybeRelocatable::Int(key), PyMaybeRelocatable::Int(val)) => {
                self.manager
                    .borrow_mut()
                    .get_tracker_mut(&self.key)
                    .map_err(to_py_error)?
                    .insert_value(&key, &val);

                Ok(())
            }
            _ => todo!(),
        }
    }
}
