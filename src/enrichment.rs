use hpo::annotations::{GeneId, OmimDiseaseId};
use pyo3::exceptions::PyNotImplementedError;
use pyo3::types::PyDict;
use pyo3::{exceptions::PyKeyError, prelude::*};

use hpo::stats::hypergeom::{disease_enrichment, gene_enrichment};

use crate::annotations::{PyGene, PyOmimDisease};
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
    /// hposet: :class:`pyhpo.HPOSet`
    ///     The set of HPOTerms to use as sampleset for calculation of
    ///     enrichment. The full ontology is used as background set.
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
                    .map(|enrichment| gene_enrichment_dict(py, enrichment))
                    .collect::<PyResult<Vec<&PyDict>>>()
            }
            EnrichmentType::Omim => {
                let mut enr = disease_enrichment(ont, &set);
                enr.sort_by(|a, b| a.pvalue().partial_cmp(&b.pvalue()).unwrap());
                enr.iter()
                    .map(|enrichment| disease_enrichment_dict(py, enrichment))
                    .collect::<PyResult<Vec<&PyDict>>>()
            }
        };
        res
    }
}

pub(crate) fn disease_enrichment_dict<'a, T>(
    py: Python<'a>,
    enrichment: &hpo::stats::Enrichment<T>,
) -> PyResult<&'a PyDict>
where
    T: std::fmt::Display + hpo::annotations::AnnotationId,
{
    let disease = get_ontology()?
        .omim_disease(&OmimDiseaseId::from(enrichment.id().as_u32()))
        .map(|d| PyOmimDisease::new(*d.id(), d.name().into()))
        .unwrap();
    let dict = PyDict::new(py);
    dict.set_item("enrichment", enrichment.pvalue())?;
    dict.set_item("fold", enrichment.enrichment())?;
    dict.set_item("count", enrichment.count())?;
    dict.set_item("item", disease.into_py(py))?;
    Ok(dict)
}

pub(crate) fn gene_enrichment_dict<'a, T>(
    py: Python<'a>,
    enrichment: &hpo::stats::Enrichment<T>,
) -> PyResult<&'a PyDict>
where
    T: std::fmt::Display + hpo::annotations::AnnotationId,
{
    let gene = get_ontology()?
        .gene(&GeneId::from(enrichment.id().as_u32()))
        .map(|g| PyGene::new(*g.id(), g.name().into()))
        .unwrap();
    let dict = PyDict::new(py);
    dict.set_item("enrichment", enrichment.pvalue())?;
    dict.set_item("fold", enrichment.enrichment())?;
    dict.set_item("count", enrichment.count())?;
    dict.set_item("item", gene.into_py(py))?;
    Ok(dict)
}
