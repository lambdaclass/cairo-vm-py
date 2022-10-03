use crate::{utils::to_py_error, vm_core::PyVM};
use cairo_rs::{
    cairo_run::write_output,
    hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor,
    types::{program::Program, relocatable::Relocatable},
    vm::{errors::vm_errors::VirtualMachineError, runners::cairo_runner::CairoRunner},
};
use pyo3::{pyfunction, PyResult};
use std::path::Path;

#[pyfunction]
#[pyo3(name = "cairo_run")]
pub fn cairo_run_py<'a>(
    path: &'a str,
    entrypoint: &'a str,
    trace_enabled: bool,
    print_output: bool,
) -> PyResult<()> {
    let path = Path::new(path);
    let program = Program::new(path, entrypoint).map_err(to_py_error)?;
    let hint_processor = BuiltinHintProcessor::new_empty();
    let mut cairo_runner = CairoRunner::new(&program, &hint_processor).map_err(to_py_error)?;
    let vm = PyVM::new(program.prime, trace_enabled);
    let end = cairo_runner
        .initialize(&mut vm.vm.borrow_mut())
        .map_err(to_py_error)?;

    run_until_pc(&mut cairo_runner, end, &vm).map_err(to_py_error)?;

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

    Ok(())
}

fn run_until_pc(
    cairo_runner: &mut CairoRunner,
    address: Relocatable,
    vm: &PyVM,
) -> Result<(), VirtualMachineError> {
    let references = cairo_runner.get_reference_list();
    let hint_data_dictionary = cairo_runner.get_hint_data_dictionary(&references)?;

    while vm.vm.borrow().run_context.pc != address {
        vm.step(
            cairo_runner.hint_executor,
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
        cairo_run::cairo_run_py("cairo_programs/fibonacci.json", "main", false, false)
            .expect("Couldn't run program");
    }

    #[test]
    fn cairo_run_array_sum() {
        cairo_run::cairo_run_py("cairo_programs/array_sum.json", "main", false, false)
            .expect("Couldn't run program");
    }
}
