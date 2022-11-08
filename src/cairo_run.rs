#[cfg(test)]
mod test {
    use crate::cairo_runner::PyCairoRunner;

    #[test]
    fn cairo_run_fibonacci() {
        let mut runner = PyCairoRunner::new(
            "cairo_programs/fibonacci.json".to_string(),
            "main".to_string(),
            None,
        )
        .unwrap();
        runner
            .cairo_run_py(false, None, None, None)
            .expect("Couldn't run program");
    }

    #[test]
    fn cairo_run_array_sum() {
        let mut runner = PyCairoRunner::new(
            "cairo_programs/array_sum.json".to_string(),
            "main".to_string(),
            Some("all".to_owned()),
        )
        .unwrap();
        runner
            .cairo_run_py(false, None, None, None)
            .expect("Couldn't run program");
    }

    #[test]
    fn cairo_run_hint_print_vars() {
        let mut runner = PyCairoRunner::new(
            "cairo_programs/hint_print_vars.json".to_string(),
            "main".to_string(),
            None,
        )
        .unwrap();
        runner
            .cairo_run_py(false, None, None, None)
            .expect("Couldn't run program");
    }
}
