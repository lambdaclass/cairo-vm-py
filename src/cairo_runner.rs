use crate::{
    relocatable::PyRelocatable,
    utils::{to_py_error, PyIoStream},
    vm_core::PyVM,
};
use cairo_rs::{types::program::Program, vm::runners::cairo_runner::CairoRunner};
use pyo3::prelude::*;
use std::{ops::DerefMut, path::Path};

#[pyclass(unsendable)]
pub struct PyCairoRunner {
    inner: CairoRunner,
}

#[pymethods]
impl PyCairoRunner {
    #[new]
    fn new(path: &str, entrypoint: &str) -> PyResult<Self> {
        let program = Program::new(Path::new(path), entrypoint).map_err(to_py_error)?;
        let cairo_runner = CairoRunner::new(&program).map_err(to_py_error)?;

        Ok(PyCairoRunner {
            inner: cairo_runner,
        })
    }

    fn initialize(&mut self, vm: &PyVM) -> PyResult<PyRelocatable> {
        let mut vm_ref = vm.vm.as_ref().borrow_mut();

        self.inner
            .initialize(vm_ref.deref_mut())
            .map(PyRelocatable::from)
            .map_err(to_py_error)
    }

    // TODO: get_reference_list(): HintReference in Python?
    // TODO: get_data_dictionary(): HintReference in Python?
    // TODO: run_until_pc(): HintProcessor in Python?

    fn relocate(&mut self, vm: &PyVM) -> PyResult<()> {
        let mut vm_ref = vm.vm.as_ref().borrow_mut();

        self.inner.relocate(vm_ref.deref_mut()).map_err(to_py_error)
    }

    fn get_output(&mut self, vm: &PyVM) -> PyResult<Option<String>> {
        let mut vm_ref = vm.vm.as_ref().borrow_mut();

        self.inner
            .get_output(vm_ref.deref_mut())
            .map_err(to_py_error)
    }

    fn write_output(&mut self, vm: &PyVM, stdout: &PyAny) -> PyResult<()> {
        let mut stdout = PyIoStream(stdout);
        let mut vm_ref = vm.vm.as_ref().borrow_mut();

        self.inner
            .write_output(vm_ref.deref_mut(), &mut stdout)
            .map_err(to_py_error)
    }
}
