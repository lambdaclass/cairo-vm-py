use crate::{
    ids::PyTypedId,
    memory_segments::PySegmentManager,
    relocatable::{PyMaybeRelocatable, PyRelocatable},
    utils::to_py_error,
};
use cairo_rs::{
    hint_processor::builtin_hint_processor::dict_manager::DictManager,
    types::relocatable::Relocatable,
};
use num_bigint::BigInt;
use pyo3::{exceptions::PyKeyError, prelude::*};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[pyclass(unsendable)]
pub struct PyDictManager {
    manager: Rc<RefCell<DictManager>>,
}

#[pyclass(unsendable)]
pub struct PyDictTracker {
    manager: Rc<RefCell<DictManager>>,
    key: Relocatable,
}

impl Default for PyDictManager {
    fn default() -> Self {
        PyDictManager::new()
    }
}

#[pymethods]
impl PyDictManager {
    #[new]
    pub fn new() -> Self {
        PyDictManager {
            manager: Rc::new(RefCell::new(DictManager::new())),
        }
    }

    pub fn new_dict(
        &self,
        segments: &mut PySegmentManager,
        initial_dict: HashMap<BigInt, BigInt>,
        py: Python,
    ) -> PyResult<PyObject> {
        Ok(PyMaybeRelocatable::from(
            self.manager
                .borrow_mut()
                .new_dict(&mut segments.vm.borrow_mut(), initial_dict)
                .map_err(to_py_error)?,
        )
        .to_object(py))
    }

    pub fn new_default_dict(
        &mut self,
        segments: &mut PySegmentManager,
        default_value: BigInt,
        initial_dict: Option<HashMap<BigInt, BigInt>>,
        py: Python,
    ) -> PyResult<PyObject> {
        Ok(PyMaybeRelocatable::from(
            self.manager
                .borrow_mut()
                .new_default_dict(&mut segments.vm.borrow_mut(), &default_value, initial_dict)
                .map_err(to_py_error)?,
        )
        .to_object(py))
    }

    pub fn get_tracker(&mut self, dict_ptr: &PyTypedId) -> PyResult<PyDictTracker> {
        let ptr_addr = dict_ptr.hint_value.clone();
        self.manager
            .borrow()
            .get_tracker(&ptr_addr)
            .map_err(to_py_error)?;
        Ok(PyDictTracker {
            manager: self.manager.clone(),
            key: ptr_addr,
        })
    }
}

#[pymethods]
impl PyDictTracker {
    #[getter]
    pub fn get_current_ptr(&self, py: Python) -> PyResult<PyObject> {
        Ok(PyRelocatable::from(
            self.manager
                .borrow_mut()
                .get_tracker_mut(&self.key)
                .map_err(to_py_error)?
                .current_ptr
                .clone(),
        )
        .into_py(py))
    }

    #[getter]
    pub fn get_data(&self, py: Python) -> PyObject {
        PyDictTracker {
            manager: self.manager.clone(),
            key: self.key.clone(),
        }
        .into_py(py)
    }

    #[setter]
    pub fn set_current_ptr(&mut self, val: PyRelocatable) -> PyResult<()> {
        self.manager
            .borrow_mut()
            .get_tracker_mut(&self.key)
            .map_err(to_py_error)?
            .current_ptr
            .offset = val.offset;
        self.key = Relocatable {
            segment_index: val.segment_index,
            offset: val.offset,
        };
        Ok(())
    }

    #[getter]
    pub fn __getitem__(&self, key: PyMaybeRelocatable, py: Python) -> PyResult<PyObject> {
        match key {
            PyMaybeRelocatable::Int(key) => Ok(PyMaybeRelocatable::from(
                self.manager
                    .borrow_mut()
                    .get_tracker_mut(&self.key)
                    .map_err(to_py_error)?
                    .get_value(&key)
                    .map_err(to_py_error)?,
            )
            .to_object(py)),
            PyMaybeRelocatable::RelocatableValue(_) => Err(PyKeyError::new_err(key.to_object(py))),
        }
    }

