use annotations::PyOrphaDisease;
use once_cell::sync::OnceCell;

use rayon::prelude::*;

use pyo3::exceptions::{PyKeyError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use hpo::annotations::{AnnotationId, GeneId, OmimDiseaseId, OrphaDiseaseId};
use hpo::similarity::{GroupSimilarity, Similarity, StandardCombiner};
use hpo::stats::hypergeom::{gene_enrichment, omim_disease_enrichment, orpha_disease_enrichment};
use hpo::term::HpoTermId;
use hpo::{HpoResult, HpoTerm, Ontology as ActualOntology};

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
fn from_obo(path: &str, transitive: bool) -> HpoResult<usize> {
    let ont = if transitive {
        ActualOntology::from_standard_transitive(path)?
    } else {
        ActualOntology::from_standard(path)?
    };
    ONTOLOGY.set(ont).unwrap();
    Ok(ONTOLOGY.get().unwrap().len())
}

/// Returns a reference to the Ontology
///
/// This method only works **after** building the ontology
///
/// # Errors
///
/// - PyNameError: Ontology not yet constructed
fn get_ontology() -> PyResult<&'static ActualOntology> {
    ONTOLOGY.get().ok_or_else(|| {
        pyo3::exceptions::PyNameError::new_err(
            "You must build the ontology first: `>> pyhpo.Ontology()`",
        )
    })
}

/// Returns a [`PyHpoTerm`] from a `u32` ID
///
/// # Errors
///
/// - PyKeyError: No term with that ID present in Ontology
/// - PyNameError: Ontology not yet constructed
fn pyterm_from_id(id: u32) -> PyResult<PyHpoTerm> {
    let term = term_from_id(id)?;
    Ok(PyHpoTerm::new(term.id(), term.name().to_string()))
}

