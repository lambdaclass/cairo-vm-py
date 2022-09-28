use std::{cell::RefCell, collections::HashMap, rc::Rc};

use cairo_rs::{
    hint_processor::{
        hint_processor_definition::HintReference, hint_processor_utils::bigint_to_usize,
    },
    serde::deserialize_program::ApTracking,
    types::{
        instruction::Register,
        relocatable::{MaybeRelocatable, Relocatable},
    },
    vm::{errors::vm_errors::VirtualMachineError, vm_core::VirtualMachine},
};
use pyo3::{pyclass, pymethods, PyObject, PyResult, Python, ToPyObject};

use crate::{
    relocatable::PyMaybeRelocatable,
    utils::to_py_error,
    vm_core::PyVM,
};

const IDS_GET_ERROR_MSG: &str = "Failed to get ids value";
const IDS_SET_ERROR_MSG: &str = "Failed to set ids value to Cairo memory";

#[pyclass(unsendable)]
pub struct PyIds {
    vm: Rc<RefCell<VirtualMachine>>,
    references: HashMap<String, HintReference>,
    ap_tracking: ApTracking,
}

#[pymethods]
impl PyIds {
    #[getter]
    pub fn __getattr__(&self, name: String, py: Python) -> PyResult<PyObject> {
        let hint_ref = self
            .references
            .get(&name)
            .ok_or(to_py_error(IDS_GET_ERROR_MSG))?;
        Ok(get_value_from_reference(&self.vm.borrow(), hint_ref, &self.ap_tracking)?.to_object(py))
    }

    pub fn __setattr__(&self, name: String, val: PyMaybeRelocatable) -> PyResult<()> {
        let hint_ref = self
            .references
            .get(&name)
            .ok_or(to_py_error(IDS_SET_ERROR_MSG))?;
        let var_addr = compute_addr_from_reference(hint_ref, &self.vm.borrow(), &self.ap_tracking)?;
        self.vm
            .borrow_mut()
            .memory
            .insert(&var_addr, &val)
            .map_err(to_py_error)
    }
}

impl PyIds {
    pub fn new(
        vm: &PyVM,
        references: &HashMap<String, HintReference>,
        ap_tracking: &ApTracking,
    ) -> PyIds {
        PyIds {
            vm: vm.get_vm(),
            references: references.clone(),
            ap_tracking: ap_tracking.clone(),
        }
    }
}

///Returns the Value given by a reference as an Option<MaybeRelocatable>
pub fn get_value_from_reference(
    vm: &VirtualMachine,
    hint_reference: &HintReference,
    ap_tracking: &ApTracking,
) -> PyResult<PyMaybeRelocatable> {
    //First handle case on only immediate
    if let (None, Some(num)) = (
        hint_reference.register.as_ref(),
        hint_reference.immediate.as_ref(),
    ) {
        return Ok(PyMaybeRelocatable::from(num));
    }
    //Then calculate address
    let var_addr = compute_addr_from_reference(hint_reference, &vm, ap_tracking)?;
    let value = if hint_reference.dereference {
        vm.memory.get(&var_addr).map_err(to_py_error)?
    } else {
        return Ok(PyMaybeRelocatable::from(var_addr));
    };
    match &value {
        Some(&MaybeRelocatable::RelocatableValue(ref rel)) => {
            if let Some(immediate) = &hint_reference.immediate {
                let modified_value = rel + bigint_to_usize(immediate).map_err(to_py_error)?;
                Ok(PyMaybeRelocatable::from(modified_value))
            } else {
                Ok(PyMaybeRelocatable::from(rel.clone()))
            }
        }
        Some(&MaybeRelocatable::Int(ref num)) => Ok(PyMaybeRelocatable::Int(num.clone())),
        None => Err(to_py_error(VirtualMachineError::FailedToGetIds)),
    }
}

