use std::collections::VecDeque;

use pyo3::exceptions::{PyIndexError, PyRuntimeError};
use pyo3::prelude::*;
use pyo3::PyResult;

use hpo::annotations::AnnotationId;

use crate::annotations::PyOmimDisease;
use crate::{from_binary, from_obo, get_ontology, pyterm_from_id, term_from_query, PyQuery};

use crate::PyGene;
use crate::PyHpoTerm;

#[pyclass(module = "hpo3", name = "Ontology")]
pub struct PyOntology {}

impl PyOntology {
    pub fn blank() -> Self {
        Self {}
    }
}

#[pymethods]
impl PyOntology {
    /// A list of all genes included in the ontology
    ///
    /// Returns
    /// -------
    /// list[:class:`hpo3.Gene`]
    ///     All genes that are associated to the :class:`hpo.HPOTerm` in the ontology
    ///
    #[getter(genes)]
    fn genes(&self) -> PyResult<Vec<PyGene>> {
        let ont = get_ontology()?;

        let mut res = Vec::new();
        for gene in ont.genes() {
            res.push(PyGene::new(*gene.id(), gene.name().into()))
        }
        Ok(res)
    }

    /// A list of all Omim Diseases included in the ontology
    ///
    /// Returns
    /// -------
    /// list[:class:`hpo3.Omim`]
    ///     All Omim diseases that are associated to the :class:`hpo.HPOTerm` in the ontology
    ///
    #[getter(omim_diseases)]
    fn omim_diseases(&self) -> PyResult<Vec<PyOmimDisease>> {
        let ont = get_ontology()?;

        let mut res = Vec::new();
        for disease in ont.omim_diseases() {
            res.push(PyOmimDisease::new(*disease.id(), disease.name().into()))
        }
        Ok(res)
    }

    /// Returns a single `HPOTerm` based on its name, synonym or id
    ///
    /// Parameters
    /// ----------
    /// query: str or int
    ///
    ///     * **str** HPO term ``Scoliosis``
    ///     * **str** synonym ``Curved spine``
    ///     * **str** HPO-ID ``HP:0002650``
    ///     * **int** HPO term id ``2650``
    ///
    /// Returns
    /// -------
    /// :class:`hpo.HPOTerm`
    ///     A single matching HPO term instance
    ///
    /// Raises
    /// ------
    /// RuntimeError
    ///     No HPO term is found for the provided query
    /// TypeError
    ///     The provided query is an unsupported type and can't be properly
    ///     converted
    /// ValueError
    ///     The provided HPO ID cannot be converted to the correct
    ///     integer representation
    ///
    /// Example
    /// -------
    ///     ::
    ///
    ///         # Search by ID (int)
    ///         >>> ontology.get_hpo_object(3)
    ///         HP:0000003 | Multicystic kidney dysplasia
    ///
    ///         # Search by HPO-ID (string)
    ///         >>> ontology.get_hpo_object('HP:0000003')
    ///         HP:0000003 | Multicystic kidney dysplasia
    ///
    ///         # Search by term (string)
    ///         >>> ontology.get_hpo_object('Multicystic kidney dysplasia')
    ///         HP:0000003 | Multicystic kidney dysplasia
    ///
    ///         # Search by synonym (string)
    ///         >>> ontology.get_hpo_object('Multicystic renal dysplasia')
    ///         HP:0000003 | Multicystic kidney dysplasia
    ///
    #[pyo3(text_signature = "($self, query)")]
    fn get_hpo_object(&self, query: PyQuery) -> PyResult<PyHpoTerm> {
        Ok(PyHpoTerm::from(term_from_query(query)?))
    }

    #[pyo3(text_signature = "($self, query)")]
    fn r#match(&self, query: &str) -> PyResult<PyHpoTerm> {
        let ont = get_ontology()?;
        for term in ont {
            if term.name() == query {
                return Ok(PyHpoTerm::from(term));
            }
        }

