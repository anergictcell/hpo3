use std::collections::HashSet;
use std::num::ParseIntError;

use rayon::prelude::*;

use pyo3::exceptions::PyRuntimeError;
use pyo3::types::PyDict;
use pyo3::{prelude::*, types::PyType};

use hpo::annotations::AnnotationId;
use hpo::similarity::{GroupSimilarity, StandardCombiner};
use hpo::Ontology;
use hpo::{term::HpoGroup, HpoSet, HpoTermId};

use crate::term::PyHpoTerm;
use crate::{
    annotations::{PyGene, PyOmimDisease},
    get_ontology,
    information_content::PyInformationContentKind,
};
use crate::{pyterm_from_id, term_from_query, PyQuery};

#[pyclass(name = "HPOSet")]
#[derive(Clone)]
pub(crate) struct PyHpoSet {
    ids: HpoGroup,
}

impl FromIterator<HpoTermId> for PyHpoSet {
    fn from_iter<T: IntoIterator<Item = HpoTermId>>(iter: T) -> Self {
        let ids: HpoGroup = iter.into_iter().collect();
        Self { ids }
    }
}

impl From<HpoSet<'_>> for PyHpoSet {
    fn from(set: HpoSet) -> Self {
        set.into_iter().map(|term| term.id()).collect()
    }
}

/// A set of HPO terms
///
/// Examples
/// --------
///
/// .. code-block: python
///
///     from pyhpo import Ontology, HPOSet
///     ont = Ontology()
///     s = HPOSet([1, 118])
///     len(s)  
///     # >> 2
///
#[pymethods]
impl PyHpoSet {
    #[new]
    fn new(terms: Vec<u32>) -> Self {
        let mut ids = HpoGroup::new();
        for id in terms {
            ids.insert(id);
        }
        Self { ids }
    }

    fn child_nodes(&self) -> PyResult<Self> {
        let ont = get_ontology()?;
        Ok(HpoSet::new(ont, self.ids.clone()).child_nodes().into())
    }

    fn remove_modifier(&self) -> PyResult<Self> {
        let ont = get_ontology()?;
        let mut new_set = HpoSet::new(ont, self.ids.clone());
        new_set.remove_modifier();
        Ok(new_set.into())
    }

    fn replace_obsolete(&self) -> PyResult<Self> {
        let ont = get_ontology()?;
        let mut new_set = HpoSet::new(ont, self.ids.clone());
        new_set.replace_obsolete();
        new_set.remove_obsolete();
        Ok(new_set.into())
    }

    fn all_genes(&self) -> PyResult<HashSet<PyGene>> {
        let ont = get_ontology()?;
        Ok(HpoSet::new(ont, self.ids.clone()).gene_ids().iter().fold(
            HashSet::new(),
            |mut set, gene_id| {
                set.insert(PyGene::from(ont.gene(gene_id).expect(
                    "gene must be present in ontology if it is connected to a term",
                )));
                set
            },
        ))
    }

    fn omim_diseases(&self) -> PyResult<HashSet<PyOmimDisease>> {
        let ont = get_ontology()?;
        Ok(HpoSet::new(ont, self.ids.clone())
            .omim_disease_ids()
            .iter()
            .fold(HashSet::new(), |mut set, disease_id| {
                set.insert(PyOmimDisease::from(ont.omim_disease(disease_id).expect(
                    "disease must be present in ontology if it is connected to a term",
                )));
                set
            }))
    }

    #[pyo3(signature = (kind = "omim"))]
    fn information_content<'a>(&'a self, py: Python<'a>, kind: &str) -> PyResult<&PyDict> {
        let kind = PyInformationContentKind::try_from(kind)?;
        let ont = get_ontology()?;
        let ics: Vec<f32> = self
            .ids
            .into_iter()
            .map(|term_id| {
                ont.hpo(term_id)
                    .expect("term must be present in the ontology if it is included in the set")
                    .information_content()
                    .get_kind(&kind.into())
            })
            .collect();

        let total: f32 = ics.iter().sum();

        let dict = PyDict::new(py);
        dict.set_item("mean", total / ics.len() as f32)?;
        dict.set_item("total", total)?;
        dict.set_item(
            "max",
            ics.iter()
                .reduce(|max, cur| if cur > max { cur } else { max }),
        )?;
        dict.set_item("all", ics)?;

