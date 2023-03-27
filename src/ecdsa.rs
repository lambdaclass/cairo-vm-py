use num_bigint::BigUint;
use std::collections::HashMap;

use cairo_vm::{
    types::relocatable::Relocatable,
    vm::{errors::vm_errors::VirtualMachineError, runners::builtin_runner::SignatureBuiltinRunner},
};

use cairo_felt::Felt252;
use pyo3::prelude::*;

use crate::relocatable::PyRelocatable;

#[pyclass(name = "Signature")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PySignature {
    signatures: HashMap<PyRelocatable, (Felt252, Felt252)>,
}

#[pymethods]
impl PySignature {
    #[new]
    pub fn new() -> Self {
        Self {
            signatures: HashMap::new(),
        }
    }

    pub fn add_signature(&mut self, address: PyRelocatable, pair: (BigUint, BigUint)) {
        self.signatures
            .insert(address, (pair.0.into(), pair.1.into()));
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
                .map_err(VirtualMachineError::Memory)?
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
    use crate::cairo_runner::PyCairoRunner;
    use crate::relocatable::PyRelocatable;

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
            BigUint::new(vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            BigUint::new(vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
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
            BigUint::new(vec![1, 0, 0, 0, 0, 0, 17, 13421772]),
            BigUint::new(vec![1, 0, 0, 0, 0, 0, 17, 13421772]),
        );

        let mut signature = PySignature::new();
        let original_signature = signature.clone();

        signature.add_signature(rel, numbers);

        let path = "cairo_programs/ecdsa.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("all_cairo".to_string()),
            false,
        )
        .unwrap();

        runner.initialize().expect("Failed to initialize VM");

        let mut binding = runner.pyvm.vm.borrow_mut();
        let signature_builtin = binding.get_signature_builtin().unwrap();

        assert!(signature.update_signature(signature_builtin).is_ok());

        assert_ne!(original_signature.signatures, signature.signatures);
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

    #[test]
    fn py_signature_default() {
        let new_py_signature = PySignature::default();
        let empty_signatures = HashMap::new();

        assert_eq!(new_py_signature.signatures, empty_signatures);
    }
}
