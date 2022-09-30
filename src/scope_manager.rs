use std::collections::HashMap;

use pyo3::{pyclass, pyfunction, pymethods, types::PyModule, PyObject, PyResult};

#[pyclass(unsendable)]
pub struct PyEnterScope {
    new_scopes: Vec<HashMap<String, PyObject>>,
}

impl PyEnterScope {
    pub fn new() -> PyEnterScope {
        PyEnterScope {
            new_scopes: Vec::<HashMap<String, PyObject>>::new(),
        }
    }
}

#[pymethods]
impl PyEnterScope {
    pub fn __call__(&mut self, variables: Option<HashMap<String, PyObject>>) {
        match variables {
            Some(variables) => self.new_scopes.push(variables),
            None => self.new_scopes.push(HashMap::new()),
        }
    }
}

#[pyclass(unsendable)]
pub struct PyExitScope {
    num: i32,
}

impl PyExitScope {
    pub fn new() -> PyExitScope {
        PyExitScope { num: 0 }
    }
}

#[pymethods]
impl PyExitScope {
    pub fn __call__(&mut self) {
        self.num += 1
    }
}