        Ok(dict)
    }

    fn variance(&self) -> Self {
        unimplemented!()
    }

    fn combinations(&self) -> Self {
        unimplemented!()
    }

    fn combinations_one_way(&self) -> Self {
        unimplemented!()
    }

    /// Calculate similarity between this and another `HPOSet`
    ///
    /// This method runs parallelized on all avaible CPU
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     ont = Ontology()
    ///     gene_sets = [g.hpo_set() for g in Ontology.genes]
    ///     gene_sets[0].similarity(gene_sets[1])
    ///     # >> 0.29546087980270386
    ///
    #[pyo3(signature = (other, kind = "omim", method = "graphic", combine = "funSimAvg"))]
    #[pyo3(text_signature = "($self, other, kind, method, combine)")]
    fn similarity(
        &self,
        other: &PyHpoSet,
        kind: &str,
        method: &str,
        combine: &str,
    ) -> PyResult<f32> {
        let ont = get_ontology()?;
        let set_a = HpoSet::new(ont, self.ids.clone());
        let set_b = HpoSet::new(ont, other.ids.clone());

        let kind = PyInformationContentKind::try_from(kind)?;

        let similarity = hpo::similarity::Builtins::new(method, kind.into())
            .map_err(|_| PyRuntimeError::new_err("Unknown method to calculate similarity"))?;
        let combiner = StandardCombiner::try_from(combine)
            .map_err(|_| PyRuntimeError::new_err("Invalid combine method specified"))?;

        let g_sim = GroupSimilarity::new(combiner, similarity);

        Ok(g_sim.calculate(&set_a, &set_b))
    }

    /// Calculate similarity between this `HPOSet` and a list of other `HPOSet`
    ///
    /// This method runs parallelized on all avaible CPU
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     ont = Ontology()
    ///     gene_sets = [g.hpo_set() for g in Ontology.genes]
    ///     similarities = gene_sets[0].batch_similarity(gene_sets)
    ///     similarities[0:4]
    ///     # >> [1.0, 0.5000048279762268, 0.29546087980270386, 0.5000059008598328]
    ///
    #[pyo3(signature =(other, kind = "omim", method = "graphic", combine = "funSimAvg"))]
    #[pyo3(text_signature = "($self, other, kind, method, combine)")]
    fn batch_similarity(
        &self,
        other: Vec<PyHpoSet>,
        kind: &str,
        method: &str,
        combine: &str,
    ) -> PyResult<Vec<f32>> {
        let ont = get_ontology()?;
        let set_a = HpoSet::new(ont, self.ids.clone());

        let kind = PyInformationContentKind::try_from(kind)?;
        let similarity = hpo::similarity::Builtins::new(method, kind.into())
            .map_err(|_| PyRuntimeError::new_err("Unknown method to calculate similarity"))?;
        let combiner = StandardCombiner::try_from(combine)
            .map_err(|_| PyRuntimeError::new_err("Invalid combine method specified"))?;

        let g_sim = GroupSimilarity::new(combiner, similarity);

        Ok(other
            .par_iter()
            .map(|sb| {
                let set_b = HpoSet::new(ont, sb.ids.clone());
                g_sim.calculate(&set_a, &set_b)
            })
            .collect())
    }

    #[allow(non_snake_case)]
    fn toJSON(&self) -> PyResult<Vec<&PyDict>> {
        unimplemented!()
    }

    fn serialize(&self) -> String {
        let mut ids = self
            .ids
            .iter()
            .map(|tid| tid.as_u32())
            .collect::<Vec<u32>>();
        ids.sort();

        let id_strings: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        id_strings.join("+")
    }

    /// Returns the HPOTerms in the set
    ///
    /// TODO: Convert this to an iterator
    fn terms(&self) -> PyResult<Vec<PyHpoTerm>> {
        let terms: Vec<PyHpoTerm> = self
            .ids
            .iter()
            .filter_map(|id| pyterm_from_id(id.as_u32()).ok())
            .collect();
        if terms.len() == self.ids.len() {
            Ok(terms)
        } else {
            Err(PyRuntimeError::new_err("Some terms are not correct"))
        }
    }

    #[classmethod]
    fn from_queries(_cls: &PyType, queries: Vec<PyQuery>) -> PyResult<Self> {
        let mut ids: Vec<HpoTermId> = Vec::with_capacity(queries.len());
        for q in queries {
            ids.push(term_from_query(q)?.id());
        }
        Ok(ids.into_iter().collect::<PyHpoSet>())
    }

    #[classmethod]
    fn from_serialized(_cls: &PyType, pickle: &str) -> PyResult<Self> {
        Ok(pickle
            .split('+')
            .map(|id| id.parse::<u32>())
            .collect::<Result<Vec<u32>, ParseIntError>>()?
            .iter()
            .map(|id| HpoTermId::from(*id))
            .collect::<PyHpoSet>())
    }

    #[classmethod]
    pub fn from_gene(_cls: &PyType, gene: &PyGene) -> PyResult<Self> {
        gene.try_into()
    }

    #[classmethod]
    pub fn from_disease(_cls: &PyType, disease: &PyOmimDisease) -> PyResult<Self> {
        disease.try_into()
    }

    fn __len__(&self) -> usize {
        self.ids.len()
    }

    fn __repr__(&self) -> String {
        format!(
            "HPOSet.from_serialized({})",
            self.ids
                .iter()
                .map(|i| i.as_u32().to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }

    fn __str__(&self) -> String {
        format!(
            "HPOSet: [{}]",
            if self.ids.len() <= 10 {
                self.ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            } else if self.ids.is_empty() {
                "-".to_string()
            } else {
                format!("{} terms", self.ids.len())
            }
        )
    }
}

impl<'a> PyHpoSet {
    pub fn set(&'a self, ont: &'a Ontology) -> HpoSet {
        HpoSet::new(ont, self.ids.clone())
    }
}

