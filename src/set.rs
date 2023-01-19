use std::collections::HashSet;
use std::num::ParseIntError;

use hpo::similarity::{GroupSimilarity, StandardCombiner};
use pyo3::exceptions::PyRuntimeError;
use pyo3::types::PyDict;
use pyo3::{prelude::*, types::PyType};

use hpo::{term::HpoGroup, HpoSet, HpoTermId};

use crate::{
    annotations::{PyGene, PyOmimDisease},
    get_ontology,
    information_content::PyInformationContentKind,
};
use crate::{term_from_query, PyQuery};

#[pyclass(name = "HPOSet")]
pub struct PyHpoSet {
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

#[pymethods]
impl PyHpoSet {
    #[new]
    fn new(terms: Vec<u32>) -> Self {
        let mut ids = HpoGroup::new();
        for id in terms {
            ids.insert(id.into());
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
        new_set.replace_obsolete(ont);
        Ok(new_set.into())
    }

    fn all_genes(&self) -> PyResult<HashSet<PyGene>> {
        let ont = get_ontology()?;
        Ok(HpoSet::new(ont, self.ids.clone()).gene_ids().iter().fold(
            HashSet::new(),
            |mut set, gene_id| {
                set.insert(PyGene::from(ont.gene(gene_id).expect("gene must be present in ontology if it is connected to a term")));
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
                set.insert(PyOmimDisease::from(ont.omim_disease(disease_id).expect("disease must be present in ontology if it is connected to a term")));
                set
            }))
    }

    #[args(kind = "\"omim\"")]
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

    #[args(kind = "\"omim\"", method = "\"graphic\"", combine = "\"funSimAvg\"")]
    #[pyo3(text_signature = "($self, other, kind)")]
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
            .collect::<PyHpoSet>()
        )
    }

    fn __len__(&self) -> usize {
        self.ids.len()
    }
}
