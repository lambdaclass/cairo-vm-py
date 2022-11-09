use crate::{
    relocatable::{PyMaybeRelocatable, PyRelocatable},
    vm_core::PyVM,
};
use cairo_rs::{
    types::relocatable::{MaybeRelocatable, Relocatable},
    vm::vm_core::VirtualMachine,
};
use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    prelude::*,
};
use std::{cell::RefCell, rc::Rc};

const MEMORY_GET_ERROR_MSG: &str = "Failed to get value from Cairo memory";
const MEMORY_SET_ERROR_MSG: &str = "Failed to set value to Cairo memory";
const MEMORY_GET_RANGE_ERROR_MSG: &str = "Failed to call get_range method from Cairo memory";
const MEMORY_ADD_RELOCATION_RULE_ERROR_MSG: &str =
    "Failed to call add_relocation_rule method from Cairo memory";

#[pyclass(unsendable)]
#[derive(Clone)]
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
        match self
            .vm
            .borrow()
            .get_maybe(key)
            .map_err(|_| PyTypeError::new_err(MEMORY_GET_ERROR_MSG))?
        {
            Some(maybe_reloc) => Ok(Some(PyMaybeRelocatable::from(maybe_reloc).to_object(py))),
            None => Ok(None),
        }
    }

    #[setter]
    pub fn __setitem__(&self, key: &PyRelocatable, value: PyMaybeRelocatable) -> PyResult<()> {
        let key: Relocatable = key.into();
        let value: MaybeRelocatable = value.into();

        self.vm
            .borrow_mut()
            .insert_value(&key, value)
            .map_err(|_| PyValueError::new_err(MEMORY_SET_ERROR_MSG))
    }

    pub fn get_range(
        &self,
        addr: PyMaybeRelocatable,
        size: usize,
        py: Python,
    ) -> PyResult<PyObject> {
        Ok(self
            .vm
            .borrow()
            .get_continuous_range(&MaybeRelocatable::from(addr), size)
            .map_err(|_| PyTypeError::new_err(MEMORY_GET_RANGE_ERROR_MSG))?
            .into_iter()
            .map(Into::<PyMaybeRelocatable>::into)
            .collect::<Vec<PyMaybeRelocatable>>()
            .to_object(py))
    }

    pub fn add_relocation_rule(
        &self,
        src_ptr: PyRelocatable,
        dest_ptr: PyRelocatable,
    ) -> Result<(), PyErr> {
        self.vm
            .borrow_mut()
            .add_relocation_rule(Relocatable::from(&src_ptr), Relocatable::from(&dest_ptr))
            .map_err(|_| PyTypeError::new_err(MEMORY_ADD_RELOCATION_RULE_ERROR_MSG))
    }
}

#[cfg(test)]
mod test {
    use crate::utils::to_vm_error;
    use crate::vm_core::PyVM;
    use crate::{memory::PyMemory, relocatable::PyRelocatable};
    use num_bigint::{BigInt, Sign};
    use pyo3::PyCell;
    use pyo3::{types::PyDict, Python};

    #[test]
    fn memory_insert_test() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }
            let memory = PyMemory::new(&vm);
            let ap = PyRelocatable::from(vm.vm.borrow().get_ap());

            let globals = PyDict::new(py);
            globals
                .set_item("memory", PyCell::new(py, memory).unwrap())
                .unwrap();
            globals
                .set_item("ap", PyCell::new(py, ap).unwrap())
                .unwrap();

            let code = "memory[ap] = 5";

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(to_vm_error), Ok(()));
        });
    }

    #[test]
    fn memory_insert_ocuppied_address_error_test() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }
            let memory = PyMemory::new(&vm);
            let ap = PyRelocatable::from(vm.vm.borrow().get_ap());

            let globals = PyDict::new(py);
            globals
                .set_item("memory", PyCell::new(py, memory).unwrap())
                .unwrap();
            globals
                .set_item("ap", PyCell::new(py, ap).unwrap())
                .unwrap();

            // we try to insert to the same address two times
            let code = r#"
memory[ap] = 5
memory[ap] = 3
"#;

            let py_result = py.run(code, Some(globals), None);

            assert!(py_result.is_err());
        });
    }

    #[test]
    fn memory_get_test() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }
            let memory = PyMemory::new(&vm);
            let ap = PyRelocatable::from((1, 1));
            let fp = PyRelocatable::from((1, 2));

            let globals = PyDict::new(py);
            globals
                .set_item("memory", PyCell::new(py, memory).unwrap())
                .unwrap();
            globals
                .set_item("ap", PyCell::new(py, ap).unwrap())
                .unwrap();
            globals
                .set_item("fp", PyCell::new(py, fp).unwrap())
                .unwrap();

            let code = r#"
memory[ap] = fp
assert memory[ap] == fp
"#;

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(to_vm_error), Ok(()));
        });
    }
}
