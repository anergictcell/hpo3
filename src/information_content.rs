use pyo3::exceptions::PyKeyError;
use pyo3::prelude::*;
use pyo3::PyErr;
use pyo3::PyResult;

/// Holds the information content for an ``HPOTerm``
#[pyclass(name = "InformationContent")]
pub struct PyInformationContent {
    omim: f32,
    orpha: f32,
    gene: f32,
}

impl From<&hpo::term::InformationContent> for PyInformationContent {
    fn from(value: &hpo::term::InformationContent) -> Self {
        Self {
            omim: value.omim_disease(),
            orpha: value.orpha_disease(),
            gene: value.gene(),
        }
    }
}

#[pymethods]
impl PyInformationContent {
    /// Returns the gene - based information content
    #[getter(gene)]
    pub fn gene(&self) -> f32 {
        self.gene
    }

    /// Returns the Omim disease - based information content
    #[getter(omim)]
    pub fn omim(&self) -> f32 {
        self.omim
    }
    /// Returns the Orpha disease - based information content
    #[getter(orpha)]
    pub fn orpha(&self) -> f32 {
        self.orpha
    }

    fn __getitem__(&self, key: &str) -> PyResult<f32> {
        match key {
            "omim" => Ok(self.omim()),
            "orpha" => Ok(self.orpha()),
            "gene" => Ok(self.gene()),
            _ => Err(PyKeyError::new_err(format!("Unknown key {}", key))),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "<InformationContent (Omim: {:.4}, Oprha: {:.4}, Gene: {:.4})>",
            self.omim(),
            self.orpha(),
            self.gene(),
        )
    }
}

#[pyclass(name = "InformationContentKind")]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PyInformationContentKind {
    Omim,
    Orpha,
    Gene,
}

impl TryFrom<&str> for PyInformationContentKind {
    type Error = PyErr;
    /// # Errors
    /// PyKeyError
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "omim" => Ok(PyInformationContentKind::Omim),
            "orpha" => Ok(PyInformationContentKind::Orpha),
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
            PyInformationContentKind::Orpha => Self::Orpha,
            PyInformationContentKind::Gene => Self::Gene,
        }
    }
}
