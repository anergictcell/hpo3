use std::collections::HashSet;
use std::hash::Hash;

use pyo3::class::basic::CompareOp;
use pyo3::exceptions::{PyKeyError, PyTypeError};
use pyo3::types::PyDict;
use pyo3::{prelude::*, types::PyType};

use hpo::annotations::AnnotationId;
use hpo::annotations::{GeneId, OmimDiseaseId};

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
    /// Returns
    /// -------
    /// int
    ///     The HGNC-ID
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     gene = list(Ontology.genes[0]
    ///     gene.id
    ///     # >> 11212
    ///
    #[getter(id)]
    pub fn id(&self) -> u32 {
        self.id.as_u32()
    }

    /// Returns the gene symbol
    ///
    /// Returns
    /// -------
    /// str
    ///     The gene symbol
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     gene = list(Ontology.genes[0]
    ///     gene.name
    ///     # >> 'BRCA2'
    ///
    #[getter(name)]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the IDs of all associated ``HPOTerm``
    ///
    /// Returns
    /// -------
    /// set(int)
    ///     A set of integers, representing the HPO-IDs
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     gene = list(Ontology.genes[0]
    ///     gene.hpo
    ///     # >> {3077, 7, 7703, 2073, 2075, 30236, .....}
    ///
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

    /// Returns a ``HPOSet`` of all associated ``HPOTerm``
    ///
    /// Returns
    /// -------
    /// :class:`pyhpo.HPOSet`
    ///     An ``HPOSet`` containing all associated ``HPOTerm``
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     gene = list(Ontology.genes[0]
    ///     gene.hpo_set()
    ///     # >> HPOSet.from_serialized(7+118+152+234+271+315, ....)
    ///
    fn hpo_set(&self) -> PyResult<PyHpoSet> {
        PyHpoSet::try_from(self)
    }

    /// Returns a gene that matches the provided query
    ///
    /// Paramaters
    /// ----------
    /// query: str or int
    ///     A gene symbol of HGNC-ID
    ///
    /// Returns
    /// -------
    /// :class:`pyhpo.Gene`
    ///     A ``Gene``
    ///
    /// Raises
    /// ------
    /// KeyError: No gene found for the query
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology, Gene
    ///     Ontology()
    ///     Gene.get("BRCA2")
    ///     # >> Gene (BRCA2)>
    ///
    ///     Gene.get(2629)
    ///     # >> <Gene (GBA1)>
    ///
    #[classmethod]
    fn get(_cls: &PyType, query: PyQuery) -> PyResult<PyGene> {
        let ont = get_ontology()?;
        match query {
            PyQuery::Str(symbol) => ont
                .gene_by_name(&symbol)
                .ok_or(PyKeyError::new_err("No gene found for query"))
                .map(|g| PyGene::new(*g.id(), g.name().into())),
            PyQuery::Id(gene_id) => ont
                .gene(&gene_id.into())
                .ok_or(PyKeyError::new_err("No gene found for query"))
                .map(|g| PyGene::new(*g.id(), g.name().into())),
        }
    }

    /// Returns a dict/JSON representation the Gene
    ///
    /// Parameters
    /// ----------
    /// verbose: bool
    ///     Indicates if all associated ``HPOTerm`` should be included in the output
    ///
    /// Returns
    /// -------
    /// Dict
    ///     Dict representation of the gene
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology, Gene
    ///     Ontology()
    ///     Gene.get("BRCA2").toJSON()
    ///     # >> {'name': 'BRCA2', 'id': 675, 'symbol': 'BRCA2'}
    ///
    #[pyo3(signature = (verbose = false))]
    #[pyo3(text_signature = "($self, verbose)")]
    #[allow(non_snake_case)]
    pub fn toJSON<'a>(&'a self, py: Python<'a>, verbose: bool) -> PyResult<&PyDict> {
        let dict = PyDict::new(py);
        dict.set_item("name", self.name())?;
        dict.set_item("id", self.id())?;
        dict.set_item("symbol", self.name())?;

        if verbose {
            let hpos: Vec<u32> = self.hpo()?.iter().copied().collect();
            dict.set_item("hpo", hpos)?;
        }
        Ok(dict)
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

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => Err(PyTypeError::new_err(
                "\"<\" is not supported for Gene instances",
            )),
            CompareOp::Le => Err(PyTypeError::new_err(
                "\"<=\" is not supported for Gene instances",
            )),
            CompareOp::Eq => Ok(self == other),
            CompareOp::Ne => Ok(self != other),
            CompareOp::Gt => Err(PyTypeError::new_err(
                "\">\" is not supported for Gene instances",
            )),
            CompareOp::Ge => Err(PyTypeError::new_err(
                "\">=\" is not supported for Gene instances",
            )),
        }
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
    /// Returns
    /// -------
    /// int
    ///     The Omim-ID
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     disease = list(Ontology.omim_diseases)[0]
    ///     disease.id    # ==> 183849
    ///
    #[getter(id)]
    pub fn id(&self) -> u32 {
        self.id.as_u32()
    }

    /// Returns the name of the disease
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     disease = list(Ontology.omim_diseases)[0]
    ///     gene.name  # ==> 'Spondyloepimetaphyseal dysplasia with hypotrichosis'
    ///
    #[getter(name)]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the IDs of all associated ``HPOTerm``
    ///
    /// Returns
    /// -------
    /// set(int)
    ///     A set of integers, representing the HPO-IDs
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     disease = list(Ontology.omim_diseases)[0]
    ///     disease.hpo
    ///     # >> {100864, 5090, 4581, 6, 2663, 3911, 6599, ...}
    ///
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

    /// Returns a ``HPOSet`` of all associated ``HPOTerm``
    ///
    /// Returns
    /// -------
    /// :class:`pyhpo.HPOSet`
    ///     An ``HPOSet`` containing all associated ``HPOTerm``
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     disease = list(Ontology.omim_diseases)[0]
    ///     disease.hpo_set()
    ///     # >> HPOSet.from_serialized(6+2651+2663+2812+2834+2869, ..._
    ///
    fn hpo_set(&self) -> PyResult<PyHpoSet> {
        PyHpoSet::try_from(self)
    }

    /// Returns a gene that matches the provided query
    ///
    /// Paramaters
    /// ----------
    /// query: int
    ///     An Omim ID
    ///
    /// Returns
    /// -------
    /// :class:`pyhpo.Omim`
    ///     A ``Omim``
    ///
    /// Raises
    /// ------
    /// KeyError: No disease found for the query
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology, Omim
    ///     Ontology()
    ///     Omim.get(183849)
    ///     # >> <OmimDisease (183849)>
    ///
    #[classmethod]
    fn get(_cls: &PyType, id: u32) -> PyResult<PyOmimDisease> {
        let ont = get_ontology()?;
        ont.omim_disease(&id.into())
            .ok_or(PyKeyError::new_err("'No disease found for query'"))
            .map(|d| PyOmimDisease::new(*d.id(), d.name().into()))
    }

    /// Returns a dict/JSON representation the Omim disease
    ///
    /// Parameters
    /// ----------
    /// verbose: bool
    ///     Indicates if all associated ``HPOTerm`` should be included in the output
    ///
    /// Returns
    /// -------
    /// Dict
    ///     Dict representation of the Omim disease
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology, Omim
    ///     Ontology()
    ///     Omim.get(183849).toJSON()
    ///     # >> {'name': 'Spondyloepimetaphyseal dysplasia with hypotrichosis', 'id': 183849}
    ///
    #[pyo3(signature = (verbose = false))]
    #[pyo3(text_signature = "($self, verbose)")]
    #[allow(non_snake_case)]
    pub fn toJSON<'a>(&'a self, py: Python<'a>, verbose: bool) -> PyResult<&PyDict> {
        let dict = PyDict::new(py);
        dict.set_item("name", self.name())?;
        dict.set_item("id", self.id())?;

        if verbose {
            let hpos: Vec<u32> = self.hpo()?.iter().copied().collect();
            dict.set_item("hpo", hpos)?;
        }

        Ok(dict)
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

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => Err(PyTypeError::new_err(
                "\"<\" is not supported for Omim instances",
            )),
            CompareOp::Le => Err(PyTypeError::new_err(
                "\"<=\" is not supported for Omim instances",
            )),
            CompareOp::Eq => Ok(self == other),
            CompareOp::Ne => Ok(self != other),
            CompareOp::Gt => Err(PyTypeError::new_err(
                "\">\" is not supported for Omim instances",
            )),
            CompareOp::Ge => Err(PyTypeError::new_err(
                "\">=\" is not supported for Omim instances",
            )),
        }
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
