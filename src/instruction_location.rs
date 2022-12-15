use cairo_rs::serde::deserialize_program::{InputFile, Location};
use pyo3::prelude::*;

#[pyclass]
pub struct InstructionLocation {
    #[pyo3(get)]
    inst: PyLocation,
    #[pyo3(get)]
    hints: Vec<Option<PyLocation>>,
    #[pyo3(get)]
    accesible_scopes: Vec<String>,
}
#[pyclass]
#[pyo3(name = "Location")]
#[derive(Clone)]
pub struct PyLocation {
    #[pyo3(get)]
    pub end_line: u32,
    #[pyo3(get)]
    pub end_col: u32,
    pub input_file: InputFile,
    pub parent_location: Option<(Box<PyLocation>, String)>,
    #[pyo3(get)]
    pub start_line: u32,
    #[pyo3(get)]
    pub start_col: u32,
}

impl From<Location> for PyLocation {
    fn from(loc: Location) -> Self {
        PyLocation {
            end_line: loc.end_line,
            end_col: loc.end_col,
            input_file: loc.input_file,
            parent_location: loc
                .parent_location
                .and_then(|(loc, string)| Some((loc.into(), string))),
            start_line: loc.start_line,
            start_col: loc.end_line,
        }
    }
}

impl From<Box<Location>> for Box<PyLocation> {
    fn from(loc: Box<Location>) -> Self {
        Box::new(PyLocation {
            end_line: loc.end_line,
            end_col: loc.end_col,
            input_file: loc.input_file,
            parent_location: loc
                .parent_location
                .and_then(|(loc, string)| Some((loc.into(), string))),
            start_line: loc.start_line,
            start_col: loc.end_line,
        })
    }
}
impl From<Location> for InstructionLocation {
    fn from(loc: Location) -> Self {
        InstructionLocation {
            inst: loc.into(),
            hints: Vec::new(),
            accesible_scopes: Vec::new(),
        }
    }
}
