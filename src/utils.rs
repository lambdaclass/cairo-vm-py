use cairo_rs::vm::errors::vm_errors::VirtualMachineError;
use pyo3::{exceptions::PyValueError, PyAny, PyErr};
use std::{
    fmt::Display,
    io::{self, Read, Write},
};

#[macro_export]
macro_rules! pycell {
    ($py:expr, $val:expr) => {
        PyCell::new($py, $val).map_err(to_vm_error)?
    };
}

pub fn to_vm_error(pyerror: PyErr) -> VirtualMachineError {
    VirtualMachineError::CustomHint(format!("{}", pyerror))
}

pub fn to_py_error<T: Display>(error: T) -> PyErr {
    PyValueError::new_err(format!("{}", error))
}

/// A Rust wrapper around Python IO streams (typing.IO).
pub struct PyIoStream<'a>(pub &'a PyAny);

impl<'a> Read for PyIoStream<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let py_buf = self
            .0
            .call_method1("read", (buf.len(),))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            .extract::<&[u8]>()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        assert!(py_buf.len() <= buf.len());
        buf[..py_buf.len()].clone_from_slice(py_buf);

        Ok(py_buf.len())
    }
}

impl<'a> Write for PyIoStream<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .call_method1("write", (buf,))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            .extract::<usize>()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0
            .call_method0("flush")
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }
}