///Computes the memory address of the ids variable indicated by the HintReference as a Relocatable
pub fn compute_addr_from_reference(
    //Reference data of the ids variable
    hint_reference: &HintReference,
    vm: &VirtualMachine,
    //ApTracking of the Hint itself
    hint_ap_tracking: &ApTracking,
) -> PyResult<Relocatable> {
    let base_addr = match hint_reference.register {
        //This should never fail
        Some(Register::FP) => vm.get_fp(),
        Some(Register::AP) => {
            let var_ap_trackig = hint_reference
                .ap_tracking_data
                .as_ref()
                .ok_or(VirtualMachineError::NoneApTrackingData)
                .map_err(to_py_error)?;

            let ap = vm.get_ap();

            apply_ap_tracking_correction(&ap, var_ap_trackig, hint_ap_tracking)
                .map_err(to_py_error)?
        }
        None => return Err(to_py_error(VirtualMachineError::NoRegisterInReference)),
    };
    if hint_reference.offset1.is_negative()
        && base_addr.offset < hint_reference.offset1.abs() as usize
    {
        return Err(to_py_error(VirtualMachineError::FailedToGetIds));
    }
    if !hint_reference.inner_dereference {
        Ok(base_addr + hint_reference.offset1 + hint_reference.offset2)
    } else {
        let addr = base_addr + hint_reference.offset1;
        let dereferenced_addr = vm.memory.get_relocatable(&addr).map_err(to_py_error)?;
        if let Some(imm) = &hint_reference.immediate {
            Ok(dereferenced_addr + bigint_to_usize(imm).map_err(to_py_error)?)
        } else {
            Ok(dereferenced_addr + hint_reference.offset2)
        }
    }
}

//TODO: Make this function public and import it from cairo-rs
fn apply_ap_tracking_correction(
    ap: &Relocatable,
    ref_ap_tracking: &ApTracking,
    hint_ap_tracking: &ApTracking,
) -> Result<Relocatable, VirtualMachineError> {
    // check that both groups are the same
    if ref_ap_tracking.group != hint_ap_tracking.group {
        return Err(VirtualMachineError::InvalidTrackingGroup(
            ref_ap_tracking.group,
            hint_ap_tracking.group,
        ));
    }
    let ap_diff = hint_ap_tracking.offset - ref_ap_tracking.offset;
    ap.sub(ap_diff)
}

#[cfg(test)]
mod tests {
    use cairo_rs::bigint;
    use num_bigint::{BigInt, Sign};
    use pyo3::{types::PyDict, PyCell};

    use crate::{memory::PyMemory, utils::to_vm_error, relocatable::PyRelocatable};

    use super::*;

    #[test]
    fn ids_get_test() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }
            //Create references
            let mut references = HashMap::new();
            references.insert(String::from("a"), HintReference::new_simple(1));
            //Insert ids.a into memory
            vm.vm
                .borrow_mut()
                .memory
                .insert(
                    &Relocatable::from((1, 1)),
                    &MaybeRelocatable::from(Into::<BigInt>::into(2_i32)),
                )
                .unwrap();

            let memory = PyMemory::new(&vm);
            let fp = PyRelocatable::from((1, 0));
            let ids = PyIds::new(&vm, &references, &ApTracking::default());

            let globals = PyDict::new(py);
            globals
                .set_item("memory", PyCell::new(py, memory).unwrap())
                .unwrap();
            globals
                .set_item("fp", PyCell::new(py, fp).unwrap())
                .unwrap();
            globals
                .set_item("ids", PyCell::new(py, ids).unwrap())
                .unwrap();

            let code = "memory[fp] = ids.a";

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(to_vm_error), Ok(()));
            //Check ids.a is now at memory[fp]
            assert_eq!(
                vm.vm.borrow().memory.get(&Relocatable::from((1, 0))),
                Ok(Some(&MaybeRelocatable::from(Into::<BigInt>::into(2_i32))))
            );
        });
    }

    #[test]
    fn ids_set_test() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }
            //Create references
            let mut references = HashMap::new();
            references.insert(String::from("a"), HintReference::new_simple(1));

            vm.vm
                .borrow_mut()
                .memory
                .insert(
                    &Relocatable::from((1, 0)),
                    &MaybeRelocatable::from(bigint!(2)),
                )
                .unwrap();

            let memory = PyMemory::new(&vm);
            let fp = PyRelocatable::from((1, 0));
            let ids = PyIds::new(&vm, &references, &ApTracking::default());

            let globals = PyDict::new(py);
            globals
                .set_item("memory", PyCell::new(py, memory).unwrap())
                .unwrap();
            globals
                .set_item("fp", PyCell::new(py, fp).unwrap())
                .unwrap();
            globals
                .set_item("ids", PyCell::new(py, ids).unwrap())
                .unwrap();

            let code = "ids.a = memory[fp]";

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(to_vm_error), Ok(()));
            //Check ids.a now contains memory[fp]
            assert_eq!(
                vm.vm.borrow().memory.get(&Relocatable::from((1, 1))),
                Ok(Some(&MaybeRelocatable::from(bigint!(2))))
            );
        });
    }
}
