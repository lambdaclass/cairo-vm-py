use crate::{utils::to_py_error, vm_core::PyVM};
use cairo_rs::{
    cairo_run::write_output,
    hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor,
    types::{program::Program, relocatable::Relocatable},
    vm::{
        errors::{
            cairo_run_errors::CairoRunError, runner_errors::RunnerError, trace_errors::TraceError,
            vm_errors::VirtualMachineError,
        },
        runners::cairo_runner::CairoRunner,
    },
};
use pyo3::{pyfunction, PyObject, PyResult};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[pyfunction]
#[pyo3(name = "cairo_run")]
pub fn cairo_run_py<'a>(
    path: &'a str,
    entrypoint: &'a str,
    trace_enabled: bool,
    print_output: bool,
    trace_file: Option<&str>,
    memory_file: Option<&str>,
    hint_locals: Option<HashMap<String, PyObject>>,
) -> PyResult<()> {
    let path = Path::new(path);
    let program = Program::new(path, entrypoint).map_err(to_py_error)?;
    let hint_processor = BuiltinHintProcessor::new_empty();
    let mut cairo_runner = CairoRunner::new(&program, &hint_processor).map_err(to_py_error)?;
    let vm = PyVM::new(program.prime, trace_enabled);
    let end = cairo_runner
        .initialize(&mut vm.vm.borrow_mut())
        .map_err(to_py_error)?;
    let mut hint_locals = hint_locals.unwrap_or(HashMap::new());
    run_until_pc(&mut cairo_runner, end, &vm, &mut hint_locals).map_err(to_py_error)?;

    vm.vm
        .borrow_mut()
        .verify_auto_deductions()
        .map_err(to_py_error)?;

    cairo_runner
        .relocate(&mut vm.vm.borrow_mut())
        .map_err(to_py_error)?;

    if print_output {
        write_output(&mut cairo_runner, &mut vm.vm.borrow_mut()).map_err(to_py_error)?;
    }

    if let Some(trace_path) = trace_file {
        let trace_path = PathBuf::from(trace_path);
        let relocated_trace = cairo_runner
            .relocated_trace
            .as_ref()
            .ok_or(CairoRunError::Trace(TraceError::TraceNotEnabled))
            .map_err(to_py_error)?;

        match cairo_rs::cairo_run::write_binary_trace(relocated_trace, &trace_path) {
            Ok(()) => (),
            Err(_e) => {
                return Err(CairoRunError::Runner(RunnerError::WriteFail)).map_err(to_py_error)
            }
        }
    }

    if let Some(memory_path) = memory_file {
        let memory_path = PathBuf::from(memory_path);
        cairo_rs::cairo_run::write_binary_memory(&cairo_runner.relocated_memory, &memory_path)
            .map_err(|_| to_py_error(CairoRunError::Runner(RunnerError::WriteFail)))?;
    }

    Ok(())
}

fn run_until_pc(
    cairo_runner: &mut CairoRunner,
    address: Relocatable,
    vm: &PyVM,
    hint_locals: &mut HashMap<String, PyObject>,
) -> Result<(), VirtualMachineError> {
    let references = cairo_runner.get_reference_list();
    let hint_data_dictionary = cairo_runner.get_hint_data_dictionary(&references)?;

    while vm.vm.borrow().get_pc() != &address {
        vm.step(
            cairo_runner.hint_executor,
            hint_locals,
            &mut cairo_runner.exec_scopes,
            &hint_data_dictionary,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::cairo_run;

    #[test]
    fn cairo_run_fibonacci() {
        cairo_run::cairo_run_py(
            "cairo_programs/fibonacci.json",
            "main",
            false,
            false,
            None,
            None,
            None,
        )
        .expect("Couldn't run program");
    }

    #[test]
    fn cairo_run_array_sum() {
        cairo_run::cairo_run_py(
            "cairo_programs/array_sum.json",
            "main",
            false,
            false,
            None,
            None,
            None,
        )
        .expect("Couldn't run program");
    }

    #[test]
    fn cairo_run_hint_print_vars() {
        cairo_run::cairo_run_py(
            "cairo_programs/hint_print_vars.json",
            "main",
            false,
            false,
            None,
            None,
            None,
        )
        .expect("Couldn't run program");
    }
}