/// Returns an [`HpoTerm`] from a `u32` ID
///
/// # Errors
///
/// - PyKeyError: No term with that ID present in Ontology
/// - PyNameError: Ontology not yet constructed
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
/// - PyValueError: query cannot be converted to HpoTermId
/// - PyRuntimeError: query is a name and does not have a match in the Ontology
/// - PyNameError: Ontology not yet constructed
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
fn pyhpo(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    let ont = PyOntology::blank();
    m.add_class::<PyGene>()?;
    m.add_class::<PyOmimDisease>()?;
    m.add_class::<PyOrphaDisease>()?;
    m.add_class::<PyHpoSet>()?;
    m.add_class::<PyHpoTerm>()?;
    m.add_class::<PyEnrichmentModel>()?;
    m.add_class::<PyInformationContent>()?;
    m.add_class::<PyOntology>()?;
    m.add_function(wrap_pyfunction!(linkage::linkage, m)?)?;
    m.add("Ontology", ont)?;
    m.add("BasicHPOSet", set::BasicPyHpoSet)?;
    m.add("HPOPhenoSet", set::PhenoSet)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__backend__", env!("CARGO_PKG_NAME"))?;
    m.add_function(wrap_pyfunction!(batch_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(batch_set_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(batch_gene_enrichment, m)?)?;
    m.add_function(wrap_pyfunction!(batch_disease_enrichment, m)?)?;
    m.add_function(wrap_pyfunction!(batch_omim_disease_enrichment, m)?)?;
    m.add_function(wrap_pyfunction!(batch_orpha_disease_enrichment, m)?)?;
    Ok(())
}

/// Calculate similarity between ``HPOSet`` in batches
///
/// This method runs parallelized on all avaible CPU
///
/// Parameters
/// ----------
/// comparisons: list[tuple[:class:`pyhpo.HPOSet`, :class:`pyhpo.HPOSet`]]
///     A list of ``HPOSet`` tuples. The two ``HPOSet`` within one tuple will
///     be compared to each other.
/// kind: str, default: ``omim``
///     Which kind of information content to use for similarity calculation
///
///     Available options:
///
///     * **omim**
///     * **orpha**
///     * **gene**
///
/// method: str, default ``graphic``
///     The method to use to calculate the similarity.
///
///     Available options:
///
///     * **resnik** - Resnik P, Proceedings of the 14th IJCAI, (1995)
///     * **lin** - Lin D, Proceedings of the 15th ICML, (1998)
///     * **jc** - Jiang J, Conrath D, ROCLING X, (1997)
///       This is different to PyHPO
///     * **jc2** - Jiang J, Conrath D, ROCLING X, (1997)
///       Same as `jc`, but kept for backwards compatibility
///     * **rel** - Relevance measure - Schlicker A, et.al.,
///       BMC Bioinformatics, (2006)
///     * **ic** - Information coefficient - Li B, et. al., arXiv, (2010)
///     * **graphic** - Graph based Information coefficient -
///       Deng Y, et. al., PLoS One, (2015)
///     * **dist** - Distance between terms
///
/// Returns
/// -------
/// list[float]
///     The similarity scores of each comparison
///
/// Raises
/// ------
/// NameError
///     Ontology not yet constructed
/// KeyError
///     Invalid ``kind`` provided
/// RuntimeError
///     Invalid ``method`` or ``combine``
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
///     similarities = helper.batch_set_similarity(gene_set_combinations[0:100], kind="omim", method="graphic", combine = "funSimAvg")
///
#[pyfunction]
#[pyo3(signature = (comparisons, kind = "omim", method = "graphic", combine = "funSimAvg"))]
#[pyo3(text_signature = "(comparisons, kind, method, combine)")]
fn batch_set_similarity(
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

/// Calculate similarity between ``HPOTerm`` in batches
///
/// This method runs parallelized on all avaible CPU
///
/// Parameters
/// ----------
/// comparisons: list[tuple[:class:`pyhpo.HPOTerm`, :class:`pyhpo.HPOTerm`]]
///     A list of ``HPOTerm`` tuples. The two ``HPOTerm`` within one tuple will
///     be compared to each other.
///
/// kind: str, default: ``omim``
///     Which kind of information content to use for similarity calculation
///
///     Available options:
///
///     * **omim**
///     * **orpha**
///     * **gene**
///
/// method: str, default ``graphic``
///     The method to use to calculate the similarity.
///
///     Available options:
///
///     * **resnik** - Resnik P, Proceedings of the 14th IJCAI, (1995)
///     * **lin** - Lin D, Proceedings of the 15th ICML, (1998)
///     * **jc** - Jiang J, Conrath D, ROCLING X, (1997)
///       This is different to PyHPO
///     * **jc2** - Jiang J, Conrath D, ROCLING X, (1997)
///       Same as `jc`, but kept for backwards compatibility
///     * **rel** - Relevance measure - Schlicker A, et.al.,
///       BMC Bioinformatics, (2006)
///     * **ic** - Information coefficient - Li B, et. al., arXiv, (2010)
///     * **graphic** - Graph based Information coefficient -
///       Deng Y, et. al., PLoS One, (2015)
///     * **dist** - Distance between terms
///
/// Returns
/// -------
/// list[float]
///     The similarity scores of each comparison
///
/// Raises
/// ------
/// KeyError
///     Invalid ``kind`` provided
/// RuntimeError
///     Invalid ``method``
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

/// Calculate enriched genes in a list of ``HPOSet``
///
/// This method runs parallelized on all avaible CPU
///
/// Calculate hypergeometric enrichment of genes associated to the terms
/// of each set. Each set is calculated individually, the returning list has
/// the same order as the input data.
///
/// Parameters
/// ----------
/// hposets: list[:class:`pyhpo.HPOSet`]
///     A list of HPOSets. The enrichment of all genes is calculated separately
///     for each HPOset in the list
///
/// Returns
/// -------
/// list[dict]
///     The enrichment result for every gene.
///     See :func:`pyhpo.stats.EnrichmentModel.enrichment` for details
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
///     from pyhpo import Ontology, helper
///
///     Ontology()
///
///     diseases = [d for d in Ontology.omim_diseases[0:100]]
///     disease_sets = [d.hpo_set() for d in diseases]
///     enrichments = helper.batch_gene_enrichment(disease_sets)
///
///     for (disease, enriched_genes) in zip(diseases, enrichments):
///         print(
///             "The top enriched genes for {} are: {}".format(
///                 disease.name,
///                 ", ".join([f"{gene['item'].name}, ({gene['enrichment']})" for gene in enriched_genes[0:5]])
///             )
///         )
///
///     # >>> The top enriched genes for Immunodeficiency 85 and autoimmunity are: TOM1, (7.207370728788139e-45), PIK3CD, (1.9560156243742087e-17), IL2RG, (1.0000718026169596e-16), BACH2, (3.373013104581288e-15), IL6ST, (3.760565282680126e-15)
///     # >>> The top enriched genes for CODAS syndrome are: LONP1, (4.209128613268585e-80), EXTL3, (5.378742851736401e-23), SMC1A, (5.338807361962185e-22), FLNA, (1.0968887647112733e-21), COL2A1, (1.1029731783630839e-21)
///     # >>> The top enriched genes for Rhizomelic chondrodysplasia punctata, type 1 are: PEX7, (9.556919089648523e-54), PEX5, (7.030392607093173e-22), PEX1, (3.7973830291601626e-19), PEX11B, (4.318791413029623e-19), HSPG2, (7.108950838424571e-19)
///     # >>> The top enriched genes for Oculopharyngodistal myopathy 4 are: RILPL1, (1.4351489331895004e-49), LRP12, (2.168165858699749e-30), GIPC1, (3.180801819975307e-27), NOTCH2NLC, (1.0700847991253517e-23), VCP, (2.8742020666947536e-20)
///
#[pyfunction]
fn batch_gene_enrichment(
    py: Python,
    hposets: Vec<PyHpoSet>,
) -> PyResult<Vec<Vec<Bound<'_, PyDict>>>> {
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
                .collect::<PyResult<Vec<Bound<'_, PyDict>>>>()
        })
        .collect::<PyResult<Vec<Vec<Bound<'_, PyDict>>>>>()
}

