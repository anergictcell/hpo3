use std::collections::HashSet;
use std::hash::Hash;

use pyo3::class::basic::CompareOp;
use pyo3::exceptions::PyRuntimeError;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use rayon::prelude::*;

use hpo::annotations::AnnotationId;
use hpo::similarity::Similarity;
use hpo::term::HpoTermId;

use crate::pyterm_from_id;
use crate::term_from_id;
use crate::ONTOLOGY;

use crate::PyGene;
use crate::PyInformationContent;
use crate::PyInformationContentKind;
use crate::PyOmimDisease;

#[pyclass(name = "HPOTerm")]
#[derive(Clone)]
pub struct PyHpoTerm {
    id: HpoTermId,
    name: String,
}

impl PyHpoTerm {
    pub fn new(id: HpoTermId, name: String) -> Self {
        Self { id, name }
    }

    /// Returns the `hpo::HpoTerm`
    ///
    /// This method assumes that this operation
    /// will succeed because the term should exist
    fn hpo(&self) -> hpo::HpoTerm {
        let ont = ONTOLOGY
            .get()
            .expect("ontology must exist when a term is present");
        ont.hpo(self.id)
            .expect("the term itself must exist in the ontology")
    }

    pub fn hpo_term_id(&self) -> HpoTermId {
        self.id
    }
}

impl TryFrom<HpoTermId> for PyHpoTerm {
    type Error = PyErr;
    fn try_from(value: HpoTermId) -> PyResult<PyHpoTerm> {
        pyterm_from_id(value.as_u32())
    }
}

impl PartialEq for PyHpoTerm {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for PyHpoTerm {}

impl Hash for PyHpoTerm {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.id.as_u32())
    }
}

#[pymethods]
impl PyHpoTerm {
    /// Returns the HPO Term ID
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     term = Ontology.hpo(11968)
    ///     term.id()    # ==> 'HP:0011968'
    ///
    #[getter(id)]
    fn id(&self) -> String {
        self.id.to_string()
    }

    /// Returns the name of the HPO Term
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     term = Ontology.hpo(11968)
    ///     term.name()  # ==> 'Feeding difficulties'
    ///
    #[getter(name)]
    fn name(&self) -> &str {
        &self.name
    }

    /// Returns the Information Content of the HPO Term
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     term = Ontology.hpo(11968)
    ///     term.information_content.omim  # ==> 2.5363943576812744
    ///     term.information_content.gene  # ==> 1.457185983657837
    ///
    #[getter(information_content)]
    fn information_content(&self) -> PyInformationContent {
        self.hpo().information_content().into()
    }

    /// A list of direct parents
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     term = Ontology.hpo(108)
    ///     term.parents  # ==> {<HpoTerm (HP:0011035)>, <HpoTerm (HP:0000107)>, <HpoTerm (HP:0100957)>}
    ///
    #[getter(parents)]
    fn parents(&self) -> HashSet<PyHpoTerm> {
        self.hpo().parents().fold(HashSet::new(), |mut set, term| {
            set.insert(PyHpoTerm {
                id: term.id(),
                name: term.name().to_string(),
            });
            set
        })
    }

    /// A list of all parents
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     term = Ontology.hpo(108)
    ///     term.all_parents  # ==> {large set}
    ///
    #[getter(all_parents)]
    fn all_parents(&self) -> HashSet<PyHpoTerm> {
        self.hpo()
            .all_parents()
            .fold(HashSet::new(), |mut set, term| {
                set.insert(PyHpoTerm {
                    id: term.id(),
                    name: term.name().to_string(),
                });
                set
            })
    }

    /// A list of direct children
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     term = Ontology.hpo(1)
    ///     term.children  # ==> {<HpoTerm (HP:0000005)>, <HpoTerm (HP:0000118)>, <HpoTerm (HP:0012823)>, <HpoTerm (HP:0032443)>, <HpoTerm (HP:0040279)>, <HpoTerm (HP:0032223)>}
    ///
    #[getter(children)]
    fn children(&self) -> HashSet<PyHpoTerm> {
        self.hpo().children().fold(HashSet::new(), |mut set, term| {
            set.insert(PyHpoTerm {
                id: term.id(),
                name: term.name().to_string(),
            });
            set
        })
    }

    /// A list of associated genes
    ///
    /// The list includes "inherited" genes that are not directly
    /// linked to the term, but to one of its children
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     term = Ontology.hpo(188)
    ///     for gene in term.genes:
    ///         print(gene.name)
    ///
    #[getter(genes)]
    fn genes(&self) -> HashSet<PyGene> {
        self.hpo().genes().fold(HashSet::new(), |mut set, gene| {
            set.insert(PyGene::from(gene));
            set
        })
    }

