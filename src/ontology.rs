use hpo::annotations::Disease;
use std::collections::VecDeque;

use hpo::HpoError;
use pyo3::exceptions::PyFileNotFoundError;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::PyResult;

use hpo::annotations::AnnotationId;

use crate::annotations::PyOmimDisease;
use crate::annotations::PyOrphaDisease;
use crate::from_builtin;
use crate::{from_binary, from_obo, get_ontology, pyterm_from_id, term_from_query, PyQuery};

use crate::PyGene;
use crate::PyHpoTerm;

#[pyclass(name = "_Ontology")]
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
    ///
    /// .. important::
    ///
    ///    The return type of this method will very likely change
    ///    into an Iterator of ``Gene``. (:doc:`api_changes`)
    ///
    /// Raises
    /// ------
    ///
    /// NameError: Ontology not yet constructed
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
    ///
    /// .. important::
    ///
    ///    The return type of this method will very likely change
    ///    into an Iterator of ``Omim``. (:doc:`api_changes`)
    ///
    /// Raises
    /// ------
    ///
    /// NameError: Ontology not yet constructed
    #[getter(omim_diseases)]
    fn omim_diseases(&self) -> PyResult<Vec<PyOmimDisease>> {
        let ont = get_ontology()?;

        let mut res = Vec::new();
        for disease in ont.omim_diseases() {
            res.push(PyOmimDisease::new(*disease.id(), disease.name().into()))
        }
        Ok(res)
    }

    /// A list of all Orpha Diseases included in the ontology
    ///
    /// Returns
    /// -------
    /// list[:class:`pyhpo.Orpha`]
    ///     All Orpha diseases that are associated to the :class:`pyhpo.HPOTerm` in the ontology
    ///
    ///
    /// .. important::
    ///
    ///    The return type of this method will very likely change
    ///    into an Iterator of ``Orpha``. (:doc:`api_changes`)
    ///
    /// Raises
    /// ------
    ///
    /// NameError: Ontology not yet constructed
    #[getter(orpha_diseases)]
    fn orpha_diseases(&self) -> PyResult<Vec<PyOrphaDisease>> {
        let ont = get_ontology()?;

        let mut res = Vec::new();
        for disease in ont.orpha_diseases() {
            res.push(PyOrphaDisease::new(*disease.id(), disease.name().into()))
        }
        Ok(res)
    }

    /// Returns a single `HPOTerm` based on its name or id
    ///
    /// Parameters
    /// ----------
    /// query: str or int
    ///
    ///     * **str** HPO term (e.g.: ``Scoliosis``)
    ///     * **str** HPO-ID (e.g.: ``HP:0002650``)
    ///     * **int** HPO term id (e.g.: ``2650``)
    ///
    /// Returns
    /// -------
    /// :class:`pyhpo.HPOTerm`
    ///     A single matching HPO term instance
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
    /// RuntimeError
    ///     No HPO term is found for the provided query
    /// TypeError
    ///     The provided query is an unsupported type and can't be properly
    ///     converted
    /// ValueError
    ///     The provided HPO ID cannot be converted to the correct
    ///     integer representation
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///
    ///     # Search by ID (int)
    ///     Ontology.get_hpo_object(3)
    ///     # >> HP:0000003 | Multicystic kidney dysplasia
    ///
    ///     # Search by HPO-ID (string)
    ///     Ontology.get_hpo_object('HP:0000003')
    ///     # >> HP:0000003 | Multicystic kidney dysplasia
    ///
    ///     # Search by term (string)
    ///     Ontology.get_hpo_object('Multicystic kidney dysplasia')
    ///     # >> HP:0000003 | Multicystic kidney dysplasia
    ///
    ///
    /// .. note::
    ///
    ///    This method differs slightly from `pyhpo`, because
    ///    it does not fall back to the synonym for searching
    ///
    #[pyo3(text_signature = "($self, query)")]
    fn get_hpo_object(&self, query: PyQuery) -> PyResult<PyHpoTerm> {
        Ok(PyHpoTerm::from(term_from_query(query)?))
    }

    /// Returns a single `HPOTerm` based on its name
    ///
    /// Parameters
    /// ----------
    /// query: str
    ///     Name of the HPO term, e.g. ``Scoliosis``
    ///
    /// Returns
    /// -------
    /// :class:`pyhpo.HPOTerm`
    ///     A single matching HPO term instance
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
    /// RuntimeError
    ///     No HPO term is found for the provided query
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///
    ///     Ontology.match('Multicystic kidney dysplasia')
    ///     # >>> HP:0000003 | Multicystic kidney dysplasia
    ///
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
    /// query1: str or int
    ///     Name, HPO-ID (HP:0040064) or integer ID of source term
    ///     e.g: ``Abnormality of the nervous system``
    /// query2: str or int
    ///     Name, HPO-ID (HP:0040064) or integer ID of target term
    ///     e.g: ``Abnormality of the nervous system``
    ///
    /// Returns
    /// -------
    /// int
    ///     Length of path
    /// list
    ///     List of HPOTerms in the path
    /// int
    ///     Number of steps from term1 to the common parent
    ///     (Not implemented. Returns ``0``)
    /// int
    ///     Number of steps from term2 to the common parent
    ///     (Not implemented. Returns ``0``)
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
    /// RuntimeError
    ///     No HPO term is found for the provided query
    /// TypeError
    ///     The provided query is an unsupported type and can't be properly
    ///     converted
    /// ValueError
    ///     The provided HPO ID cannot be converted to the correct
    ///     integer representation
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///
    ///     Ontology.path(40064, 'Multicystic kidney dysplasia')
    ///     # >> (
    ///     # >>     8,
    ///     # >>     [
    ///     # >>         <HpoTerm (HP:0040064)>, <HpoTerm (HP:0000118)>,
    ///     # >>         <HpoTerm (HP:0000119)>, <HpoTerm (HP:0000079)>,
    ///     # >>         <HpoTerm (HP:0010935)>, <HpoTerm (HP:0000077)>,
    ///     # >>         <HpoTerm (HP:0012210)>, <HpoTerm (HP:0000107)>,
    ///     # >>         <HpoTerm (HP:0000003)>
    ///     # >>     ],
    ///     # >>     0,
    ///     # >>     0
    ///     # >> )
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

    /// Returns a list of HPOTerms that match the query
    ///
    /// Parameters
    /// ----------
    /// query: str
    ///     Query for substring search of HPOTerms
    ///
    /// Returns
    /// -------
    /// list[:class:`HPOTerm`]
    ///     All terms matching the query string
    ///
    ///
    /// .. important::
    ///
    ///    The return type of this method will very likely change
    ///    into an Iterator of ``HPOTerm``. (:doc:`api_changes`)
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///
    ///     for term in Ontology.search("kidney dis"):
    ///         print(term)
    ///
    ///     # >> HP:0003774 | Stage 5 chronic kidney disease
    ///     # >> HP:0012622 | Chronic kidney disease
    ///     # >> HP:0012623 | Stage 1 chronic kidney disease
    ///     # >> HP:0012624 | Stage 2 chronic kidney disease
    ///     # >> HP:0012625 | Stage 3 chronic kidney disease
    ///     # >> HP:0012626 | Stage 4 chronic kidney disease
    ///
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
    /// Parameters
    /// ----------
    /// id: int
    ///     ID of the term as int (``HP:0000123`` --> ``123``)
    ///
    /// Returns
    /// -------
    /// :class:`pyhpo.HPOTerm`
    ///     The HPO-Term
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
    /// KeyError
    ///     No HPO term is found for the provided query
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
    ///     term.name()  # >> 'Feeding difficulties'
    ///     term.id()    # >> 'HP:0011968'
    ///     int(tern)    # >> 11968
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
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
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
    ///     Ontology.version()
    ///     # >> "2023-04-05"
    ///
    fn version(&self) -> PyResult<String> {
        Ok(get_ontology()?.hpo_version())
    }

    /// Constructs the ontology based on provided ontology files
    ///
    /// The ontology files can be in the standard format as provided
    /// by Jax or as a binary file as generated by `hpo`
    ///
    /// Parameters
    /// ----------
    /// data_folder: str
    ///     Path to the source files (default: `./ontology.hpo`)
    /// binary: bool
    ///     Whether the input format is binary (default true)
    /// transitive: bool
    ///     Whether to associate HPOTerms transitively to genes.
    ///     You must provide the `phenotype_to_genes.txt` input file.

    ///    # This requires the files:
    /// # - Actual OBO data: hp.obo from https://hpo.jax.org/app/data/ontology
    /// # - Links between HPO and OMIM diseases: phenotype.hpoa from https://hpo.jax.org/app/data/annotations
    /// # - Links between HPO and Genes: [`genes_to_phenotype.txt`](http://purl.obolibrary.org/obo/hp/hpoa/genes_to_phenotype.txt)
    /// #

    #[pyo3(signature = (data_folder = "", from_obo_file = true, transitive = false))]
    fn __call__(&self, data_folder: &str, from_obo_file: bool, transitive: bool) -> PyResult<()> {
        if get_ontology().is_ok() {
            println!("The Ontology has been built before already");
            return Ok(());
        }
        if data_folder.is_empty() {
            from_builtin();
            Ok(())
        } else if from_obo_file {
            match from_obo(data_folder, transitive) {
                Ok(_) => return Ok(()),
                Err(HpoError::CannotOpenFile(filename)) => {
                    if filename.ends_with("genes_to_phenotype.txt") {
                        return Err(PyFileNotFoundError::new_err("Starting with v1.2.0, hpo3 changed the way \
                            how the ontology is build from JAX-OBO source. It now requires the `genes_to_phenotype.txt` \
                            file. Please check the documentation for more info or add the `transitive=True` argument."));
                    }
                    return Err(PyFileNotFoundError::new_err(
                        format!("Unable to open {filename}. Please check if you specified the correct path and all files are present.")
                    ));
                }
                Err(err) => {
                    return Err(PyRuntimeError::new_err(format!(
                        "Error loading the ontology. Please check if the data is correct: {err}"
                    )));
                }
            }
        } else {
            from_binary(data_folder);
            return Ok(());
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
    ///     len(Ontology)  # >> 17059
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
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

    /// Subset the Ontology to retrieve a single ``HPOTerm``
    ///
    /// Parameters
    /// ----------
    /// id: int
    ///     The integer representation of the HPO-ID
    ///
    /// Returns
    /// -------
    /// :class:`HPOTerm`
    ///     The ``HPOTerm``
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
    /// KeyError
    ///     No HPO term is found for the provided query
    ///
    fn __getitem__(&self, id: u32) -> PyResult<PyHpoTerm> {
        self.hpo(id)
    }

    /// Iterate all ``HPOTerms`` within the Ontology
    ///
    /// Returns
    /// -------
    /// Iterator[:class:`HPOTerm`]
    ///     An iterator of ``HPOTerm``\s
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
    ///
    fn __iter__(&self) -> PyResult<OntologyIterator> {
        OntologyIterator::new()
    }
}

#[pyclass(name = "OntologyIterator")]
struct OntologyIterator {
    ids: VecDeque<u32>,
}

impl OntologyIterator {
    fn new() -> PyResult<Self> {
        let ids: VecDeque<u32> = get_ontology()?
            .into_iter()
            .map(|term| term.id().as_u32())
            .collect();
        Ok(Self { ids })
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