/// Deprecated since 1.3.0
///
/// Use :func:`pyhpo.helper.batch_omim_disease_enrichment` or
/// :func:`pyhpo.helper.batch_orpha_disease_enrichment` instead
#[pyfunction]
fn batch_disease_enrichment(
    py: Python,
    hposets: Vec<PyHpoSet>,
) -> PyResult<Vec<Vec<Bound<'_, PyDict>>>> {
    batch_omim_disease_enrichment(py, hposets)
}

/// Calculate enriched Omim diseases in a list of ``HPOSet``
///
/// This method runs parallelized on all avaible CPU
///
/// Calculate the hypergeometric enrichment of Omim diseases associated to the terms
/// of each set. Each set is calculated individually, the returning list has
/// the same order as the input data.
///
/// Parameters
/// ----------
/// hposets: list[:class:`pyhpo.HPOSet`]
///     A list of HPOSets. The enrichment of all diseases is calculated separately
///     for each HPOset in the list
///
/// Returns
/// -------
/// list[dict]
///     The enrichment result for every disease.
///     See :func:`pyhpo.stats.EnrichmentModel.enrichment` for details
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
///     from pyhpo import Ontology, helper
///
///     Ontology()
///
///     genes = [g for g in Ontology.genes[0:100]]
///     gene_sets = [g.hpo_set() for g in genes]
///     enrichments = helper.batch_omim_disease_enrichment(gene_sets)
///
///     for (gene, enriched_diseases) in zip(genes, enrichments):
///         print(
///             "The top enriched diseases for {} are: {}".format(
///                 gene.name,
///                 ", ".join([f"{disease['item'].name}, ({disease['enrichment']})" for disease in enriched_diseases[0:5]])
///             )
///         )
///
///     # >>> The top enriched diseases for C7 are: C7 deficiency, (3.6762699175625894e-42), C6 deficiency, (3.782313673973149e-37), C5 deficiency, (2.6614254464758174e-33), Complement factor B deficiency, (4.189056541495023e-32), Complement component 8 deficiency, type II, (8.87368759499919e-32)
///     # >>> The top enriched diseases for WNT5A are: Robinow syndrome, autosomal recessive, (0.0), Robinow syndrome, autosomal dominant 1, (0.0), Pallister-Killian syndrome, (1.2993558687813034e-238), Robinow syndrome, autosomal dominant 3, (1.2014167106834296e-223), Peters-plus syndrome, (2.5163107554882648e-216)
///     # >>> The top enriched diseases for TYMS are: Dyskeratosis congenita, X-linked, (5.008058437787544e-192), Dyskeratosis congenita, digenic, (2.703378203105612e-184), Dyskeratosis congenita, autosomal dominant 2, (1.3109083102058795e-150), Bloom syndrome, (3.965926308699221e-141), Dyskeratosis congenita, autosomal dominant 3, (1.123439117889186e-131)
///
#[pyfunction]
fn batch_omim_disease_enrichment(
    py: Python,
    hposets: Vec<PyHpoSet>,
) -> PyResult<Vec<Vec<Bound<'_, PyDict>>>> {
    let ont = get_ontology()?;
    let enrichments = hposets
        .par_iter()
        .map(|pyset| {
            let mut enrichment = omim_disease_enrichment(ont, &pyset.set(ont));
            enrichment.sort_by(|a, b| a.pvalue().partial_cmp(&b.pvalue()).unwrap());
            enrichment
        })
        .collect::<Vec<Vec<hpo::stats::Enrichment<OmimDiseaseId>>>>();

    enrichments
        .iter()
        .map(|set| {
            set.iter()
                .map(|enrichment| crate::enrichment::omim_disease_enrichment_dict(py, enrichment))
                .collect::<PyResult<Vec<Bound<'_, PyDict>>>>()
        })
        .collect::<PyResult<Vec<Vec<Bound<'_, PyDict>>>>>()
}

