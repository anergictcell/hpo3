use pyo3::exceptions::{PyKeyError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;

use once_cell::sync::OnceCell;
use rayon::prelude::*;

use hpo::similarity::{GraphIc, GroupSimilarity, StandardCombiner};
use hpo::term::{HpoGroup, HpoTermId};
use hpo::{HpoTerm, Ontology as ActualOntology};

mod annotations;
mod information_content;
mod ontology;
mod set;
mod term;

use crate::annotations::{PyGene, PyOmimDisease};
use crate::information_content::{PyInformationContent, PyInformationContentKind};
use crate::ontology::PyOntology;
use crate::set::PyHpoSet;
use crate::term::PyHpoTerm;

static ONTOLOGY: OnceCell<ActualOntology> = OnceCell::new();

fn from_binary(path: &str) -> usize {
    let ont = ActualOntology::from_binary(path).unwrap();
    ONTOLOGY.set(ont).unwrap();
    ONTOLOGY.get().unwrap().len()
}

fn from_obo(path: &str) -> usize {
    let ont = ActualOntology::from_standard(path).unwrap();
    ONTOLOGY.set(ont).unwrap();
    ONTOLOGY.get().unwrap().len()
}

fn get_ontology() -> PyResult<&'static ActualOntology> {
    ONTOLOGY.get().ok_or_else(|| {
        pyo3::exceptions::PyNameError::new_err(
            "You must initialize the ontology first: `ont = hpo3.Ontology()`",
        )
    })
}

fn pyterm_from_id(id: u32) -> PyResult<PyHpoTerm> {
    let term = term_from_id(id)?;
    Ok(PyHpoTerm::new(term.id(), term.name().to_string()))
}

fn term_from_id(id: u32) -> PyResult<hpo::HpoTerm<'static>> {
    let ont = get_ontology()?;
    match ont.hpo(id.into()) {
        Some(term) => Ok(term),
        None => Err(PyKeyError::new_err(format!("No HPOTerm for index {}", id))),
    }
}

