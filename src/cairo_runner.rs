use crate::{
    relocatable::PyRelocatable,
    utils::{to_py_error, PyIoStream},
    vm_core::PyVM,
};
use cairo_rs::{types::program::Program, vm::runners::cairo_runner::CairoRunner};
use num_bigint::{BigInt, Sign};
use pyo3::prelude::*;
use std::path::Path;

#[pyclass(unsendable)]
pub struct PyCairoRunner {
    inner: CairoRunner,
    pyvm: PyVM,
}

#[pymethods]
impl PyCairoRunner {
    #[new]
    fn new(path: &str, entrypoint: &str) -> PyResult<Self> {
        let program = Program::new(Path::new(path), entrypoint).map_err(to_py_error)?;
        let cairo_runner = CairoRunner::new(&program).map_err(to_py_error)?;

        Ok(PyCairoRunner {
            inner: cairo_runner,
            pyvm: PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
            ),
        })
    }

    fn initialize(&mut self) -> PyResult<PyRelocatable> {
        self.inner
            .initialize(&mut self.pyvm.vm.borrow_mut())
            .map(PyRelocatable::from)
            .map_err(to_py_error)
    }

    // TODO: get_reference_list(): HintReference in Python?
    // TODO: get_data_dictionary(): HintReference in Python?
    // TODO: run_until_pc(): HintProcessor in Python?

    fn relocate(&mut self) -> PyResult<()> {
        self.inner
            .relocate(&mut self.pyvm.vm.borrow_mut())
            .map_err(to_py_error)
    }

    fn get_output(&mut self) -> PyResult<Option<String>> {
        self.inner
            .get_output(&mut self.pyvm.vm.borrow_mut())
            .map_err(to_py_error)
    }

    fn write_output(&mut self, stdout: &PyAny) -> PyResult<()> {
        let mut stdout = PyIoStream(stdout);

        self.inner
            .write_output(&mut self.pyvm.vm.borrow_mut(), &mut stdout)
            .map_err(to_py_error)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_cairo_runner() {
        PyCairoRunner::new("cairo_programs/fibonacci.json", "main").unwrap();
    }

    #[test]
    fn initialize_runner() {
        let mut runner = PyCairoRunner::new("cairo_programs/fibonacci.json", "main").unwrap();
        runner.initialize().unwrap();
    }

    // TODO: Test get_reference_list().
    // TODO: Test get_data_dictionary().
    // TODO: Test run_until_pc().

    #[test]
    fn runner_relocate() {
        let mut runner = PyCairoRunner::new("cairo_programs/fibonacci.json", "main").unwrap();
        runner.relocate().unwrap();
    }

    #[test]
    fn get_output() {
        let mut runner = PyCairoRunner::new("cairo_programs/fibonacci.json", "main").unwrap();
        runner.get_output().unwrap();
    }

    #[test]
    fn write_output() {
        Python::with_gil(|py| {
            let mut runner = PyCairoRunner::new("cairo_programs/fibonacci.json", "main").unwrap();

            let py_io = py.import("io").unwrap();
            let py_bytes_io_class = py_io.getattr("BytesIO").unwrap();
            let py_stream = py_bytes_io_class.call0().unwrap();

            runner.write_output(py_stream).unwrap();
        })
    }
}