    #[setter]
    pub fn __setitem__(
        &mut self,
        key: PyMaybeRelocatable,
        val: PyMaybeRelocatable,
        py: Python,
    ) -> PyResult<()> {
        match (&key, &val) {
            (PyMaybeRelocatable::Int(key), PyMaybeRelocatable::Int(val)) => {
                self.manager
                    .borrow_mut()
                    .get_tracker_mut(&self.key)
                    .map_err(to_py_error)?
                    .insert_value(key, val);

                Ok(())
            }
            _ => Err(PyKeyError::new_err(key.to_object(py))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ids::PyIds, memory::PyMemory, utils::to_vm_error, vm_core::PyVM};
    use cairo_rs::{
        hint_processor::hint_processor_definition::HintReference,
        serde::deserialize_program::{ApTracking, Member},
        types::relocatable::Relocatable,
        types::{instruction::Register, relocatable::MaybeRelocatable},
    };
    use num_bigint::{BigInt, Sign};
    use pyo3::{types::PyDict, PyCell};

    use super::*;

    #[test]
    fn new_dict() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }

            let dict_manager = PyDictManager::default();

            let memory = PyMemory::new(&vm);
            let ap = PyRelocatable::from(vm.vm.borrow().get_ap());
            let segment_manager = PySegmentManager::new(&vm, PyMemory::new(&vm));

            let globals = PyDict::new(py);
            globals
                .set_item("memory", PyCell::new(py, memory).unwrap())
                .unwrap();
            globals
                .set_item("ap", PyCell::new(py, ap).unwrap())
                .unwrap();
            globals
                .set_item("dict_manager", PyCell::new(py, dict_manager).unwrap())
                .unwrap();
            globals
                .set_item("segments", PyCell::new(py, segment_manager).unwrap())
                .unwrap();

            let code = r#"
memory[ap] = dict_manager.new_dict(segments, {})
memory[ap + 1] = dict_manager.new_dict(segments, {})
"#;

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(to_vm_error), Ok(()));

            let mb_relocatable = vm.vm.borrow().get_maybe(&Relocatable::from((1, 0)));
            assert_eq!(
                mb_relocatable,
                Ok(Some(MaybeRelocatable::RelocatableValue(Relocatable::from(
                    (2, 0)
                ))))
            );
            let mb_relocatable = vm.vm.borrow().get_maybe(&Relocatable::from((1, 1)));
            assert_eq!(
                mb_relocatable,
                Ok(Some(MaybeRelocatable::RelocatableValue(Relocatable::from(
                    (3, 0)
                ))))
            );
        });
    }

    #[test]
    fn tracker_read() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }

            let dict_manager = PyDictManager::default();

            let segment_manager = PySegmentManager::new(&vm, PyMemory::new(&vm));

            //Create references
            let mut references = HashMap::new();
            references.insert(
                String::from("dict"),
                HintReference {
                    register: Some(Register::FP),
                    offset1: 0,
                    offset2: 0,
                    inner_dereference: false,
                    ap_tracking_data: None,
                    immediate: None,
                    dereference: true,
                    cairo_type: Some(String::from("DictAccess*")),
                },
            );

            let mut struct_types: HashMap<String, HashMap<String, Member>> = HashMap::new();
            struct_types.insert(String::from("DictAccess"), HashMap::new());

            let ids = PyIds::new(
                &vm,
                &references,
                &ApTracking::default(),
                &HashMap::new(),
                Rc::new(struct_types),
            );

            let globals = PyDict::new(py);
            globals
                .set_item("dict_manager", PyCell::new(py, dict_manager).unwrap())
                .unwrap();
            globals
                .set_item("ids", PyCell::new(py, ids).unwrap())
                .unwrap();
            globals
                .set_item("segments", PyCell::new(py, segment_manager).unwrap())
                .unwrap();

            let code = r#"
initial_dict = { 1: 2, 4: 8, 16: 32 }
ids.dict = dict_manager.new_dict(segments, initial_dict)
dict_tracker = dict_manager.get_tracker(ids.dict)
assert dict_tracker.data[1] == 2
assert dict_tracker.data[4] == 8
assert dict_tracker.data[16] == 32
"#;

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(to_vm_error), Ok(()));
        });
    }

    #[test]
    fn tracker_read_default_dict() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }

            let dict_manager = PyDictManager::default();

            let segment_manager = PySegmentManager::new(&vm, PyMemory::new(&vm));

            let mut references = HashMap::new();

            // Create reference with type DictAccess*
            references.insert(
                String::from("dict"),
                HintReference {
                    register: Some(Register::FP),
                    offset1: 0,
                    offset2: 0,
                    inner_dereference: false,
                    ap_tracking_data: None,
                    immediate: None,
                    dereference: true,
                    cairo_type: Some(String::from("DictAccess*")),
                },
            );

            let mut struct_types: HashMap<String, HashMap<String, Member>> = HashMap::new();

            // Create dummy type DictAccess
            struct_types.insert(String::from("DictAccess"), HashMap::new());

            let ids = PyIds::new(
                &vm,
                &references,
                &ApTracking::default(),
                &HashMap::new(),
                Rc::new(struct_types),
            );

            let globals = PyDict::new(py);
            globals
                .set_item("dict_manager", PyCell::new(py, dict_manager).unwrap())
                .unwrap();
            globals
                .set_item("ids", PyCell::new(py, ids).unwrap())
                .unwrap();
            globals
                .set_item("segments", PyCell::new(py, segment_manager).unwrap())
                .unwrap();

            let code = r#"