fn term_from_query(query: PyQuery) -> PyResult<HpoTerm<'static>> {
    match query {
        PyQuery::Id(id) => return term_from_id(id),
        PyQuery::Str(term_name) => {
            if term_name.starts_with("HP:") {
                match HpoTermId::try_from(term_name.as_str()) {
                    Ok(termid) => return term_from_id(termid.as_u32()),
                    Err(_) => {
                        return Err(PyValueError::new_err(format!(
                            "Invalid id: {}",
                            term_name
                        )))
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

#[pyclass]
pub struct Helpers {}

#[pymethods]
impl Helpers {
    /// Calculates the similarity score of two HPO Term Sets
    ///
    /// Arguments
    /// ---------
    /// a: `List[int]`
    ///     IDs of the terms in Group-A as `int` (`HP:0000123` --> `123`)
    /// b: `List[int]`
    ///     IDs of the terms in Group-B as `int` (`HP:0000123` --> `123`)
    /// kind: :class:`hpo3.InformationContentKind`
    ///     Which kind of information content to use for similarity calculation
    ///     default: :class:`hpo3.InformationContentKind.Omim`
    ///
    /// Returns
    /// -------
    /// `float`
    ///     The similarity score
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from hpo3 import Ontology
    ///     from hpo3 import InformationContentKind
    ///
    ///     ont = Ontology()
    ///
    ///     # compare two groups using default information content (OMIM):
    ///     # - HP:0011968, HP:0001743
    ///     # - HP:0000119, HP:0000121, HP:0000122
    ///     ont.group_similarity([11968, 1743], [119, 121, 122])
    ///
    ///     # compare two groups using Gene:
    ///     ont.group_similarity([11968, 1743], [119, 121, 122], InformationContentKind.Gene)
    ///
    #[args(kind = "PyInformationContentKind::Omim")]
    #[pyo3(text_signature = "($self, a, b, kind)")]
    fn group_similarity(&self, a: Vec<u32>, b: Vec<u32>, kind: PyInformationContentKind) -> f32 {
        let combiner = StandardCombiner::FunSimAvg;
        let similarity = GraphIc::new(kind.into());
        let sim = GroupSimilarity::new(combiner, similarity);
        let ont = ONTOLOGY.get().unwrap();

        let mut group_a = HpoGroup::default();
        for t in a {
            group_a.insert(HpoTermId::try_from(t).expect("Invalid HpoTermId"));
        }

        let mut group_b = HpoGroup::default();
        for t in b {
            group_b.insert(HpoTermId::try_from(t).expect("Invalid HpoTermId"));
        }

        sim.calculate(
            &hpo::HpoSet::new(ont, group_a),
            &hpo::HpoSet::new(ont, group_b),
        )
    }

    /// Calculates the similarity score of one HPO Term Set to many other sets
    ///
    /// This method uses the `rayon` parallel iterator and will utilize all available CPU cores
    ///
    /// Arguments
    /// ---------
    /// a: `List[int]`
    ///     IDs of the terms in Group-A as `int` (`HP:0000123` --> `123`)
    /// b: `List[List[int]]`
    ///     List of Groups, each being a List of IDs of the terms as `int` (`HP:0000123` --> `123`)
    /// kind: :class:`hpo3.InformationContentKind`
    ///     Which kind of information content to use for similarity calculation
    ///     default: :class:`hpo3.InformationContentKind.Omim`
    ///
    /// Returns
    /// -------
    /// `List[float]`
    ///     The similarity scores for every a - b comparison
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from hpo3 import Ontology
    ///     from hpo3 import InformationContentKind
    ///
    ///     ont = Ontology()
    ///
    ///     # compare HP:0011968, HP:0001743 to
    ///     # - HP:0000119, HP:0000121, HP:0000122
    ///     # - HP:0000113, HP:0000124, HP:0000125
    ///     ont.group_similarity(
    ///         [11968, 1743],
    ///         [
    ///             [119, 121, 122],
    ///             [113, 124, 125]
    ///         ]
    ///     )
    ///
    ///     # use Gene information content
    ///     ont.group_similarity(
    ///         [11968, 1743],
    ///         [
    ///             [119, 121, 122],
    ///             [113, 124, 125]
    ///         ],
    ///         InformationContentKind.Gene
    ///     )
    ///
    #[args(kind = "PyInformationContentKind::Omim")]
    #[pyo3(text_signature = "($self, a, b, kind)")]
    fn group_similarity_batch(
        &self,
        a: Vec<u32>,
        b: Vec<Vec<u32>>,
        kind: PyInformationContentKind,
    ) -> Vec<f32> {
        let combiner = StandardCombiner::FunSimAvg;
        let similarity = GraphIc::new(kind.into());
        let sim = GroupSimilarity::new(combiner, similarity);
        let ont = ONTOLOGY.get().unwrap();

        let mut group_a = HpoGroup::default();
        for t in a {
            group_a.insert(HpoTermId::try_from(t).expect("Invalid HpoTermId"));
        }
        let set_a = hpo::HpoSet::new(ont, group_a);

        b.into_iter()
            .par_bridge()
            .map(|set| {
                let mut group_b = HpoGroup::default();
                for t in set {
                    group_b.insert(HpoTermId::try_from(t).expect("Invalid HpoTermId"));
                }
                sim.calculate(&set_a, &hpo::HpoSet::new(ont, group_b))
            })
            .collect()
    }

    /// Compares the provided HPO Term set with all OMIM diseases and returns the similarity scores
    ///
    /// This method uses the `rayon` parallel iterator and will utilize all available CPU cores
    ///
    /// Arguments
    /// ---------
    /// term_ids: `List[int]`
    ///     IDs of the terms in the Set as `int` (`HP:0000123` --> `123`)
    /// kind: :class:`hpo3.InformationContentKind`
    ///     Which kind of information content to use for similarity calculation
    ///     default: :class:`hpo3.InformationContentKind.Omim`
    ///
    /// Returns
    /// -------
    /// `List[float]`
    ///     The similarity scores for every disease
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from hpo3 import Ontology
    ///     from hpoer import InformationContent
    ///
    ///     ont = Ontology()
    ///
    ///     # returns similarity scores to each disease, calculated based on default (Omim)
    ///     res = ont.disease_similarity([11968, 1743])
    ///
    ///     # returns similarity scores to each disease, calculated based on `Gene`
    ///     res = ont.disease_similarity([11968, 1743], InformationContent.Gene)
    ///
    #[args(kind = "PyInformationContentKind::Omim")]
    #[pyo3(text_signature = "($self, term_ids, kind)")]
    fn disease_similarity(
        &self,
        term_ids: Vec<u32>,
        kind: PyInformationContentKind,
    ) -> Vec<(&str, String, f32)> {
        let combiner = StandardCombiner::FunSimAvg;
        let similarity = GraphIc::new(kind.into());
        let sim = GroupSimilarity::new(combiner, similarity);
        let ont = ONTOLOGY.get().unwrap();

        let set_a = hpo::HpoSet::new(ont, HpoGroup::from(term_ids));

        let mut all: Vec<(&str, String, f32)> = ont
            .omim_diseases()
            .par_bridge()
            .map(|disease| {
                let res = sim.calculate(&set_a, &disease.to_hpo_set(ont));
                (disease.name(), disease.id().to_string(), res)
            })
            .collect();
        all.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        all
    }

    /// Compares the provided HPO Term set with all genes and returns the similarity scores
    ///
    /// This method uses the `rayon` parallel iterator and will utilize all available CPU cores
    ///
    /// Arguments
    /// ---------
    /// term_ids: `List[int]`
    ///     IDs of the terms in the Set as `int` (`HP:0000123` --> `123`)
    /// kind: :class:`hpo3.InformationContentKind`
    ///     Which kind of information content to use for similarity calculation
    ///     default: :class:`hpo3.InformationContentKind.Omim`
    ///
    /// Returns
    /// -------
    /// `List[float]`
    ///     The similarity scores for every gene
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from hpo3 import Ontology
    ///     ont = Ontology()
    ///
    ///     # returns similarity scores to each gene, calculated based on default (Omim)
    ///     res = ont.gene_similarity([11968, 1743])
    ///
    ///     # returns similarity scores to each gene, calculated based on `Gene`
    ///     res = ont.gene_similarity([11968, 1743], InformationContent.Gene)
    ///
    #[args(kind = "PyInformationContentKind::Omim")]
    #[pyo3(text_signature = "($self, term_ids, kind)")]
    fn gene_similarity(
        &self,
        term_ids: Vec<u32>,
        kind: PyInformationContentKind,
    ) -> Vec<(&str, f32)> {
        let combiner = StandardCombiner::FunSimAvg;
        let similarity = GraphIc::new(kind.into());
        let sim = GroupSimilarity::new(combiner, similarity);
        let ont = ONTOLOGY.get().unwrap();
        let mut group = HpoGroup::default();
        for t in term_ids {
            group.insert(HpoTermId::try_from(t).expect("Invalid HpoTermId"));
        }
        let set_a = hpo::HpoSet::new(ont, group);
        let mut all: Vec<(&str, f32)> = ont
            .genes()
            .par_bridge()
            .map(|gene| {
                let res = sim.calculate(&set_a, &gene.to_hpo_set(ont));
                (gene.name(), res)
            })
            .collect();
        all.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        all
    }
}

#[derive(FromPyObject)]
pub enum PyQuery {
    Id(u32),
    Str(String),
}

/// Python bindings for the Rust hpo crate
///
/// This library aims to be a drop-in replacement for
/// [`pyhpo`](https://pypi.org/project/pyhpo/)
#[pymodule]
fn hpo3(_py: Python, m: &PyModule) -> PyResult<()> {
    let ont = PyOntology::blank();
    m.add_class::<PyGene>()?;
    m.add_class::<PyOmimDisease>()?;
    m.add_class::<PyHpoSet>()?;
    m.add_class::<PyHpoTerm>()?;
    m.add("Ontology", ont)?;
    Ok(())
}