        Err(PyRuntimeError::new_err("No HPO entry found"))
    }

    /// Calculates the shortest path from one to another HPO Term
    ///
    /// IMPORTANT NOTE
    /// --------------
    /// This method is not correctly implemented and will only return
    /// the distance, but not the actual path. It will instead return
    /// an empty list
    #[pyo3(text_signature = "($self, query1, query2)")]
    fn path(
        &self,
        query1: PyQuery,
        query2: PyQuery,
    ) -> PyResult<(usize, Vec<PyHpoTerm>, usize, usize)> {
        let t1 = term_from_query(query1)?;
        let t2 = term_from_query(query2)?;
        let dist = t1
            .distance_to_term(&t2)
            .ok_or_else(|| PyIndexError::new_err("no path between the two terms"))?;
        Ok((dist, vec![], 0, 0))
    }

    /// TODO: Return an actual iterator instead
    #[pyo3(text_signature = "($self, query)")]
    fn search(&self, query: &str) -> PyResult<Vec<PyHpoTerm>> {
        let mut res = Vec::new();
        let ont = get_ontology()?;
        for term in ont {
            if term.name().contains(query) {
                res.push(PyHpoTerm::from(term))
            }
        }

        Ok(res)
    }

    /// Returns the HpoTerm with the provided `id`
    ///
    /// Arguments
    /// ---------
    /// a: `int`
    ///     ID of the term as `int` (`HP:0000123` --> `123`)
    ///
    /// Returns
    /// -------
    /// :class:`hpo3.HpoTerm`
    ///     The HPO-Term
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from hpo3 import Ontology
    ///     
    ///     ont = Ontology()
    ///     
    ///     term = ont.hpo(11968)
    ///     term.name()  # ==> 'Feeding difficulties'
    ///     term.id()    # ==> 'HP:0011968'
    ///     int(tern)    # ==> 11968
    ///
    #[pyo3(text_signature = "($self, id)")]
    fn hpo(&self, id: u32) -> PyResult<PyHpoTerm> {
        pyterm_from_id(id)
    }

    /// Constructs the ontology based on provided ontology files
    ///
    /// The ontology files can be in the standard format as provided
    /// by Jax or as a binary file as generated by `hpo`
    ///
    /// Arguments
    /// path: str
    ///     Path to the source files (default: `./ontology.hpo`)
    /// binary: bool
    ///     Whether the input format is binary (default true)
    #[pyo3(signature = (path = "ontology.hpo", binary = true))]
    fn __call__(&self, path: &str, binary: bool) {
        if binary {
            from_binary(path);
        } else {
            from_obo(path);
        }
    }

    /// Returns the number of HPO-Terms in the Ontology
    ///
    /// Returns
    /// -------
    /// int
    ///     The number of HPO-Terms in the Ontology
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from hpo3 import Ontology
    ///     ont = Ontology()
    ///     len(ont)  # ==> 17059
    ///
    fn __len__(&self) -> PyResult<usize> {
        Ok(get_ontology()?.len())
    }

    fn __repr__(&self) -> String {
        match get_ontology() {
            Ok(ont) => format!("<hpo3.Ontology with {} terms>", ont.len()),
            _ => String::from("<hpo3.Ontology (no data loaded, yet)>"),
        }
    }

    fn __getitem__(&self, id: u32) -> PyResult<PyHpoTerm> {
        self.hpo(id)
    }

    fn __iter__(&self) -> OntologyIterator {
        OntologyIterator::new()
    }
}

#[pyclass(module = "hpo3", name = "OntologyIterator")]
struct OntologyIterator {
    ids: VecDeque<u32>,
}

impl OntologyIterator {
    fn new() -> Self {
        let ids: VecDeque<u32> = get_ontology()
            .unwrap()
            .into_iter()
            .map(|term| term.id().as_u32())
            .collect();
        Self { ids }
    }
}

#[pymethods]
impl OntologyIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyHpoTerm> {
        slf.ids.pop_front().map(|id| pyterm_from_id(id).unwrap())
    }
}
