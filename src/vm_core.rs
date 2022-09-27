use cairo_rs::vm::{runners::builtin_runner::BuiltinRunner, vm_core::VirtualMachine};
use num_bigint::BigInt;
use pyo3::{pyclass, pymethods};
use std::{cell::RefCell, rc::Rc};

#[pyclass(unsendable)]
pub struct PyVM {
    vm: Rc<RefCell<VirtualMachine>>,
}

#[pymethods]
impl PyVM {
    #[new]
    pub fn new(prime: BigInt, trace_enabled: bool) -> PyVM {
        PyVM {
            vm: Rc::new(RefCell::new(VirtualMachine::new(
                prime,
                Vec::new(),
                trace_enabled,
            ))),
        }
    }
}

impl PyVM {
    pub(crate) fn get_vm(&self) -> Rc<RefCell<VirtualMachine>> {
        Rc::clone(&self.vm)
    }
}
