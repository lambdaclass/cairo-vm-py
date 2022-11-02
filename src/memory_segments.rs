use crate::{
    relocatable::{PyMaybeRelocatable, PyRelocatable},
    utils::to_py_error,
    vm_core::PyVM,
};
use cairo_rs::{types::relocatable::MaybeRelocatable, vm::vm_core::VirtualMachine};
use pyo3::{prelude::*, types::PyIterator};
use std::{cell::RefCell, rc::Rc};

#[pyclass(name = "MemorySegmentManager", unsendable)]
pub struct PySegmentManager {
    vm: Rc<RefCell<VirtualMachine>>,
}

#[pymethods]
impl PySegmentManager {
    #[new]
    pub fn new(vm: &PyVM) -> PySegmentManager {
        PySegmentManager { vm: vm.get_vm() }
    }

    pub fn add(&self) -> PyResult<PyRelocatable> {
        Ok(self.vm.borrow_mut().add_memory_segment().into())
    }

    #[args(apply_modulo_to_args = true)]
    pub fn gen_arg(
        &self,
        py: Python,
        arg: Py<PyAny>,
        apply_modulo_to_args: bool,
    ) -> PyResult<PyObject> {
        Ok(
            PyMaybeRelocatable::from(match PyIterator::from_object(py, &arg) {
                Ok(iterator) => {
                    let segment_ptr = MaybeRelocatable::RelocatableValue(
                        self.vm.borrow_mut().add_memory_segment(),
                    );
                    self.write_arg(
                        py,
                        segment_ptr.clone().into(),
                        iterator.to_object(py),
                        apply_modulo_to_args,
                    )?;
                    segment_ptr
                }
                _ => {
                    let mut value: MaybeRelocatable = arg.extract::<PyMaybeRelocatable>(py)?.into();
                    if apply_modulo_to_args {
                        value = value
                            .mod_floor(self.vm.borrow().get_prime())
                            .map_err(to_py_error)?;
                    }
                    value
                }
            })
            .to_object(py),
        )
    }

    #[args(apply_modulo_to_args = true)]
    pub fn write_arg(
        &self,
        py: Python<'_>,
        ptr: PyMaybeRelocatable,
        arg: Py<PyAny>,
        apply_modulo_to_args: bool,
    ) -> PyResult<PyObject> {
        let ptr: MaybeRelocatable = ptr.into();

        let arg_iter = PyIterator::from_object(py, &arg)?;
        let mut data = Vec::new();
        for value in arg_iter {
            data.push(
                self.gen_arg(py, value?.to_object(py), apply_modulo_to_args)?
                    .extract::<PyMaybeRelocatable>(py)?
                    .into(),
            );
        }

        self.vm
            .borrow_mut()
            .load_data(&ptr, data)
            .map(|x| PyMaybeRelocatable::from(x).to_object(py))
            .map_err(to_py_error)
    }
}

#[cfg(test)]
mod test {
    use std::ops::Add;

    use super::PySegmentManager;
    use crate::{relocatable::PyMaybeRelocatable, vm_core::PyVM};
    use cairo_rs::{bigint, types::relocatable::Relocatable};
    use num_bigint::{BigInt, Sign};
    use pyo3::{Python, ToPyObject};

    #[test]
    fn add_segment_test() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );
        let segments = PySegmentManager::new(&vm);
        assert!(segments.add().is_ok());
    }

    #[test]
    fn write_arg_test() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
            );
            let segments = PySegmentManager::new(&vm);

            let ptr = segments.add().unwrap();
            segments
                .write_arg(
                    py,
                    PyMaybeRelocatable::RelocatableValue(ptr),
                    py.eval("[1, 2, [3, 4], [5, 6]]", None, None)
                        .unwrap()
                        .to_object(py),
                    true,
                )
                .unwrap();

            let vm_ref = vm.get_vm();
            let vm_ref = vm_ref.borrow();

            assert_eq!(
                vm_ref
                    .get_maybe(&Relocatable::from((0, 0)))
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(1),
            );
            assert_eq!(
                vm_ref
                    .get_maybe(&Relocatable::from((0, 1)))
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(2),
            );

            let relocatable = vm_ref
                .get_maybe(&Relocatable::from((0, 2)))
                .unwrap()
                .unwrap()
                .get_relocatable()
                .unwrap()
                .clone();

            assert_eq!(
                vm_ref
                    .get_maybe(&relocatable)
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(3),
            );
            assert_eq!(
                vm_ref
                    .get_maybe(&relocatable.clone().add(1_i32))
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(4),
            );
            assert!(vm_ref
                .get_maybe(&relocatable.clone().add(2_i32))
                .unwrap()
                .is_none());

            let relocatable = vm_ref
                .get_maybe(&Relocatable::from((0, 3)))
                .unwrap()
                .unwrap()
                .get_relocatable()
                .unwrap()
                .clone();

            assert_eq!(
                vm_ref
                    .get_maybe(&relocatable)
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(5),
            );
            assert_eq!(
                vm_ref
                    .get_maybe(&relocatable.clone().add(1_i32))
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(6),
            );
            assert!(vm_ref.get_maybe(&relocatable.add(2_i32)).unwrap().is_none());

            assert!(vm_ref
                .get_maybe(&Relocatable::from((0, 4)))
                .unwrap()
                .is_none());
        });
    }
}
