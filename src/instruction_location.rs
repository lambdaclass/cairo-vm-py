use cairo_rs::serde::deserialize_program::{InputFile, Location};
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
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
                .map(|(loc, string)| (loc.into(), string)),
            start_line: loc.start_line,
            start_col: loc.start_col,
        }
    }
}

impl From<PyLocation> for Location {
    fn from(loc: PyLocation) -> Self {
        Location {
            end_line: loc.end_line,
            end_col: loc.end_col,
            input_file: loc.input_file,
            parent_location: loc
                .parent_location
                .map(|(loc, string)| (loc.into(), string)),
            start_line: loc.start_line,
            start_col: loc.start_col,
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
                .map(|(loc, string)| (loc.into(), string)),
            start_line: loc.start_line,
            start_col: loc.start_col,
        })
    }
}

impl From<Box<PyLocation>> for Box<Location> {
    fn from(loc: Box<PyLocation>) -> Self {
        Box::new(Location {
            end_line: loc.end_line,
            end_col: loc.end_col,
            input_file: loc.input_file,
            parent_location: loc
                .parent_location
                .map(|(loc, string)| (loc.into(), string)),
            start_line: loc.start_line,
            start_col: loc.start_col,
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

#[pymethods]
impl PyLocation {
    #[getter]
    pub fn parent_location(&self) -> Option<(PyLocation, String)> {
        self.parent_location
            .as_ref()
            .map(|(loc, str)| (*(loc.clone()), str.clone()))
    }
    pub fn to_string_with_content(&self, message: String) -> String {
        let loc = Into::<Location>::into(self.clone());
        loc.to_string(&message)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pylocation_from_location() {
        let loc = Location {
            end_line: 1,
            end_col: 2,
            input_file: InputFile {
                filename: String::from("file.cairo"),
            },
            parent_location: None,
            start_line: 4,
            start_col: 5,
        };
        let pyloc = PyLocation {
            end_line: 1,
            end_col: 2,
            input_file: InputFile {
                filename: String::from("file.cairo"),
            },
            parent_location: None,
            start_line: 4,
            start_col: 5,
        };
        assert_eq!(pyloc, PyLocation::from(loc))
    }

    #[test]
    fn box_pylocation_from_box_location() {
        let loc = Box::new(Location {
            end_line: 1,
            end_col: 2,
            input_file: InputFile {
                filename: String::from("file.cairo"),
            },
            parent_location: None,
            start_line: 4,
            start_col: 5,
        });
        let pyloc = Box::new(PyLocation {
            end_line: 1,
            end_col: 2,
            input_file: InputFile {
                filename: String::from("file.cairo"),
            },
            parent_location: None,
            start_line: 4,
            start_col: 5,
        });
        assert_eq!(pyloc, Box::<PyLocation>::from(loc))
    }

    #[test]
    fn pylocation_from_location_with_parent() {
        let loc = Location {
            end_line: 1,
            end_col: 2,
            input_file: InputFile {
                filename: String::from("file_a.cairo"),
            },
            parent_location: Some((
                Box::new(Location {
                    end_line: 6,
                    end_col: 7,
                    input_file: InputFile {
                        filename: String::from("file_b.cairo"),
                    },
                    parent_location: None,
                    start_line: 8,
                    start_col: 9,
                }),
                String::from("Unexpected exception"),
            )),
            start_line: 4,
            start_col: 5,
        };
        let pyloc = PyLocation {
            end_line: 1,
            end_col: 2,
            input_file: InputFile {
                filename: String::from("file_a.cairo"),
            },
            parent_location: Some((
                Box::new(PyLocation {
                    end_line: 6,
                    end_col: 7,
                    input_file: InputFile {
                        filename: String::from("file_b.cairo"),
                    },
                    parent_location: None,
                    start_line: 8,
                    start_col: 9,
                }),
                String::from("Unexpected exception"),
            )),
            start_line: 4,
            start_col: 5,
        };
        assert_eq!(pyloc, PyLocation::from(loc))
    }

    #[test]
    fn instruction_location_from_location() {
        let loc = Location {
            end_line: 1,
            end_col: 2,
            input_file: InputFile {
                filename: String::from("file.cairo"),
            },
            parent_location: None,
            start_line: 4,
            start_col: 5,
        };

        let inst_location = InstructionLocation {
            inst: PyLocation {
                end_line: 1,
                end_col: 2,
                input_file: InputFile {
                    filename: String::from("file.cairo"),
                },
                parent_location: None,
                start_line: 4,
                start_col: 5,
            },
            hints: Vec::new(),
            accesible_scopes: Vec::new(),
        };
        assert_eq!(inst_location, InstructionLocation::from(loc))
    }

    #[test]
    fn location_from_pylocation() {
        let pyloc = PyLocation {
            end_line: 1,
            end_col: 2,
            input_file: InputFile {
                filename: String::from("file.cairo"),
            },
            parent_location: None,
            start_line: 4,
            start_col: 5,
        };
        let loc = Location {
            end_line: 1,
            end_col: 2,
            input_file: InputFile {
                filename: String::from("file.cairo"),
            },
            parent_location: None,
            start_line: 4,
            start_col: 5,
        };
        assert_eq!(loc, Location::from(pyloc))
    }

    #[test]
    fn box_location_from_box_pylocation() {
        let pyloc = Box::new(PyLocation {
            end_line: 1,
            end_col: 2,
            input_file: InputFile {
                filename: String::from("file.cairo"),
            },
            parent_location: None,
            start_line: 4,
            start_col: 5,
        });
        let loc = Box::new(Location {
            end_line: 1,
            end_col: 2,
            input_file: InputFile {
                filename: String::from("file.cairo"),
            },
            parent_location: None,
            start_line: 4,
            start_col: 5,
        });
        assert_eq!(loc, Box::<Location>::from(pyloc))
    }

    #[test]
    fn get_parent_location_none() {
        let pyloc = Box::new(PyLocation {
            end_line: 1,
            end_col: 2,
            input_file: InputFile {
                filename: String::from("file.cairo"),
            },
            parent_location: None,
            start_line: 4,
            start_col: 5,
        });
        assert_eq!(pyloc.parent_location(), None)
    }

    #[test]
    fn get_parent_location_some() {
        let pyloc = PyLocation {
            end_line: 1,
            end_col: 2,
            input_file: InputFile {
                filename: String::from("file_a.cairo"),
            },
            parent_location: Some((
                Box::new(PyLocation {
                    end_line: 6,
                    end_col: 7,
                    input_file: InputFile {
                        filename: String::from("file_b.cairo"),
                    },
                    parent_location: None,
                    start_line: 8,
                    start_col: 9,
                }),
                String::from("Unexpected exception"),
            )),
            start_line: 4,
            start_col: 5,
        };
        assert_eq!(
            pyloc.parent_location(),
            Some((
                PyLocation {
                    end_line: 6,
                    end_col: 7,
                    input_file: InputFile {
                        filename: String::from("file_b.cairo"),
                    },
                    parent_location: None,
                    start_line: 8,
                    start_col: 9,
                },
                String::from("Unexpected exception"),
            ))
        )
    }

    #[test]
    fn to_string_with_content_no_parent() {
        let pyloc = Box::new(PyLocation {
            end_line: 1,
            end_col: 1,
            input_file: InputFile {
                filename: String::from("file.cairo"),
            },
            parent_location: None,
            start_line: 1,
            start_col: 1,
        });
        assert_eq!(
            pyloc.to_string_with_content(String::from("Message")),
            String::from("file.cairo:1:1:Message")
        )
    }
}
