use pyo3::PyCell;
use std::rc::Rc;
use crate::{utils::to_vm_error, relocatable::PyRelocatable};
use cairo_rs::{hint_processor::{builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData, proxies::{vm_proxy::VMProxy, exec_scopes_proxy::ExecutionScopesProxy}}, vm::errors::vm_errors::VirtualMachineError};
use pyo3::{Python, types::PyDict};

use crate::{memory::PyMemory, memory_segments::PySegmentManager, pycell};

struct PythonExecutor {
    
}

impl PythonExecutor {
    fn execute_hint(
        &self,
        vm_proxy: &mut VMProxy,
        _exec_scopes_proxy: &mut ExecutionScopesProxy,
        hint_data: &HintProcessorData,
    ) -> Result<(), VirtualMachineError> {
        Python::with_gil(|py| -> Result<(), VirtualMachineError> {
                let memory = PyMemory { memory: Rc::clone(&vm_proxy.memory)};
                let segments = PySegmentManager { segment_manager: Rc::clone(&vm_proxy.segments), memory: Rc::clone(&vm_proxy.memory) };
                let globals = PyDict::new(py);
                let ap =  PyRelocatable::new((1, vm_proxy.run_context.ap));
                let fp =  PyRelocatable::new((1, vm_proxy.run_context.fp));
                globals.set_item("memory", pycell!(py, memory)).map_err(to_vm_error)?;
                globals.set_item("segments", pycell!(py, segments)).map_err(to_vm_error)?;
                globals.set_item("ap", pycell!(py, ap)).map_err(to_vm_error)?;
                globals.set_item("fp", pycell!(py, fp)).map_err(to_vm_error)?;
                py.run(&hint_data.code, Some(globals), None).map_err(to_vm_error)?;
                Ok(())
        })?;
        Ok(())
    }
}
