use pyo3::exceptions::PyNotImplementedError;
use pyo3::types::PyDict;
use pyo3::{exceptions::PyKeyError, prelude::*};

use hpo::stats::hypergeom::{disease_enrichment, gene_enrichment};

use crate::get_ontology;
use crate::set::PyHpoSet;

#[derive(Clone)]
enum EnrichmentType {
    Gene,
    Omim,
}

/// Returns a new `EnrichmentModel` to calculate enrichment
/// for either Genes or Omim Diseases
///
/// Parameters
/// ----------
/// category: str
///     Specify `gene` or `omim` to determine which enrichments to calculate
///
/// Examples
/// --------
///
/// .. code-block:: python
///
///     from hpo3 import Ontology, Gene, Omim
///     from hpo3 import stats
///
///     ont = Ontology()
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
    #[new]
    fn new(category: &str) -> PyResult<Self> {
        let kind = match category {
            "gene" => EnrichmentType::Gene,
            "omim" => EnrichmentType::Omim,
            _ => return Err(PyKeyError::new_err("kind")),
        };
        Ok(PyEnrichmentModel { kind })
    }

    /// Calculate the enrichment for all genes or diseeases in the `HPOSet`
    ///
    /// Parameters
    /// ----------
    /// method: str
    ///     Currently, only `hypergeom` is implemented
    /// hposet: :class:`hpo3.HPOSet`
    ///     The set of HPOTerms to use as sampleset for calculation of
    ///     enrichment. The full ontology is used as background set.
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from hpo3 import Ontology, Gene, Omim
    ///     from hpo3 import stats
    ///
    ///     ont = Ontology()
    ///     model = stats.EnrichmentModel("omim")
    ///
    ///     # you can crate a custom HPOset or use a Gene or Disease
    ///     term_set = Gene.get("GBA1").hpo_set()
    ///
    ///     enriched_diseases = model.enrichment("hypergeom", term_set)
    ///
    ///     # currently, the result only contains the ID, so you must
    ///     # get the actual disease from `OmimDisease`
    ///     top_disease = Omim.get(enriched_diseases[0]["item"])
    ///
    #[pyo3(text_signature = "($self, method, hposet)")]
    fn enrichment<'a>(
        &self,
        py: Python<'a>,
        method: &str,
        hposet: &PyHpoSet,
    ) -> PyResult<Vec<&'a PyDict>> {
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
                    .map(|enrichment| enrichment_dict(py, enrichment))
                    .collect::<PyResult<Vec<&PyDict>>>()
            }
            EnrichmentType::Omim => {
                let mut enr = disease_enrichment(ont, &set);
                enr.sort_by(|a, b| a.pvalue().partial_cmp(&b.pvalue()).unwrap());
                enr.iter()
                    .map(|enrichment| enrichment_dict(py, enrichment))
                    .collect::<PyResult<Vec<&PyDict>>>()
            }
        };
        res
    }
}

pub(crate) fn enrichment_dict<'a, T>(
    py: Python<'a>,
    enrichment: &hpo::stats::Enrichment<T>,
) -> PyResult<&'a PyDict>
where
    T: std::fmt::Display + hpo::annotations::AnnotationId,
{
    let dict = PyDict::new(py);
    dict.set_item("enrichment", enrichment.pvalue())?;
    dict.set_item("fold", enrichment.enrichment())?;
    dict.set_item("count", enrichment.count())?;
    dict.set_item("item", enrichment.id().as_u32())?;
    Ok(dict)
}
