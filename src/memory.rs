use crate::{
    relocatable::{PyMaybeRelocatable, PyRelocatable},
    vm_core::PyVM,
};
use cairo_rs::{types::relocatable::MaybeRelocatable, vm::vm_core::VirtualMachine};
use num_bigint::BigInt;
use pyo3::{
    exceptions::{PyKeyError, PyTypeError, PyValueError},
    prelude::*,
};
use std::{cell::RefCell, rc::Rc};

const MEMORY_GET_ERROR_MSG: &str = "Failed to get value from Cairo memory";
const MEMORY_SET_ERROR_MSG: &str = "Failed to set value to Cairo memory";
const MEMORY_SET_TYPE_ERROR_MSG: &str = "Failed to set downcast Python value";

#[pyclass(unsendable)]
pub struct PyMemory {
    vm: Rc<RefCell<VirtualMachine>>,
}

#[pymethods]
impl PyMemory {
    #[new]
    pub fn new(vm: &PyVM) -> PyMemory {
        PyMemory { vm: vm.get_vm() }
    }

    #[getter]
    pub fn __getitem__(&self, key: &PyRelocatable, py: Python) -> PyResult<Option<PyObject>> {
        let key = key.to_relocatable();
        match self.vm.borrow().memory.get(&key) {
            Ok(Some(maybe_reloc)) => Ok(Some(PyMaybeRelocatable::from(maybe_reloc).to_object(py))),
            Ok(None) => Ok(None),
            Err(_) => Err(PyKeyError::new_err(MEMORY_GET_ERROR_MSG)),
        }
    }

    #[setter]
    pub fn __setitem__(&self, key: &PyRelocatable, value: &PyAny) -> PyResult<()> {
        let key = key.to_relocatable();

        let value = if let Ok(num) = value.extract::<BigInt>() {
            MaybeRelocatable::from(num)
        } else if let Ok(pyrelocatable) = value.extract::<PyRelocatable>() {
            MaybeRelocatable::from(pyrelocatable.to_relocatable())
        } else if let Ok(py_maybe_reloc) = value.extract::<PyMaybeRelocatable>() {
            py_maybe_reloc.to_maybe_relocatable()
        } else {
            return Err(PyTypeError::new_err(MEMORY_SET_TYPE_ERROR_MSG));
        };

        self.vm
            .borrow_mut()
            .memory
            .insert_value(&key, value)
            .map_err(|_| PyValueError::new_err(MEMORY_SET_ERROR_MSG))
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::*;

    #[test]
    fn new_memory_test() {
        Python::with_gil(|py| {
            let res = py.run(
                r#"
import cairo_rs
memory = cairo_rs.PyMemory()
            "#,
                None,
                None,
            );

            assert!(res.is_ok());
        });
    }
}