impl TryFrom<&PyGene> for PyHpoSet {
    type Error = PyErr;
    fn try_from(gene: &PyGene) -> Result<Self, Self::Error> {
        let ont = get_ontology()?;
        Ok(ont
            .gene(&gene.id().into())
            .expect("ontology must. be present and gene must be included")
            .to_hpo_set(ont)
            .into())
    }
}

impl TryFrom<&PyOmimDisease> for PyHpoSet {
    type Error = PyErr;
    fn try_from(disease: &PyOmimDisease) -> Result<Self, Self::Error> {
        let ont = get_ontology()?;
        Ok(ont
            .omim_disease(&disease.id().into())
            .expect("ontology must. be present and gene must be included")
            .to_hpo_set(ont)
            .into())
    }
}

#[pyclass(name = "BasicHPOSet")]
#[derive(Clone, Default, Debug)]
pub(crate) struct BasicPyHpoSet;

impl BasicPyHpoSet {
    fn build<I: IntoIterator<Item = HpoTermId>>(ids: I) -> PyHpoSet {
        let ont = get_ontology().expect("Ontology must be initialized");
        let mut group = HpoGroup::new();
        for id in ids {
            group.insert(id);
        }
        let mut set = HpoSet::new(ont, group);
        let mut set = set.child_nodes();
        set.replace_obsolete();
        set.remove_obsolete();
        set.remove_modifier();
        PyHpoSet::new(set.iter().map(|term| term.id().as_u32()).collect())
    }
}

#[pymethods]
impl BasicPyHpoSet {
    fn __call__(&self, terms: Vec<u32>) -> PyHpoSet {
        BasicPyHpoSet::build(terms.iter().map(|id| HpoTermId::from_u32(*id)))
    }

    #[classmethod]
    fn from_queries(_cls: &PyType, queries: Vec<PyQuery>) -> PyResult<PyHpoSet> {
        let mut ids: Vec<HpoTermId> = Vec::with_capacity(queries.len());
        for q in queries {
            ids.push(term_from_query(q)?.id());
        }
        Ok(BasicPyHpoSet::build(ids))
    }

    #[classmethod]
    fn from_serialized(_cls: &PyType, pickle: &str) -> PyResult<PyHpoSet> {
        Ok(BasicPyHpoSet::build(
            pickle
                .split('+')
                .map(|id| id.parse::<u32>())
                .collect::<Result<Vec<u32>, ParseIntError>>()?
                .iter()
                .map(|id| HpoTermId::from_u32(*id)),
        ))
    }
}
