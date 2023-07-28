use once_cell::sync::OnceCell;

use rayon::prelude::*;

use pyo3::exceptions::{PyKeyError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use hpo::annotations::{AnnotationId, GeneId, OmimDiseaseId};
use hpo::similarity::{GroupSimilarity, Similarity, StandardCombiner};
use hpo::stats::hypergeom::{disease_enrichment, gene_enrichment};
use hpo::term::HpoTermId;
use hpo::{HpoTerm, Ontology as ActualOntology};

mod annotations;
mod enrichment;
mod information_content;
mod linkage;
mod ontology;
mod set;
mod term;

use crate::annotations::{PyGene, PyOmimDisease};
use crate::enrichment::PyEnrichmentModel;
use crate::information_content::{PyInformationContent, PyInformationContentKind};
use crate::ontology::PyOntology;
use crate::set::PyHpoSet;
use crate::term::PyHpoTerm;

static ONTOLOGY: OnceCell<ActualOntology> = OnceCell::new();

/// Builds the ontology from a binary HPO dump
fn from_binary(path: &str) -> usize {
    let ont = ActualOntology::from_binary(path).unwrap();
    ONTOLOGY.set(ont).unwrap();
    ONTOLOGY.get().unwrap().len()
}

fn from_builtin() -> usize {
    let bytes = include_bytes!("../data/ontology.hpo");
    let ont = ActualOntology::from_bytes(&bytes[..]).expect("Unable to build Ontology");
    ONTOLOGY.set(ont).unwrap();
    ONTOLOGY.get().unwrap().len()
}

/// Builds the ontology from the JAX download files
fn from_obo(path: &str) -> usize {
    let ont = ActualOntology::from_standard(path).unwrap();
    ONTOLOGY.set(ont).unwrap();
    ONTOLOGY.get().unwrap().len()
}

/// Returns a reference to the Ontology
///
/// This method only works **after** building the ontology
fn get_ontology() -> PyResult<&'static ActualOntology> {
    ONTOLOGY.get().ok_or_else(|| {
        pyo3::exceptions::PyNameError::new_err(
            "You must build the ontology first: `>> pyhpo.Ontology()`",
        )
    })
}

/// Returns a [`PyHpoTerm`] from a `u32` ID
fn pyterm_from_id(id: u32) -> PyResult<PyHpoTerm> {
    let term = term_from_id(id)?;
    Ok(PyHpoTerm::new(term.id(), term.name().to_string()))
}

/// Returns an [`HpoTerm`] from a `u32` ID
///
/// # Errors
///
/// PyKeyError: No term with that ID present in Ontology
fn term_from_id(id: u32) -> PyResult<hpo::HpoTerm<'static>> {
    let ont = get_ontology()?;
    match ont.hpo(id) {
        Some(term) => Ok(term),
        None => Err(PyKeyError::new_err(format!("No HPOTerm for index {}", id))),
    }
}

/// Returns an [`HpoTerm`] from a `str` or `u32` query
///
/// # Errors
///
/// PyValueError: query cannot be converted to HpoTermId
/// PyRuntimeError: query is a name and does not have a match in the Ontology
fn term_from_query(query: PyQuery) -> PyResult<HpoTerm<'static>> {
    match query {
        PyQuery::Id(id) => {
            return term_from_id(id).map_err(|_| PyRuntimeError::new_err("Unknown HPO term"))
        }
        PyQuery::Str(term_name) => {
            if term_name.starts_with("HP:") {
                match HpoTermId::try_from(term_name.as_str()) {
                    Ok(termid) => {
                        return term_from_id(termid.as_u32())
                            .map_err(|_| PyRuntimeError::new_err("Unknown HPO term"))
                    }
                    Err(_) => {
                        return Err(PyValueError::new_err(format!("Invalid id: {}", term_name)))
                    }
                }
            } else {
                let ont = get_ontology()?;
                for term in ont {
                    if term.name() == term_name {
                        return Ok(term);
                    }
                }
            }
        }
    };
    Err(PyRuntimeError::new_err("Unknown HPO term"))
}

