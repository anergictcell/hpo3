use hpo::annotations::{Disease, OrphaDiseaseId};
use hpo::annotations::{GeneId, OmimDiseaseId};
use pyo3::exceptions::PyNotImplementedError;
use pyo3::types::PyDict;
use pyo3::{exceptions::PyKeyError, prelude::*};

use hpo::stats::hypergeom::{gene_enrichment, omim_disease_enrichment, orpha_disease_enrichment};

use crate::annotations::{PyGene, PyOmimDisease, PyOrphaDisease};
use crate::get_ontology;
use crate::set::PyHpoSet;

#[derive(Clone)]
enum EnrichmentType {
    Gene,
    Omim,
    Orpha,
}

/// Calculate the hypergeometric enrichment of genes
/// or diseases in a set of HPO terms
///
/// Parameters
/// ----------
/// category: str
///     Specify ``gene``, ``omim`` or ``orpha`` to determine which enrichments to calculate
///
/// Raises
/// ------
/// KeyError
///     Invalid category, only ``gene``, ``omim`` or ``orpha`` are possible
///
/// Examples
/// --------
///
/// .. code-block:: python
///
///     from pyhpo import Ontology, Gene, Omim
///     from pyhpo import stats
///
///     Ontology()
///     model = stats.EnrichmentModel("omim")
///
///     # use the `model.enrichment` method to calculate
///     # the enrichment of Omim Diseases within an HPOSet
///
#[pyclass(name = "EnrichmentModel")]
#[derive(Clone)]
pub(crate) struct PyEnrichmentModel {
    kind: EnrichmentType,
}

#[pymethods]
impl PyEnrichmentModel {
    /// Returns a new `EnrichmentModel` to calculate enrichment
    /// for either Genes, Omim or Orpha Diseases
    ///
    /// Parameters
    /// ----------
    /// category: str
    ///     Specify ``gene``, ``omim`` or ``orpha`` to determine which enrichments to calculate
    ///
    /// Raises
    /// ------
    /// KeyError
    ///     Invalid category, only ``gene``, ``omim`` or ``orpha`` are possible
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology, Gene, Omim
    ///     from pyhpo import stats
    ///
    ///     Ontology()
    ///     model = stats.EnrichmentModel("omim")
    ///
    ///     # use the `model.enrichment` method to calculate
    ///     # the enrichment of Omim Diseases within an HPOSet
    ///
    #[new]
    fn new(category: &str) -> PyResult<Self> {
        let kind = match category {
            "gene" => EnrichmentType::Gene,
            "omim" => EnrichmentType::Omim,
            "orpha" => EnrichmentType::Orpha,
            _ => return Err(PyKeyError::new_err("kind")),
        };
        Ok(PyEnrichmentModel { kind })
    }

    /// Calculate the enrichment for all genes or diseeases in the `HPOSet`
    ///
    /// Parameters
    /// ----------
    /// method: `str`
    ///     Currently, only `hypergeom` is implemented
    /// hposet: :class:`pyhpo.HPOSet`
    ///     The set of HPOTerms to use as sampleset for calculation of
    ///     enrichment. The full ontology is used as background set.
    ///
    /// Returns
    /// -------
    /// list[dict]
    ///     a list with dict that contain data about the enrichment, with the keys:
    ///
    ///     * **enrichment** : `float`
    ///         The hypergeometric enrichment score
    ///     * **fold** : `float`
    ///         The fold enrichment
    ///     * **count** : `int`
    ///         Number of occurrences
    ///     * **item** : `Gene` :class:`pyhpo.Gene`, :class:`pyhpo.Omim` or :class:`pyhpo.Orpha`
    ///         The actual enriched gene or disease
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
    /// NotImplementedError
    ///     invalid ``method`` provided, only ``hypergeom`` is implemented
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology, Gene, Omim
    ///     from pyhpo import stats
    ///
    ///     Ontology()
    ///     model = stats.EnrichmentModel("omim")
    ///
    ///     # you can crate a custom HPOset or use a Gene or Disease
    ///     term_set = Gene.get("GBA1").hpo_set()
    ///
    ///     enriched_diseases = model.enrichment("hypergeom", term_set)
    ///
    ///     enriched_diseases[0]
    ///
    ///     # >> {
    ///     # >>     "enrichment": 7.708086517543451e-223,
    ///     # >>     "fold": 27.44879391414045,
    ///     # >>     "count": 164,
    ///     # >>     "item": <OmimDisease (608013)>
    ///     # >> }
    ///
    ///
    #[pyo3(text_signature = "($self, method, hposet)")]
    fn enrichment<'a>(
        &self,
        py: Python<'a>,
        method: &str,
        hposet: &PyHpoSet,
    ) -> PyResult<Vec<Bound<'a, PyDict>>> {
        let ont = get_ontology()?;
        let set = hposet.set(ont);

        if method != "hypergeom" {
            // we currently only implement hypergeometric enrichment.
            // Once we support more methods, we should refactor this method
            // accordingly.
            return Err(PyNotImplementedError::new_err(
                "Enrichment method not implemented",
            ));
        };

        let res = match self.kind {
            EnrichmentType::Gene => {
                let mut enr = gene_enrichment(ont, &set);
                enr.sort_by(|a, b| a.pvalue().partial_cmp(&b.pvalue()).unwrap());
                enr.iter()
                    .map(|enrichment| gene_enrichment_dict(py, enrichment))
                    .collect::<PyResult<Vec<Bound<'a, PyDict>>>>()
            }
            EnrichmentType::Omim => {
                let mut enr = omim_disease_enrichment(ont, &set);
                enr.sort_by(|a, b| a.pvalue().partial_cmp(&b.pvalue()).unwrap());
                enr.iter()
                    .map(|enrichment| omim_disease_enrichment_dict(py, enrichment))
                    .collect::<PyResult<Vec<Bound<'a, PyDict>>>>()
            }
            EnrichmentType::Orpha => {
                let mut enr = orpha_disease_enrichment(ont, &set);
                enr.sort_by(|a, b| a.pvalue().partial_cmp(&b.pvalue()).unwrap());
                enr.iter()
                    .map(|enrichment| orpha_disease_enrichment_dict(py, enrichment))
                    .collect::<PyResult<Vec<Bound<'a, PyDict>>>>()
            }
        };
        res
    }
}

