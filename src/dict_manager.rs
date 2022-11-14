use crate::{
    memory_segments::PySegmentManager,
    relocatable::{PyMaybeRelocatable, PyRelocatable},
    utils::to_py_error,
};
use cairo_rs::{
    hint_processor::builtin_hint_processor::dict_manager::DictManager,
    types::relocatable::Relocatable,
};
use num_bigint::BigInt;
use pyo3::{
    exceptions::{PyAttributeError, PyKeyError},
    prelude::*,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[pyclass(unsendable)]
pub struct PyDictManager {
    pub manager: Rc<RefCell<DictManager>>,
    pub last_tracker: Relocatable,
}

#[pymethods]
impl PyDictManager {
    #[new]
    pub fn new() -> Self {
        PyDictManager {
            manager: Rc::new(RefCell::new(DictManager::new())),
            last_tracker: Relocatable::from((0, 0)),
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
                .borrow_mut()
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
                .borrow_mut()
                .new_default_dict(&mut segments.vm.borrow_mut(), &default_value, initial_dict)
                .map_err(to_py_error)?,
        )
        .to_object(py))
    }

    pub fn get_tracker(&mut self, dict_ptr: &PyRelocatable) -> PyResult<PyDictManager> {
        self.manager
            .borrow_mut()
            .trackers
            .get_mut(&dict_ptr.segment_index)
            .unwrap()
            .current_ptr
            .offset = dict_ptr.offset;
        self.last_tracker = Relocatable::from((dict_ptr.segment_index, dict_ptr.offset));
        Ok(PyDictManager {
            manager: self.manager.to_owned(),
            last_tracker: self.last_tracker.to_owned(),
        })
    }

    pub fn __getattr__(&self, name: &str, py: Python) -> PyResult<PyObject> {
        if name == "current_ptr" {
            return Ok(PyRelocatable::from(
                self.manager
                    .borrow_mut()
                    .trackers
                    .get_mut(&self.last_tracker.segment_index)
                    .unwrap()
                    .current_ptr
                    .to_owned(),
            )
            .into_py(py));
        }

        if name == "data" {
            return Ok(PyDictManager {
                manager: self.manager.to_owned(),
                last_tracker: self.last_tracker.to_owned(),
            }
            .into_py(py));
        }

        Err(PyAttributeError::new_err(name.to_string()))
    }

    pub fn __setattr__(&mut self, name: &str, val: PyRelocatable) -> PyResult<()> {
        if name == "current_ptr" {
            self.manager
                .borrow_mut()
                .trackers
                .get_mut(&val.segment_index)
                .unwrap()
                .current_ptr
                .offset = val.offset;
            return Ok(());
        }
        Err(PyAttributeError::new_err(name.to_string()))
    }

    #[getter]
    pub fn __getitem__(&self, key: PyMaybeRelocatable, py: Python) -> PyResult<PyObject> {
        match key {
            PyMaybeRelocatable::Int(key) => Ok(PyMaybeRelocatable::from(
                self.manager
                    .borrow_mut()
                    .trackers
                    .get_mut(&self.last_tracker.segment_index)
                    .unwrap()
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
                    .trackers
                    .get_mut(&self.last_tracker.segment_index)
                    .unwrap()
                    .insert_value(&key, &val);

                Ok(())
            }
            _ => todo!(),
        }
    }
}