#[derive(FromPyObject)]
pub enum PyQuery {
    Id(u32),
    Str(String),
}

#[derive(FromPyObject)]
pub enum TermOrId {
    Term(PyHpoTerm),
    Id(u32),
}

/// Python bindings for the Rust hpo crate
///
/// This library aims to be a drop-in replacement for
/// `pyhpo <https://pypi.org/project/pyhpo/>`_
#[pymodule]
fn pyhpo(py: Python, m: &PyModule) -> PyResult<()> {
    let ont = PyOntology::blank();
    m.add_class::<PyGene>()?;
    m.add_class::<PyOmimDisease>()?;
    m.add_class::<PyHpoSet>()?;
    m.add_class::<PyHpoTerm>()?;
    m.add("Ontology", ont)?;
    m.add("BasicHPOSet", set::BasicPyHpoSet::default())?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__backend__", env!("CARGO_PKG_NAME"))?;
    register_helper_module(py, m)?;
    register_stats_module(py, m)?;
    register_set_module(py, m)?;
    register_annotations_module(py, m)?;
    Ok(())
}

fn register_annotations_module(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let child_module = PyModule::new(py, "annotations")?;
    child_module.add_class::<PyGene>()?;
    child_module.add_class::<PyOmimDisease>()?;
    parent_module.add_submodule(child_module)?;

    py.import("sys")?
        .getattr("modules")?
        .set_item("pyhpo.annotations", child_module)?;

    Ok(())
}

fn register_set_module(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let child_module = PyModule::new(py, "set")?;
    child_module.add_class::<set::BasicPyHpoSet>()?;
    child_module.add_class::<set::PyHpoSet>()?;
    child_module.add_class::<set::PhenoSet>()?;
    parent_module.add_submodule(child_module)?;

    py.import("sys")?
        .getattr("modules")?
        .set_item("pyhpo.set", child_module)?;

    Ok(())
}

fn register_helper_module(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let child_module = PyModule::new(py, "helper")?;
    child_module.add_function(wrap_pyfunction!(batch_similarity, child_module)?)?;
    child_module.add_function(wrap_pyfunction!(set_batch_similarity, child_module)?)?;
    child_module.add_function(wrap_pyfunction!(batch_gene_enrichment, child_module)?)?;
    child_module.add_function(wrap_pyfunction!(batch_disease_enrichment, child_module)?)?;
    parent_module.add_submodule(child_module)?;

    py.import("sys")?
        .getattr("modules")?
        .set_item("pyhpo.helper", child_module)?;
    Ok(())
}

fn register_stats_module(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let child_module = PyModule::new(py, "stats")?;
    child_module.add_class::<PyEnrichmentModel>()?;
    child_module.add_function(wrap_pyfunction!(linkage::linkage, child_module)?)?;
    parent_module.add_submodule(child_module)?;

    py.import("sys")?
        .getattr("modules")?
        .set_item("pyhpo.stats", child_module)?;

    Ok(())
}

/// Calculate similarity between `HPOSet`s in batches
///
/// This method runs parallelized on all avaible CPU
///
/// Examples
/// --------
///
/// .. code-block:: python
///
///     import itertools
///     from pyhpo import Ontology, HPOSet, helper
///
///     Ontology()
///
///     gene_sets = [g.hpo_set() for g in Ontology.genes]
///     gene_set_combinations = [(a[0], a[1]) for a in itertools.combinations(gene_sets,2)]
///     similarities = helper.set_batch_similarity(gene_set_combinations[0:100], kind="omim", method="graphic", combine = "funSimAvg")
///
#[pyfunction]
#[pyo3(signature = (comparisons, kind = "omim", method = "graphic", combine = "funSimAvg"))]
#[pyo3(text_signature = "(comparisons, kind, method, combine)")]
fn set_batch_similarity(
    comparisons: Vec<(PyHpoSet, PyHpoSet)>,
    kind: &str,
    method: &str,
    combine: &str,
) -> PyResult<Vec<f32>> {
    let ont = get_ontology()?;

    let kind = PyInformationContentKind::try_from(kind)?;
    let similarity = hpo::similarity::Builtins::new(method, kind.into())
        .map_err(|_| PyRuntimeError::new_err("Unknown method to calculate similarity"))?;
    let combiner = StandardCombiner::try_from(combine)
        .map_err(|_| PyRuntimeError::new_err("Invalid combine method specified"))?;

    let g_sim = GroupSimilarity::new(combiner, similarity);

    Ok(comparisons
        .par_iter()
        .map(|comp| {
            let set_a = comp.0.set(ont);
            let set_b = comp.1.set(ont);
            g_sim.calculate(&set_a, &set_b)
        })
        .collect())
}