    /// A list of associated OMIM diseases
    ///
    /// The list includes "inherited" diseases that are not directly
    /// linked to the term, but to one of its children
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     term = Ontology.hpo(188)
    ///     for disease in term.diseases:
    ///         print(disease.name)
    ///
    #[getter(omim_diseases)]
    fn diseases(&self) -> HashSet<PyOmimDisease> {
        self.hpo()
            .omim_diseases()
            .fold(HashSet::new(), |mut set, disease| {
                set.insert(PyOmimDisease::from(disease));
                set
            })
    }

    /// A list of the phenotypical categories the term belongs to
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     term = Ontology.hpo(10049)
    ///     for cat in term.categories:
    ///         print(cat.name)
    ///
    #[getter(categories)]
    fn categories(&self) -> PyResult<Vec<PyHpoTerm>> {
        self.hpo()
            .categories()
            .iter()
            .map(|id| pyterm_from_id(id.as_u32()))
            .collect()
    }

    #[pyo3(text_signature = "($self, other)")]
    fn parent_of(&self, other: &PyHpoTerm) -> bool {
        self.hpo().parent_of(&other.hpo())
    }

    #[pyo3(text_signature = "($self, other)")]
    fn child_of(&self, other: &PyHpoTerm) -> bool {
        self.hpo().child_of(&other.hpo())
    }

    #[pyo3(text_signature = "($self)")]
    fn parent_ids(&self) -> Vec<u32> {
        self.hpo().parent_ids().iter().map(|t| t.as_u32()).collect()
    }

    #[pyo3(text_signature = "($self, other)")]
    fn common_ancestors(&self, other: &PyHpoTerm) -> HashSet<PyHpoTerm> {
        self.hpo()
            .common_ancestors(&other.hpo())
            .iter()
            .fold(HashSet::new(), |mut set, term| {
                set.insert(PyHpoTerm::from(term));
                set
            })
    }

    #[pyo3(text_signature = "($self)")]
    fn count_parents(&self) -> usize {
        self.hpo().parent_ids().len()
    }

    #[pyo3(text_signature = "($self)")]
    fn shortest_path_to_root(&self) -> usize {
        let root = term_from_id(1).expect("the root must exist");
        self.hpo()
            .distance_to_ancestor(&root)
            .expect("the root term must be an ancestor")
    }

    #[pyo3(text_signature = "($self, other)")]
    /// Calculates the shortest path to an ancestor HPO Term
    ///
    /// If `other` is not a parent term, the distance will be `Inf`.
    ///
    /// As a minor difference to `PyHPO`, this method does not return
    /// a Tuple, but a list of terms.
    fn shortest_path_to_parent(&self, other: &PyHpoTerm) -> (f32, Vec<PyHpoTerm>) {
        let path = if let Some(path) = self.hpo().path_to_ancestor(&other.into()) {
            path
        } else {
            return (f32::INFINITY, vec![]);
        };
        (
            path.len() as f32,
            path.iter()
                .map(|id| {
                    pyterm_from_id(id.as_u32())
                        .expect("the term must exist because its an ancestor term")
                })
                .collect(),
        )
    }

    /// Calculates the shortest path to another HPO Term
    ///
    /// IMPORTANT NOTE
    /// --------------
    /// This method is not correctly implemented and will not return
    /// the sub-paths distances
    #[pyo3(text_signature = "($self, other)")]
    pub fn path_to_other(
        &self,
        other: &PyHpoTerm,
    ) -> PyResult<(usize, Vec<PyHpoTerm>, usize, usize)> {
        let path = self
            .hpo()
            .path_to_term(&other.into())
            .ok_or_else(|| PyRuntimeError::new_err("No path found"))?;
        Ok((
            path.len(),
            path.iter()
                .map(|id| pyterm_from_id(id.as_u32()).expect("term must be part of Ontology"))
                .collect(),
            0,
            0,
        ))
    }

    /// Calculates the similarity score of two HPO Terms
    ///
    /// Arguments
    /// ---------
    /// other: `int`
    ///     ID of the other term as `int` (`HP:0000123` --> `123`)
    /// kind: `str`, default: `omim`
    ///     Which kind of information content to use for similarity calculation
    ///     
    ///     Available options:
    ///
    ///     * **omim**
    ///     * **gene**
    ///
    /// method: `str`, default `graphic`
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
    /// `float`
    ///     The similarity score
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///
    ///     Ontology()
    ///     term = Ontology[11968]
    ///
    ///     term.similarity_score(Ontology[1743])
    ///
    ///     # compare HP:0011968 and HP:0001743 using Gene
    ///     term.similarity_score(Ontology[1743], kind="gene")
    ///
    #[pyo3(signature = (other, kind = "omim", method = "graphic"))]
    #[pyo3(text_signature = "($self, other, kind, method)")]
    fn similarity_score(&self, other: &PyHpoTerm, kind: &str, method: &str) -> PyResult<f32> {
        let kind = PyInformationContentKind::try_from(kind)?;

        let term_a = self.hpo();
        let term_b = other.hpo();

        let similarity = hpo::similarity::Builtins::new(method, kind.into())
            .map_err(|_| PyRuntimeError::new_err("Unknown method to calculate similarity"))?;
        Ok(similarity.calculate(&term_a, &term_b))
    }

