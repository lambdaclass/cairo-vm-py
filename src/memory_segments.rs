use crate::{
    memory::PyMemory,
    relocatable::{PyMaybeRelocatable, PyRelocatable},
    utils::to_py_error,
    vm_core::PyVM,
};
use cairo_vm::{types::relocatable::MaybeRelocatable, vm::vm_core::VirtualMachine};
use pyo3::{prelude::*, types::PyIterator};
use std::{cell::RefCell, rc::Rc};

#[pyclass(name = "MemorySegmentManager", unsendable)]
pub struct PySegmentManager {
    vm: Rc<RefCell<VirtualMachine>>,
    #[pyo3(get)]
    memory: PyMemory,
}

#[pymethods]
impl PySegmentManager {
    #[new]
    pub fn new(vm: &PyVM, memory: PyMemory) -> PySegmentManager {
        PySegmentManager {
            vm: vm.get_vm(),
            memory,
        }
    }

    pub fn add(&self) -> PyResult<PyRelocatable> {
        Ok(self.vm.borrow_mut().add_memory_segment().into())
    }

    #[pyo3(signature = (arg, apply_modulo_to_args = true))]
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
                _ => arg.extract::<PyMaybeRelocatable>(py)?.into(),
            })
            .to_object(py),
        )
    }

    #[pyo3(signature = (ptr, arg, apply_modulo_to_args = true))]
    pub fn write_arg(
        &self,
        py: Python<'_>,
        ptr: PyMaybeRelocatable,
        arg: Py<PyAny>,
        apply_modulo_to_args: bool,
    ) -> PyResult<PyObject> {
        let ptr: MaybeRelocatable = ptr.into();

        let arg_iter = PyIterator::from_object(py, &arg)?;
        let mut data = Vec::<MaybeRelocatable>::new();
        for value in arg_iter {
            data.push(
                self.gen_arg(py, value?.to_object(py), apply_modulo_to_args)?
                    .extract::<PyMaybeRelocatable>(py)?
                    .into(),
            );
        }

        let pointer = ptr
            .get_relocatable()
            .ok_or_else(|| to_py_error("Invalid pointer"))?;

        self.vm
            .borrow_mut()
            .load_data(pointer, &data)
            .map(|x| PyMaybeRelocatable::from(x).to_object(py))
            .map_err(to_py_error)
    }

    pub fn add_temp_segment(&mut self) -> PyResult<PyRelocatable> {
        Ok(PyRelocatable::from(
            self.vm.borrow_mut().add_temporary_segment(),
        ))
    }

    pub fn get_segment_used_size(&self, segment_index: usize) -> Option<usize> {
        (*self.vm).borrow().get_segment_used_size(segment_index)
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Borrow;

    use super::PySegmentManager;
    use crate::{memory::PyMemory, relocatable::PyMaybeRelocatable, vm_core::PyVM};
    use cairo_felt::Felt;
    use cairo_vm::types::relocatable::{MaybeRelocatable, Relocatable};
    use pyo3::{Python, ToPyObject};

    #[test]
    fn add_segment_test() {
        let vm = PyVM::new(false);
        let segments = PySegmentManager::new(&vm, PyMemory::new(&vm));
        assert!(segments.add().is_ok());
    }

    #[test]
    fn write_arg_test() {
        Python::with_gil(|py| {
            let vm = PyVM::new(false);
            let segments = PySegmentManager::new(&vm, PyMemory::new(&vm));

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

            let vm_ref = (*(vm.vm)).borrow();
            let vm_ref = vm_ref.borrow();

            assert_eq!(
                vm_ref
                    .get_maybe(&Relocatable::from((0, 0)))
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &Felt::new(1),
            );
            assert_eq!(
                vm_ref
                    .get_maybe(&Relocatable::from((0, 1)))
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &Felt::new(2),
            );

            let relocatable = vm_ref
                .get_maybe(&Relocatable::from((0, 2)))
                .unwrap()
                .get_relocatable()
                .unwrap();

            assert_eq!(
                vm_ref
                    .get_maybe(&relocatable)
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &Felt::new(3),
            );
            assert_eq!(
                vm_ref
                    .get_maybe(&(&relocatable + 1))
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &Felt::new(4),
            );
            assert!(vm_ref.get_maybe(&(&relocatable + 2)).is_none());

            let relocatable = vm_ref
                .get_maybe(&Relocatable::from((0, 3)))
                .unwrap()
                .get_relocatable()
                .unwrap();

            assert_eq!(
                vm_ref
                    .get_maybe(&relocatable)
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &Felt::new(5),
            );
            assert_eq!(
                vm_ref
                    .get_maybe(&(&relocatable + 1))
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &Felt::new(6),
            );
            assert!(vm_ref.get_maybe(&(&relocatable + 2)).is_none());

            assert!(vm_ref.get_maybe(&Relocatable::from((0, 4))).is_none());
        });
    }

    #[test]
    fn add_temp_segment_test() {
        let mut vm = PyVM::new(false);
        let memory = PyMemory::new(&vm);
        let mut segments = PySegmentManager::new(&mut vm, memory);
        assert!(segments.add_temp_segment().is_ok());
    }

    #[test]
    fn get_segment_used_size() {
        let vm = PyVM::new(false);

        let memory = PyMemory::new(&vm);
        let segments = PySegmentManager::new(&vm, memory);

        let segment = segments.add().expect("Unable to add a new segment.");
        assert!((*(vm.vm))
            .borrow_mut()
            .load_data(
                Relocatable::from(&segment).into(),
                &vec![
                    MaybeRelocatable::from(1),
                    MaybeRelocatable::from(2),
                    MaybeRelocatable::from(3),
                    MaybeRelocatable::from(4),
                ],
            )
            .is_ok());
        (*(vm.vm)).borrow_mut().segments.compute_effective_sizes();

        assert_eq!(
            segments.get_segment_used_size(segment.segment_index as _),
            Some(4),
        );
    }
}
