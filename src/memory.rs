use crate::{
    relocatable::{PyMaybeRelocatable, PyRelocatable},
    utils::to_py_error,
    vm_core::PyVM,
};

use cairo_vm::{
    types::relocatable::{MaybeRelocatable, Relocatable},
    vm::vm_core::VirtualMachine,
};
use num_bigint::BigUint;
use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    prelude::*,
};
use std::{cell::RefCell, rc::Rc};

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
    pub fn __getitem__(&self, key: &PyRelocatable, py: Python) -> Option<PyObject> {
        self.vm
            .borrow()
            .get_maybe(key)
            .map(|x| PyMaybeRelocatable::from(x).to_object(py))
    }

    #[setter]
    pub fn __setitem__(&self, key: &PyRelocatable, value: PyMaybeRelocatable) -> PyResult<()> {
        let key: Relocatable = key.into();
        let value: MaybeRelocatable = value.into();

        self.vm
            .borrow_mut()
            .insert_value(key, value)
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
            .get_continuous_range(
                MaybeRelocatable::from(addr)
                    .get_relocatable()
                    .ok_or_else(|| {
                        PyTypeError::new_err("Cannot get range from non-relocatable address")
                    })?,
                size,
            )
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

    /// Return a continuous section of memory as a vector of integers.
    pub fn get_range_as_ints(&self, addr: PyRelocatable, size: usize) -> PyResult<Vec<BigUint>> {
        Ok(self
            .vm
            .borrow()
            .get_integer_range(Relocatable::from(&addr), size)
            .map_err(to_py_error)?
            .into_iter()
            .map(|num| num.into_owned().to_biguint())
            .collect())
    }
}

#[cfg(test)]
mod test {
    use crate::biguint;
    use crate::relocatable::PyMaybeRelocatable;
    use crate::relocatable::PyMaybeRelocatable::RelocatableValue;
    use crate::vm_core::PyVM;
    use crate::{memory::PyMemory, relocatable::PyRelocatable};
    use cairo_vm::types::relocatable::{MaybeRelocatable, Relocatable};
    use num_bigint::BigUint;
    use pyo3::PyCell;
    use pyo3::{types::PyDict, Python};

    #[test]
    fn memory_insert_test() {
        Python::with_gil(|py| {
            let vm = PyVM::new(false);
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

            assert!(py_result.is_ok());
        });
    }

    #[test]
    fn memory_insert_ocuppied_address_error_test() {
        Python::with_gil(|py| {
            let vm = PyVM::new(false);
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
            let vm = PyVM::new(false);
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

            assert!(py_result.is_ok());
        });
    }

    #[test]
    fn get_range() {
        Python::with_gil(|py| {
            let vm = PyVM::new(false);

            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }

            vm.vm.borrow_mut().set_pc(Relocatable::from((0, 0)));
            vm.vm.borrow_mut().set_ap(2);
            vm.vm.borrow_mut().set_fp(2);

            vm.vm
                .borrow_mut()
                .insert_value(Relocatable::from((0, 0)), 2345108766317314046)
                .unwrap();
            vm.vm
                .borrow_mut()
                .insert_value(Relocatable::from((1, 0)), Relocatable::from((2, 0)))
                .unwrap();
            vm.vm
                .borrow_mut()
                .insert_value(Relocatable::from((1, 1)), Relocatable::from((3, 0)))
                .unwrap();

            let maybe_relocatable = MaybeRelocatable::from((1, 0));
            let size = 2;
            let memory = PyMemory::new(&vm);

            let range = memory
                .get_range(maybe_relocatable.into(), size, py)
                .unwrap()
                .extract::<Vec<PyMaybeRelocatable>>(py)
                .unwrap();

            assert_eq!(
                range,
                vec![
                    RelocatableValue(PyRelocatable {
                        segment_index: 2,
                        offset: 0
                    }),
                    RelocatableValue(PyRelocatable {
                        segment_index: 3,
                        offset: 0
                    })
                ]
            );
        });
    }

    #[test]
    fn get_range_with_gap() {
        Python::with_gil(|py| {
            let vm = PyVM::new(false);

            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }

            vm.vm.borrow_mut().set_pc(Relocatable::from((0, 0)));
            vm.vm.borrow_mut().set_ap(2);
            vm.vm.borrow_mut().set_fp(2);

            vm.vm
                .borrow_mut()
                .insert_value(Relocatable::from((0, 0)), 2345108766317314046)
                .unwrap();
            vm.vm
                .borrow_mut()
                .insert_value(Relocatable::from((1, 0)), Relocatable::from((2, 0)))
                .unwrap();
            vm.vm
                .borrow_mut()
                .insert_value(Relocatable::from((1, 2)), Relocatable::from((3, 0)))
                .unwrap();

            let maybe_relocatable = MaybeRelocatable::from((1, 0));
            let size = 2;
            let memory = PyMemory::new(&vm);

            let range = memory.get_range(maybe_relocatable.into(), size, py);

            assert!(range.is_err());
            assert!(range
                .unwrap_err()
                .to_string()
                .contains("Failed to call get_range method from Cairo memory"));
        });
    }

    // Test that get_range_as_ints() works as intended.
    #[test]
    fn get_range_as_ints() {
        let vm = PyVM::new(false);
        let memory = PyMemory::new(&vm);

        let addr = {
            let mut vm = vm.vm.borrow_mut();
            let addr = vm.add_memory_segment();

            vm.load_data(
                MaybeRelocatable::from(&addr).get_relocatable().unwrap(),
                &vec![1.into(), 2.into(), 3.into(), 4.into()],
            )
            .expect("memory insertion failed");

            addr
        };

        assert_eq!(
            memory
                .get_range_as_ints(addr.into(), 4)
                .expect("get_range_as_ints() failed"),
            vec![
                biguint!(1_u32),
                biguint!(2_u32),
                biguint!(3_u32),
                biguint!(4_u32)
            ],
        );
    }

    // Test that get_range_as_ints() fails when not all values are integers.
    #[test]
    fn get_range_as_ints_mixed() {
        let vm = PyVM::new(false);
        let memory = PyMemory::new(&vm);

        let addr = {
            let mut vm = vm.vm.borrow_mut();
            let addr = vm.add_memory_segment();

            vm.load_data(
                MaybeRelocatable::from(&addr).get_relocatable().unwrap(),
                &vec![
                    1.into(),
                    2.into(),
                    MaybeRelocatable::RelocatableValue((1, 2).into()),
                    4.into(),
                ],
            )
            .expect("memory insertion failed");

            addr
        };

        memory
            .get_range_as_ints(addr.into(), 4)
            .expect_err("get_range_as_ints() succeeded (should have failed)");
    }

    // Test that get_range_as_ints() fails when the requested range is larger than the available
    // segments.
    #[test]
    fn get_range_as_ints_incomplete() {
        let vm = PyVM::new(false);
        let memory = PyMemory::new(&vm);

        let addr = {
            let mut vm = vm.vm.borrow_mut();
            let addr = vm.add_memory_segment();

            vm.load_data(addr, &vec![1.into(), 2.into(), 3.into(), 4.into()])
                .expect("memory insertion failed");

            addr
        };

        memory
            .get_range_as_ints(addr.into(), 8)
            .expect_err("get_range_as_ints() succeeded (should have failed)");
    }
}