    /// Calculates the similarity score between the term and a batch of other terms
    ///
    /// This method is useful if you want to compare the term to **thousands** of other terms.
    /// It will utilize all avaible CPU for parallel processing.
    ///
    /// Arguments
    /// ---------
    /// others: `List[int]`
    ///     ID of the other term as `int` (`HP:0000123` --> `123`)
    /// kind: `str`, default: `omim`
    ///     Which kind of information content to use for similarity calculation
    ///
    ///     Available options:
    ///
    ///     * **omim**
    ///     * **gene**
    ///
    /// method: `str`, default `graphic`
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
    /// `List[float]`
    ///     The similarity scores
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///
    ///     Ontology()
    ///     term = Ontology[11968]
    ///
    ///     term.similarity_scores(list(Ontology))
    ///
    ///
    #[pyo3(signature = (others, kind = "omim", method = "graphic"))]
    #[pyo3(text_signature = "($self, others, kind, method)")]
    fn similarity_scores(
        &self,
        others: Vec<PyHpoTerm>,
        kind: &str,
        method: &str,
    ) -> PyResult<Vec<f32>> {
        let kind = PyInformationContentKind::try_from(kind)?;

        let term_a = self.hpo();

        let similarity = hpo::similarity::Builtins::new(method, kind.into())
            .map_err(|_| PyRuntimeError::new_err("Unknown method to calculate similarity"))?;

        Ok(others
            .par_iter()
            .map(|term_b| {
                let t2: hpo::HpoTerm = term_b.into();
                similarity.calculate(&term_a, &t2)
            })
            .collect())
    }

    #[pyo3(signature = (verbose = false))]
    #[pyo3(text_signature = "($self, verbose)")]
    #[allow(non_snake_case)]
    pub fn toJSON<'a>(&'a self, py: Python<'a>, verbose: bool) -> PyResult<&PyDict> {
        let term = self.hpo();
        let dict = PyDict::new(py);
        dict.set_item("name", term.name())?;
        dict.set_item("id", term.id().to_string())?;
        dict.set_item("int", term.id().as_u32())?;

        if verbose {
            let ic = PyDict::new(py);
            ic.set_item("gene", term.information_content().gene())?;
            ic.set_item("omim", term.information_content().omim_disease())?;
            ic.set_item("orpha", 0.0)?;
            ic.set_item("decipher", 0.0)?;
            dict.set_item::<&str, Vec<&str>>("synonym", vec![])?;
            dict.set_item("comment", "")?;
            dict.set_item("definition", "")?;
            dict.set_item::<&str, Vec<&str>>("xref", vec![])?;
            dict.set_item::<&str, Vec<&str>>("is_a", vec![])?;
            dict.set_item("ic", ic)?;
        }
        Ok(dict)
    }

    fn __str__(&self) -> String {
        format!("{} | {}", self.id(), self.name())
    }

    fn __repr__(&self) -> String {
        format!("<HpoTerm ({})>", self.id())
    }

    fn __int__(&self) -> u32 {
        self.id.as_u32()
    }

    fn __hash__(&self) -> u32 {
        self.__int__()
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => Err(PyTypeError::new_err(
                "\"<\" is not supported for HPOTerm instances",
            )),
            CompareOp::Le => Err(PyTypeError::new_err(
                "\"<=\" is not supported for HPOTerm instances",
            )),
            CompareOp::Eq => Ok(self == other),
            CompareOp::Ne => Ok(self != other),
            CompareOp::Gt => Err(PyTypeError::new_err(
                "\">\" is not supported for HPOTerm instances",
            )),
            CompareOp::Ge => Err(PyTypeError::new_err(
                "\">=\" is not supported for HPOTerm instances",
            )),
        }
    }
}

impl From<&PyHpoTerm> for hpo::HpoTerm<'static> {
    fn from(value: &PyHpoTerm) -> hpo::HpoTerm<'static> {
        term_from_id(value.id.as_u32())
            .expect("term must exist in ontology since it comes from an HPOTerm")
    }
}

impl From<hpo::HpoTerm<'_>> for PyHpoTerm {
    fn from(term: hpo::HpoTerm<'_>) -> Self {
        PyHpoTerm {
            id: term.id(),
            name: term.name().to_string(),
        }
    }
}
