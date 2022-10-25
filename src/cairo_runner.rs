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

#[cfg(test)]
mod test {
    use super::*;
    use num_bigint::{BigInt, Sign};

    #[test]
    fn create_cairo_runner() {
        PyCairoRunner::new("cairo_programs/fibonacci.json", "main").unwrap();
    }

    #[test]
    fn initialize_runner() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );

        let mut runner = PyCairoRunner::new("cairo_programs/fibonacci.json", "main").unwrap();
        runner.initialize(&vm).unwrap();
    }

    // TODO: Test get_reference_list().
    // TODO: Test get_data_dictionary().
    // TODO: Test run_until_pc().

    #[test]
    fn runner_relocate() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );

        let mut runner = PyCairoRunner::new("cairo_programs/fibonacci.json", "main").unwrap();
        runner.relocate(&vm).unwrap();
    }

    #[test]
    fn get_output() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        );

        let mut runner = PyCairoRunner::new("cairo_programs/fibonacci.json", "main").unwrap();
        runner.get_output(&vm).unwrap();
    }

    #[test]
    fn write_output() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
            );

            let mut runner = PyCairoRunner::new("cairo_programs/fibonacci.json", "main").unwrap();

            let py_io = py.import("io").unwrap();
            let py_bytes_io_class = py_io.getattr("BytesIO").unwrap();
            let py_stream = py_bytes_io_class.call0().unwrap();

            runner.write_output(&vm, py_stream).unwrap();
        })
    }
}
