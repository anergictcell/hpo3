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
    dict.set_item("item_name", enrichment.id().to_string())?;
    dict.set_item("item", enrichment.id().as_u32())?;
    Ok(dict)
}