/// Calculate similarity between `HPOTerm`s in batches
///
/// This method runs parallelized on all avaible CPU
///
/// Examples
/// --------
///
/// .. code-block:: python
///
///     import itertools
///     from pyhpo import Ontology, HPOSet, helper
///
///     Ontology()
///
///     terms = [t for t in Ontology]
///     term_combinations = [(a[0], a[1]) for a in itertools.combinations(terms,2)]
///     similarities = helper.batch_similarity(term_combinations[0:10000], kind="omim", method="graphic")
///
#[pyfunction]
#[pyo3(signature = (comparisons, kind = "omim", method = "graphic"))]
#[pyo3(text_signature = "(comparisons, kind, method)")]
fn batch_similarity(
    comparisons: Vec<(PyHpoTerm, PyHpoTerm)>,
    kind: &str,
    method: &str,
) -> PyResult<Vec<f32>> {
    let kind = PyInformationContentKind::try_from(kind)?;
    let similarity = hpo::similarity::Builtins::new(method, kind.into())
        .map_err(|_| PyRuntimeError::new_err("Unknown method to calculate similarity"))?;

    Ok(comparisons
        .par_iter()
        .map(|comp| {
            let t1: hpo::HpoTerm = (&comp.0).into();
            let t2: hpo::HpoTerm = (&comp.1).into();
            similarity.calculate(&t1, &t2)
        })
        .collect())
}

#[pyfunction]
fn batch_gene_enrichment(py: Python, hposets: Vec<PyHpoSet>) -> PyResult<Vec<Vec<&PyDict>>> {
    let ont = get_ontology()?;
    let enrichments = hposets
        .par_iter()
        .map(|pyset| {
            let mut enrichment = gene_enrichment(ont, &pyset.set(ont));
            enrichment.sort_by(|a, b| a.pvalue().partial_cmp(&b.pvalue()).unwrap());
            enrichment
        })
        .collect::<Vec<Vec<hpo::stats::Enrichment<GeneId>>>>();

    enrichments
        .iter()
        .map(|set| {
            set.iter()
                .map(|enrichment| crate::enrichment::gene_enrichment_dict(py, enrichment))
                .collect::<PyResult<Vec<&PyDict>>>()
        })
        .collect::<PyResult<Vec<Vec<&PyDict>>>>()
}

#[pyfunction]
fn batch_disease_enrichment(py: Python, hposets: Vec<PyHpoSet>) -> PyResult<Vec<Vec<&PyDict>>> {
    let ont = get_ontology()?;
    let enrichments = hposets
        .par_iter()
        .map(|pyset| {
            let mut enrichment = disease_enrichment(ont, &pyset.set(ont));
            enrichment.sort_by(|a, b| a.pvalue().partial_cmp(&b.pvalue()).unwrap());
            enrichment
        })
        .collect::<Vec<Vec<hpo::stats::Enrichment<OmimDiseaseId>>>>();

    enrichments
        .iter()
        .map(|set| {
            set.iter()
                .map(|enrichment| crate::enrichment::disease_enrichment_dict(py, enrichment))
                .collect::<PyResult<Vec<&PyDict>>>()
        })
        .collect::<PyResult<Vec<Vec<&PyDict>>>>()
}
