use std::collections::HashMap;

use pyo3::{pyclass, pyfunction, pymethods, types::PyModule, PyObject, PyResult};

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

#[pyfunction]
#[pyo3(pass_module)]
pub fn vm_enter_scope(
    module: &PyModule,
    variables: Option<HashMap<String, PyObject>>,
) -> PyResult<()> {
    let scope_manager = module.getattr("scope")?;
    scope_manager.call_method1("enter_scope", (variables,))?;
    Ok(())
}

#[pyfunction]
#[pyo3(pass_module)]
pub fn vm_exit_scope(module: &PyModule) -> PyResult<()> {
    let scope_manager = module.getattr("scope")?;
    scope_manager.call_method0("exit_scope")?;
    Ok(())
}
