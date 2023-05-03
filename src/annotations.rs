use hpo::annotations::AnnotationId;
use pyo3::{prelude::*, types::PyType};
use std::collections::HashSet;

use hpo::annotations::{GeneId, OmimDiseaseId};
use std::hash::Hash;

use crate::{get_ontology, set::PyHpoSet, PyQuery};

pub trait PythonAnnotation {}

#[pyclass(name = "Gene")]
pub(crate) struct PyGene {
    id: GeneId,
    name: String,
}

impl PyGene {
    pub fn new(id: GeneId, name: String) -> Self {
        Self { id, name }
    }
}

impl PythonAnnotation for PyGene {}

#[pymethods]
impl PyGene {
    /// Returns the Gene Id
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     ont = Ontology()
    ///     gene = ont.genes()[0]
    ///     gene.id()    # ==> 11212
    ///
    #[getter(id)]
    pub fn id(&self) -> u32 {
        self.id.as_u32()
    }

    /// Returns the name of the HPO Term
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     ont = Ontology()
    ///     gene = ont.genes()[0]
    ///     gene.name()
    ///     # >> 'BRCA2'
    ///
    #[getter(name)]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[getter(hpo)]
    pub fn hpo(&self) -> PyResult<HashSet<u32>> {
        let ont = get_ontology()?;
        Ok(ont
            .gene(&self.id)
            .expect("ontology must be present and gene must be included")
            .hpo_terms()
            .iter()
            .fold(HashSet::new(), |mut set, tid| {
                set.insert(tid.as_u32());
                set
            }))
    }

    fn hpo_set(&self) -> PyResult<PyHpoSet> {
        PyHpoSet::try_from(self)
    }

    #[classmethod]
    fn get(_cls: &PyType, query: PyQuery) -> PyResult<Option<PyGene>> {
        let ont = get_ontology()?;
        match query {
            PyQuery::Str(symbol) => Ok(ont
                .gene_by_name(&symbol)
                .map(|g| PyGene::new(*g.id(), g.name().into()))),
            PyQuery::Id(gene_id) => Ok(ont
                .gene(&gene_id.into())
                .map(|g| PyGene::new(*g.id(), g.name().into()))),
        }
    }

    fn __str__(&self) -> String {
        format!("{} | {}", self.id(), self.name())
    }

    fn __repr__(&self) -> String {
        format!("<Gene ({})>", self.name())
    }

    fn __int__(&self) -> u32 {
        self.id.as_u32()
    }

    fn __hash__(&self) -> u32 {
        self.__int__()
    }
}

impl PartialEq for PyGene {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
impl Eq for PyGene {}

impl Hash for PyGene {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.id.as_u32())
    }
}

impl From<&hpo::annotations::Gene> for PyGene {
    fn from(value: &hpo::annotations::Gene) -> Self {
        Self {
            id: *value.id(),
            name: value.name().into(),
        }
    }
}

#[pyclass(name = "Omim")]
pub(crate) struct PyOmimDisease {
    id: OmimDiseaseId,
    name: String,
}

impl PyOmimDisease {
    pub fn new(id: OmimDiseaseId, name: String) -> Self {
        Self { id, name }
    }
}

impl PythonAnnotation for PyOmimDisease {}

#[pymethods]
impl PyOmimDisease {
    /// Returns the OmimDisease Id
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     ont = Ontology()
    ///     gene = ont.omim_diseases()[0]
    ///     gene.id    # ==> 41232
    ///
    #[getter(id)]
    pub fn id(&self) -> u32 {
        self.id.as_u32()
    }

    /// Returns the name of the HPO Term
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     ont = Ontology()
    ///     gene = ont.omim_diseases()[0]
    ///     gene.name  # ==> 'Gaucher'
    ///
    #[getter(name)]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[getter(hpo)]
    pub fn hpo(&self) -> PyResult<HashSet<u32>> {
        let ont = get_ontology()?;
        Ok(ont.omim_disease(&self.id).unwrap().hpo_terms().iter().fold(
            HashSet::new(),
            |mut set, tid| {
                set.insert(tid.as_u32());
                set
            },
        ))
    }

    fn hpo_set(&self) -> PyResult<PyHpoSet> {
        PyHpoSet::try_from(self)
    }

    #[classmethod]
    fn get(_cls: &PyType, id: u32) -> PyResult<Option<PyOmimDisease>> {
        let ont = get_ontology()?;
        Ok(ont
            .omim_disease(&id.into())
            .map(|d| PyOmimDisease::new(*d.id(), d.name().into())))
    }

    fn __str__(&self) -> String {
        format!("{} | {}", self.id(), self.name())
    }

    fn __repr__(&self) -> String {
        format!("<OmimDisease ({})>", self.id())
    }

    fn __int__(&self) -> u32 {
        self.id.as_u32()
    }

    fn __hash__(&self) -> u32 {
        self.__int__()
    }
}

impl PartialEq for PyOmimDisease {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
impl Eq for PyOmimDisease {}

impl Hash for PyOmimDisease {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.id.as_u32())
    }
}

impl From<&hpo::annotations::OmimDisease> for PyOmimDisease {
    fn from(value: &hpo::annotations::OmimDisease) -> Self {
        Self {
            id: *value.id(),
            name: value.name().into(),
        }
    }
}