/// Returns the disease enrichment data as a Python dict
///
/// # Errors
///
/// - PyNameError: Ontology not yet constructed
pub(crate) fn omim_disease_enrichment_dict<'a, T>(
    py: Python<'a>,
    enrichment: &hpo::stats::Enrichment<T>,
) -> PyResult<Bound<'a, PyDict>>
where
    T: std::fmt::Display + hpo::annotations::AnnotationId,
{
    let disease = get_ontology()?
        .omim_disease(&OmimDiseaseId::from(enrichment.id().as_u32()))
        .map(|d| PyOmimDisease::new(*d.id(), d.name().into()))
        .unwrap();
    let dict = PyDict::new_bound(py);
    dict.set_item("enrichment", enrichment.pvalue())?;
    dict.set_item("fold", enrichment.enrichment())?;
    dict.set_item("count", enrichment.count())?;
    dict.set_item("item", disease.into_py(py))?;
    Ok(dict)
}

/// Returns the disease enrichment data as a Python dict
///
/// # Errors
///
/// - PyNameError: Ontology not yet constructed
pub(crate) fn orpha_disease_enrichment_dict<'a, T>(
    py: Python<'a>,
    enrichment: &hpo::stats::Enrichment<T>,
) -> PyResult<Bound<'a, PyDict>>
where
    T: std::fmt::Display + hpo::annotations::AnnotationId,
{
    let disease = get_ontology()?
        .orpha_disease(&OrphaDiseaseId::from(enrichment.id().as_u32()))
        .map(|d| PyOrphaDisease::new(*d.id(), d.name().into()))
        .unwrap();
    let dict = PyDict::new_bound(py);
    dict.set_item("enrichment", enrichment.pvalue())?;
    dict.set_item("fold", enrichment.enrichment())?;
    dict.set_item("count", enrichment.count())?;
    dict.set_item("item", disease.into_py(py))?;
    Ok(dict)
}

/// Returns the gene enrichment data as a Python dict
///
/// # Errors
///
/// - PyNameError: Ontology not yet constructed
pub(crate) fn gene_enrichment_dict<'a, T>(
    py: Python<'a>,
    enrichment: &hpo::stats::Enrichment<T>,
) -> PyResult<Bound<'a, PyDict>>
where
    T: std::fmt::Display + hpo::annotations::AnnotationId,
{
    let gene = get_ontology()?
        .gene(&GeneId::from(enrichment.id().as_u32()))
        .map(|g| PyGene::new(*g.id(), g.name().into()))
        .unwrap();
    let dict = PyDict::new_bound(py);
    dict.set_item("enrichment", enrichment.pvalue())?;
    dict.set_item("fold", enrichment.enrichment())?;
    dict.set_item("count", enrichment.count())?;
    dict.set_item("item", gene.into_py(py))?;
    Ok(dict)
}