ids.dict = dict_manager.new_default_dict(segments, 42, {})
dict_tracker = dict_manager.get_tracker(ids.dict)
assert dict_tracker.data[33] == 42
assert dict_tracker.data[223] == 42
assert dict_tracker.data[412] == 42
"#;

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(to_vm_error), Ok(()));
        });
    }

    #[test]
    fn tracker_write() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }

            let dict_manager = PyDictManager::default();

            let segment_manager = PySegmentManager::new(&vm, PyMemory::new(&vm));

            //Create references
            let mut references = HashMap::new();
            references.insert(
                String::from("dict"),
                HintReference {
                    register: Some(Register::FP),
                    offset1: 0,
                    offset2: 0,
                    inner_dereference: false,
                    ap_tracking_data: None,
                    immediate: None,
                    dereference: true,
                    cairo_type: Some(String::from("DictAccess*")),
                },
            );

            let mut struct_types: HashMap<String, HashMap<String, Member>> = HashMap::new();
            struct_types.insert(String::from("DictAccess"), HashMap::new());

            let ids = PyIds::new(
                &vm,
                &references,
                &ApTracking::default(),
                &HashMap::new(),
                Rc::new(struct_types),
            );

            let globals = PyDict::new(py);
            globals
                .set_item("dict_manager", PyCell::new(py, dict_manager).unwrap())
                .unwrap();
            globals
                .set_item("ids", PyCell::new(py, ids).unwrap())
                .unwrap();
            globals
                .set_item("segments", PyCell::new(py, segment_manager).unwrap())
                .unwrap();

            let code = r#"
ids.dict = dict_manager.new_dict(segments, {})
dict_tracker = dict_manager.get_tracker(ids.dict)

dict_tracker.data[1] = 5
assert dict_tracker.data[1] == 5

dict_tracker.data[1] = 22
assert dict_tracker.data[1] == 22
"#;

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(to_vm_error), Ok(()));
        });
    }

    #[test]
    fn tracker_get_and_set_current_ptr() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }

            let dict_manager = PyDictManager::default();

            let segment_manager = PySegmentManager::new(&vm, PyMemory::new(&vm));

            let mut references = HashMap::new();

            // Inserts `start_ptr` on references and memory
            references.insert(String::from("start_ptr"), HintReference::new_simple(0));
            vm.vm
                .borrow_mut()
                .insert_value(&Relocatable::from((1, 0)), &MaybeRelocatable::from((2, 0)))
                .unwrap();

            // Inserts `end_ptr` on references and memory
            references.insert(String::from("end_ptr"), HintReference::new_simple(1));
            vm.vm
                .borrow_mut()
                .insert_value(&Relocatable::from((1, 1)), &MaybeRelocatable::from((2, 1)))
                .unwrap();

            // Create reference with type DictAccess*
            references.insert(
                String::from("dict"),
                HintReference {
                    register: Some(Register::FP),
                    offset1: 2,
                    offset2: 0,
                    inner_dereference: false,
                    ap_tracking_data: None,
                    immediate: None,
                    dereference: true,
                    cairo_type: Some(String::from("DictAccess*")),
                },
            );

            let mut struct_types: HashMap<String, HashMap<String, Member>> = HashMap::new();

            // Create dummy type DictAccess
            struct_types.insert(String::from("DictAccess"), HashMap::new());

            let ids = PyIds::new(
                &vm,
                &references,
                &ApTracking::default(),
                &HashMap::new(),
                Rc::new(struct_types),
            );

            let globals = PyDict::new(py);
            globals
                .set_item("dict_manager", PyCell::new(py, dict_manager).unwrap())
                .unwrap();
            globals
                .set_item("ids", PyCell::new(py, ids).unwrap())
                .unwrap();
            globals
                .set_item("segments", PyCell::new(py, segment_manager).unwrap())
                .unwrap();

            let code = r#"
ids.dict = dict_manager.new_dict(segments, {})
dict_tracker = dict_manager.get_tracker(ids.dict)

assert dict_tracker.current_ptr == ids.start_ptr

dict_tracker.current_ptr = ids.end_ptr
assert dict_tracker.current_ptr == ids.end_ptr
"#;

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(to_vm_error), Ok(()));
        });
    }
}
