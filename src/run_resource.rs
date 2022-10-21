use std::ops::SubAssign;

use pyo3::{pyclass, pymethods};

#[pyclass]
pub struct RunResource {
    steps: Option<i32>,
}

#[pymethods]
impl RunResource {
    #[new]
    pub fn new(steps: Option<i32>) -> Self {
        Self { steps }
    }

    /// Returns true if the resources were consumed.
    pub fn consumed(&self) -> bool {
        match self.steps {
            Some(s) => s <= 0,
            None => false,
        }
    }

    /// Consumes one Cairo step.
    pub fn consume_step(&mut self) {
        match self.steps.as_mut() {
            Some(s) => s.sub_assign(1),
            None => {},
        }
    }
}
