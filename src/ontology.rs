use std::collections::VecDeque;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::PyResult;

use hpo::annotations::AnnotationId;

use crate::annotations::PyOmimDisease;
use crate::from_builtin;
use crate::{from_binary, from_obo, get_ontology, pyterm_from_id, term_from_query, PyQuery};

use crate::PyGene;
use crate::PyHpoTerm;

#[pyclass(name = "Ontology")]
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
    /// list[:class:`pyhpo.Gene`]
    ///     All genes that are associated to the :class:`pyhpo.HPOTerm` in the ontology
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
    /// list[:class:`pyhpo.Omim`]
    ///     All Omim diseases that are associated to the :class:`pyhpo.HPOTerm` in the ontology
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
    /// query: `str` or `int`
    ///
    ///     * **str** HPO term ``Scoliosis``
    ///     * **str** synonym ``Curved spine``
    ///     * **str** HPO-ID ``HP:0002650``
    ///     * **int** HPO term id ``2650``
    ///
    /// Returns
    /// -------
    /// :class:`pyhpo.HPOTerm`
    ///     A single matching HPO term instance
    ///
    /// Raises
    /// ------
    /// `RuntimeError`
    ///     No HPO term is found for the provided query
    /// `TypeError`
    ///     The provided query is an unsupported type and can't be properly
    ///     converted
    /// `ValueError`
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

    /// Returns a single `HPOTerm` based on its name
    ///
    /// Parameters
    /// ----------
    /// query: `str`
    ///     Name of the HPO term, e.g. ``Scoliosis``
    ///
    /// Returns
    /// -------
    /// :class:`pyhpo.HPOTerm`
    ///     A single matching HPO term instance
    ///
    /// Raises
    /// ------
    /// `RuntimeError`
    ///     No HPO term is found for the provided query
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

    /// Returns the shortest path from one to another HPO Term
    ///
    /// Parameters
    /// ----------
    /// query1: `str` or `int`
    ///     HPO term 1, synonym or HPO-ID (HP:00001) to match
    ///     HPO term id (Integer based)
    ///     e.g: ``Abnormality of the nervous system``
    /// query2: `str` or `int`
    ///     HPO term 2, synonym or HPO-ID (HP:00001) to match
    ///     HPO term id (Integer based)
    ///     e.g: ``Abnormality of the nervous system``
    ///
    /// Returns
    /// -------
    /// int
    ///     Length of path
    /// tuple
    ///     Tuple of HPOTerms in the path
    /// int
    ///     Number of steps from term-1 to the common parent
    ///     **(Not yet implemented. Returns ``0``)**
    /// int
    ///     Number of steps from term-2 to the common parent
    ///     **(Not yet implemented. Returns ``0``)**
    ///
    #[pyo3(text_signature = "($self, query1, query2)")]
    fn path(
        &self,
        query1: PyQuery,
        query2: PyQuery,
    ) -> PyResult<(usize, Vec<PyHpoTerm>, usize, usize)> {
        let t1: PyHpoTerm = term_from_query(query1)?.into();
        let t2: PyHpoTerm = term_from_query(query2)?.into();
        t1.path_to_other(&t2)
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
    /// :class:`pyhpo.HPOTerm`
    ///     The HPO-Term
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     
    ///     Ontology()
    ///     
    ///     term = Ontology.hpo(11968)
    ///     term.name()  # ==> 'Feeding difficulties'
    ///     term.id()    # ==> 'HP:0011968'
    ///     int(tern)    # ==> 11968
    ///
    #[pyo3(text_signature = "($self, id)")]
    fn hpo(&self, id: u32) -> PyResult<PyHpoTerm> {
        pyterm_from_id(id)
    }

    /// Returns the HPO version
    ///
    /// Returns
    /// -------
    /// str
    ///     The HPO version, e.g. ``2023-04-05``
    fn version(&self) -> PyResult<String> {
        Ok(get_ontology()?.hpo_version())
    }

    /// Constructs the ontology based on provided ontology files
    ///
    /// The ontology files can be in the standard format as provided
    /// by Jax or as a binary file as generated by `hpo`
    ///
    /// Arguments
    /// data_folder: str
    ///     Path to the source files (default: `./ontology.hpo`)
    /// binary: bool
    ///     Whether the input format is binary (default true)
    #[pyo3(signature = (data_folder = "", from_obo_file = true))]
    fn __call__(&self, data_folder: &str, from_obo_file: bool) {
        if get_ontology().is_ok() {
            println!("The Ontology has been built before already");
            return;
        }
        if data_folder.is_empty() {
            from_builtin();
        } else if from_obo_file {
            from_obo(data_folder);
        } else {
            from_binary(data_folder);
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
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     len(Ontology)  # ==> 17059
    ///
    fn __len__(&self) -> PyResult<usize> {
        Ok(get_ontology()?.len())
    }

    fn __repr__(&self) -> String {
        match get_ontology() {
            Ok(ont) => format!("<pyhpo.Ontology with {} terms>", ont.len()),
            _ => String::from("<pyhpo.Ontology (no data loaded, yet)>"),
        }
    }

    fn __getitem__(&self, id: u32) -> PyResult<PyHpoTerm> {
        self.hpo(id)
    }

    fn __iter__(&self) -> OntologyIterator {
        OntologyIterator::new()
    }
}

#[pyclass(name = "OntologyIterator")]
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
