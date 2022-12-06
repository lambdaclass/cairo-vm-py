use std::collections::HashMap;

use cairo_rs::{
    types::relocatable::Relocatable,
    vm::{errors::vm_errors::VirtualMachineError, runners::builtin_runner::SignatureBuiltinRunner},
};

use num_bigint::BigInt;
use pyo3::prelude::*;

use crate::relocatable::PyRelocatable;

#[pyclass(name = "Signature")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PySignature {
    signatures: HashMap<PyRelocatable, (BigInt, BigInt)>,
}

#[pymethods]
impl PySignature {
    #[new]
    pub fn new() -> Self {
        Self {
            signatures: HashMap::new(),
        }
    }

    pub fn add_signature(&mut self, address: PyRelocatable, pair: (BigInt, BigInt)) {
        self.signatures.insert(address, pair);
    }
}

impl PySignature {
    pub fn update_signature(
        &self,
        signature_builtin: &mut SignatureBuiltinRunner,
    ) -> Result<(), VirtualMachineError> {
        for (address, pair) in self.signatures.iter() {
            signature_builtin
                .add_signature(Relocatable::from(address), pair)
                .map_err(VirtualMachineError::MemoryError)?
        }
        Ok(())
    }
}

impl Default for PySignature {
    fn default() -> Self {
        Self::new()
    }
}

impl ToPyObject for PySignature {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.clone().into_py(py)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::relocatable::PyRelocatable;
    use num_bigint::{BigInt, Sign};

    use crate::cairo_runner::PyCairoRunner;

    use std::fs;

    #[test]
    fn create_empty_py_signature() {
        PySignature::new();
    }

    #[test]
    fn add_py_signature() {
        let rel = PyRelocatable {
            segment_index: 2,
            offset: 0,
        };

        let numbers = (
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
        );

        let mut signature = PySignature::new();

        signature.add_signature(rel, numbers);
    }

    #[test]
    fn update_py_signature() {
        let rel = PyRelocatable {
            segment_index: 2,
            offset: 0,
        };

        let numbers = (
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
        );

        let mut signature = PySignature::new();

        signature.add_signature(rel, numbers);

        let path = "cairo_programs/ecdsa.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("all".to_string()),
            false,
        )
        .unwrap();

        if runner.pyvm.vm.borrow_mut().get_signature_builtin().is_ok() {
            signature
                .update_signature(runner.pyvm.vm.borrow_mut().get_signature_builtin().unwrap());
        }
    }

    #[test]

    fn py_signature_to_py_object() {
        let new_py_signature = PySignature::new();

        Python::with_gil(|py| {
            let py_object = new_py_signature
                .to_object(py)
                .extract::<PySignature>(py)
                .unwrap();

            assert_eq!(py_object, PySignature::new());
        });
    }
}