/// Calculate enriched Orpha diseases in a list of ``HPOSet``
///
/// This method runs parallelized on all avaible CPU
///
/// Calculate the hypergeometric enrichment of Orpha diseases associated to the terms
/// of each set. Each set is calculated individually, the returning list has
/// the same order as the input data.
///
/// Parameters
/// ----------
/// hposets: list[:class:`pyhpo.HPOSet`]
///     A list of HPOSets. The enrichment of all diseases is calculated separately
///     for each HPOset in the list
///
/// Returns
/// -------
/// list[dict]
///     The enrichment result for every disease.
///     See :func:`pyhpo.stats.EnrichmentModel.enrichment` for details
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
///     from pyhpo import Ontology, helper
///
///     Ontology()
///
///     genes = [g for g in Ontology.genes[0:100]]
///     gene_sets = [g.hpo_set() for g in genes]
///     enrichments = helper.batch_orpha_disease_enrichment(gene_sets)
///
///     for (gene, enriched_diseases) in zip(genes, enrichments):
///         print(
///             "The top enriched diseases for {} are: {}".format(
///                 gene.name,
///                 ", ".join([f"{disease['item'].name}, ({disease['enrichment']})" for disease in enriched_diseases[0:5]])
///             )
///         )
///
///     # >>> The top enriched diseases for C7 are: C7 deficiency, (3.6762699175625894e-42), C6 deficiency, (3.782313673973149e-37), C5 deficiency, (2.6614254464758174e-33), Complement factor B deficiency, (4.189056541495023e-32), Complement component 8 deficiency, type II, (8.87368759499919e-32)
///     # >>> The top enriched diseases for WNT5A are: Robinow syndrome, autosomal recessive, (0.0), Robinow syndrome, autosomal dominant 1, (0.0), Pallister-Killian syndrome, (1.2993558687813034e-238), Robinow syndrome, autosomal dominant 3, (1.2014167106834296e-223), Peters-plus syndrome, (2.5163107554882648e-216)
///     # >>> The top enriched diseases for TYMS are: Dyskeratosis congenita, X-linked, (5.008058437787544e-192), Dyskeratosis congenita, digenic, (2.703378203105612e-184), Dyskeratosis congenita, autosomal dominant 2, (1.3109083102058795e-150), Bloom syndrome, (3.965926308699221e-141), Dyskeratosis congenita, autosomal dominant 3, (1.123439117889186e-131)
///
#[pyfunction]
fn batch_orpha_disease_enrichment(
    py: Python,
    hposets: Vec<PyHpoSet>,
) -> PyResult<Vec<Vec<Bound<'_, PyDict>>>> {
    let ont = get_ontology()?;
    let enrichments = hposets
        .par_iter()
        .map(|pyset| {
            let mut enrichment = orpha_disease_enrichment(ont, &pyset.set(ont));
            enrichment.sort_by(|a, b| a.pvalue().partial_cmp(&b.pvalue()).unwrap());
            enrichment
        })
        .collect::<Vec<Vec<hpo::stats::Enrichment<OrphaDiseaseId>>>>();

    enrichments
        .iter()
        .map(|set| {
            set.iter()
                .map(|enrichment| crate::enrichment::orpha_disease_enrichment_dict(py, enrichment))
                .collect::<PyResult<Vec<Bound<'_, PyDict>>>>()
        })
        .collect::<PyResult<Vec<Vec<Bound<'_, PyDict>>>>>()
}
