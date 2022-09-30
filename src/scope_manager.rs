use std::collections::HashMap;

use pyo3::{pyclass, pymethods, PyObject};

#[pyclass(unsendable)]
pub struct PyScopeManager {
    exit: i32,
    enter: Vec<HashMap<String, PyObject>>,
}

impl PyScopeManager {
    pub fn new() -> PyScopeManager {
        PyScopeManager {
            exit: 0,
            enter: Vec::new(),
        }
    }
}

#[pymethods]
impl PyScopeManager {
    pub fn enter_scope(&mut self, variables: Option<HashMap<String, PyObject>>) {
        match variables {
            Some(variables) => self.enter.push(variables),
            None => self.enter.push(HashMap::new()),
        }
    }

    pub fn exit_scope(&mut self) {
        self.exit += 1
    }
}
