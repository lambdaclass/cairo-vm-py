use crate::relocatable::PyRelocatable;
use pyo3::{pyclass, pymethods};

#[pyclass]
pub struct PyRunContext {
    pub pc: PyRelocatable,
    pub ap: PyRelocatable,
    pub fp: PyRelocatable,
}

#[pymethods]
impl PyRunContext {
    #[getter]
    pub fn pc(&self) -> PyRelocatable {
        self.pc.clone()
    }

    #[getter]
    pub fn ap(&self) -> PyRelocatable {
        self.ap.clone()
    }

    #[getter]
    pub fn fp(&self) -> PyRelocatable {
        self.fp.clone()
    }
}

#[cfg(test)]
mod test {
    use crate::run_context::PyRunContext;

    #[test]
    fn ap() {
        let run_context = PyRunContext {
            pc: (1, 2).into(),
            ap: (3, 4).into(),
            fp: (5, 6).into(),
        };

        assert_eq!(run_context.pc(), (1, 2).into());
        assert_eq!(run_context.ap(), (3, 4).into());
        assert_eq!(run_context.fp(), (5, 6).into());
    }
}
