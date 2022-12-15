use cairo_rs::serde::deserialize_program::Location;
use pyo3::prelude::*;

#[pyclass]
pub struct InstructionLocation {
    inst: Location,
    hints: Vec<Option<Location>>,
    accesible_scopes: Vec<String>,
}

impl From<Location> for InstructionLocation {
    fn from(loc: Location) -> Self {
        InstructionLocation {
            inst: loc,
            hints: Vec::new(),
            accesible_scopes: Vec::new(),
        }
    }
}
