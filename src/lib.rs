use once_cell::sync::OnceCell;

use rayon::prelude::*;

use pyo3::exceptions::{PyKeyError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use hpo::annotations::{AnnotationId, GeneId, OmimDiseaseId};
use hpo::similarity::{GroupSimilarity, StandardCombiner};
use hpo::stats::hypergeom::{disease_enrichment, gene_enrichment};
use hpo::term::HpoTermId;
use hpo::{HpoTerm, Ontology as ActualOntology};

mod annotations;
mod enrichment;
mod information_content;
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
            "You must build the ontology first: `ont = hpo3.Ontology()`",
        )
    })
}

/// Returns a [`PyHpoTerm`] from a `u32` ID
fn pyterm_from_id(id: u32) -> PyResult<PyHpoTerm> {
    let term = term_from_id(id)?;
    Ok(PyHpoTerm::new(term.id(), term.name().to_string()))
}

/// Returns an [`HpoTerm`] from a `u32` ID
fn term_from_id(id: u32) -> PyResult<hpo::HpoTerm<'static>> {
    let ont = get_ontology()?;
    match ont.hpo(id) {
        Some(term) => Ok(term),
        None => Err(PyKeyError::new_err(format!("No HPOTerm for index {}", id))),
    }
}

/// Returns an [`HpoTerm`] from a `str` or `u32` query
fn term_from_query(query: PyQuery) -> PyResult<HpoTerm<'static>> {
    match query {
        PyQuery::Id(id) => return term_from_id(id),
        PyQuery::Str(term_name) => {
            if term_name.starts_with("HP:") {
                match HpoTermId::try_from(term_name.as_str()) {
                    Ok(termid) => return term_from_id(termid.as_u32()),
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

/// Python bindings for the Rust hpo crate
///
/// This library aims to be a drop-in replacement for
/// `pyhpo <https://pypi.org/project/pyhpo/>`_
#[pymodule]
fn hpo3(py: Python, m: &PyModule) -> PyResult<()> {
    let ont = PyOntology::blank();
    m.add_class::<PyGene>()?;
    m.add_class::<PyOmimDisease>()?;
    m.add_class::<PyHpoSet>()?;
    m.add_class::<PyHpoTerm>()?;
    m.add("Ontology", ont)?;
    register_helper_module(py, m)?;
    register_stats_module(py, m)?;
    Ok(())
}

fn register_helper_module(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let child_module = PyModule::new(py, "helper")?;
    child_module.add_function(wrap_pyfunction!(set_batch_similarity, child_module)?)?;
    child_module.add_function(wrap_pyfunction!(batch_gene_enrichment, child_module)?)?;
    child_module.add_function(wrap_pyfunction!(batch_disease_enrichment, child_module)?)?;
    parent_module.add_submodule(child_module)?;
    Ok(())
}

fn register_stats_module(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let child_module = PyModule::new(py, "stats")?;
    child_module.add_class::<PyEnrichmentModel>()?;
    parent_module.add_submodule(child_module)?;
    Ok(())
}

/// Calculate `similarity between `HPOSet`s in batches
///
/// This method runs parallelized on all avaible CPU
///
/// Examples
/// --------
///
/// .. code-block:: python
///
///     import itertools
///     from hpo3 import Ontology, HPOSet, helper
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
                .map(|enrichment| crate::enrichment::enrichment_dict(py, enrichment))
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
                .map(|enrichment| crate::enrichment::enrichment_dict(py, enrichment))
                .collect::<PyResult<Vec<&PyDict>>>()
        })
        .collect::<PyResult<Vec<Vec<&PyDict>>>>()
}
