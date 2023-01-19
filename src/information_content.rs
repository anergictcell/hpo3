use pyo3::exceptions::PyKeyError;
use pyo3::prelude::*;
use pyo3::PyErr;
use pyo3::PyResult;

#[pyclass(name = "InformationContent")]
pub struct PyInformationContent {
    omim: f32,
    gene: f32,
}

impl From<&hpo::term::InformationContent> for PyInformationContent {
    fn from(value: &hpo::term::InformationContent) -> Self {
        Self {
            omim: value.omim_disease(),
            gene: value.gene(),
        }
    }
}

#[pymethods]
impl PyInformationContent {
    #[getter(gene)]
    pub fn gene(&self) -> f32 {
        self.gene
    }

    #[getter(omim)]
    pub fn omim(&self) -> f32 {
        self.omim
    }

    fn __getitem__(&self, key: &str) -> PyResult<f32> {
        match key {
            "omim" => Ok(self.omim()),
            "gene" => Ok(self.gene()),
            _ => Err(PyKeyError::new_err(format!("Unknown key {}", key))),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "<InformationContent (Omim: {:.4}, Gene: {:.4})>",
            self.gene(),
            self.omim()
        )
    }
}

#[pyclass(name = "InformationContentKind")]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PyInformationContentKind {
    Omim,
    Gene,
}

impl TryFrom<&str> for PyInformationContentKind {
    type Error = PyErr;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "omim" => Ok(PyInformationContentKind::Omim),
            "gene" => Ok(PyInformationContentKind::Gene),
            _ => Err(PyKeyError::new_err(format!(
                "Unknown information content kind {}",
                value
            ))),
        }
    }
}

impl From<PyInformationContentKind> for hpo::term::InformationContentKind {
    fn from(value: PyInformationContentKind) -> Self {
        match value {
            PyInformationContentKind::Omim => Self::Omim,
            PyInformationContentKind::Gene => Self::Gene,
        }
    }
}
