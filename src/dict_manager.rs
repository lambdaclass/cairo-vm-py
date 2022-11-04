use crate::{
    memory_segments::PySegmentManager, relocatable::PyMaybeRelocatable, utils::to_py_error,
};
use cairo_rs::hint_processor::builtin_hint_processor::dict_manager::DictManager;
use num_bigint::BigInt;
use pyo3::{pyclass, pymethods, PyResult};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[pyclass(unsendable)]
pub struct PyDictManager {
    pub(crate) manager: Rc<RefCell<DictManager>>,
}

#[pymethods]
impl PyDictManager {
    pub fn new_dict(
        &mut self,
        segments: &mut PySegmentManager,
        initial_dict: HashMap<BigInt, BigInt>,
    ) -> PyResult<PyMaybeRelocatable> {
        Ok(self
            .manager
            .borrow_mut()
            .new_dict(&mut segments.vm.borrow_mut(), initial_dict)
            .map_err(to_py_error)?)
    }
}
