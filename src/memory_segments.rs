use crate::{
    relocatable::{PyMaybeRelocatable, PyRelocatable},
    utils::to_py_error,
    vm_core::PyVM,
};
use cairo_rs::{types::relocatable::MaybeRelocatable, vm::vm_core::VirtualMachine};
use pyo3::{prelude::*, types::PyIterator};
use std::{borrow::Cow, cell::RefCell, rc::Rc};

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

    pub fn write_arg(
        &self,
        py: Python<'_>,
        ptr: PyMaybeRelocatable,
        arg: Py<PyAny>,
    ) -> PyResult<PyObject> {
        // Recursive function which inserts every integer it finds, while also inserting references
        // to new segments when an iterable is found. Those new segments will also be filled with
        // data using this same function.
        fn write_arg_iter(
            py: Python,
            ptr: &MaybeRelocatable,
            arg_iter: &PyIterator,
            vm: &mut VirtualMachine,
        ) -> PyResult<MaybeRelocatable> {
            let mut ptr = Cow::Borrowed(ptr);
            for value in arg_iter {
                let value = value?;

                ptr = Cow::Owned(match PyIterator::from_object(py, value) {
                    Ok(arg_iter) => {
                        let segment_ptr =
                            MaybeRelocatable::RelocatableValue(vm.add_memory_segment());
                        write_arg_iter(py, &segment_ptr, arg_iter, vm)?;
                        vm.load_data(&ptr, vec![segment_ptr]).map_err(to_py_error)?
                    }
                    Err(_) => {
                        let value = value.extract::<PyMaybeRelocatable>()?;
                        vm.load_data(&ptr, vec![value.to_maybe_relocatable()])
                            .map_err(to_py_error)?
                    }
                });
            }

            Ok(ptr.into_owned())
        }

        let mut vm = self.vm.borrow_mut();
        Ok(PyMaybeRelocatable::from(write_arg_iter(
            py,
            &ptr.to_maybe_relocatable(),
            PyIterator::from_object(py, &arg)?,
            &mut vm,
        )?)
        .to_object(py))
    }
}

#[cfg(test)]
mod test {
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
                .unwrap();

            assert_eq!(
                vm_ref
                    .get_maybe(relocatable)
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(3),
            );
            assert_eq!(
                vm_ref
                    .get_maybe(&relocatable.add(1).unwrap())
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(4),
            );
            assert!(vm_ref
                .get_maybe(&relocatable.add(2).unwrap())
                .unwrap()
                .is_none());

            let relocatable = vm_ref
                .get_maybe(&Relocatable::from((0, 3)))
                .unwrap()
                .unwrap()
                .get_relocatable()
                .unwrap();

            assert_eq!(
                vm_ref
                    .get_maybe(relocatable)
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(5),
            );
            assert_eq!(
                vm_ref
                    .get_maybe(&relocatable.add(1).unwrap())
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(6),
            );
            assert!(vm_ref
                .get_maybe(&relocatable.add(2).unwrap())
                .unwrap()
                .is_none());

            assert!(vm_ref
                .get_maybe(&Relocatable::from((0, 4)))
                .unwrap()
                .is_none());
        });
    }
}
