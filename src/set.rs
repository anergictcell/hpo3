use std::collections::{HashSet, VecDeque};
use std::num::ParseIntError;

use rayon::prelude::*;

use pyo3::exceptions::{PyAttributeError, PyRuntimeError};
use pyo3::types::PyDict;
use pyo3::{prelude::*, types::PyType};

use hpo::annotations::{AnnotationId, Disease};
use hpo::similarity::{GroupSimilarity, StandardCombiner};
use hpo::Ontology;
use hpo::{term::HpoGroup, HpoSet, HpoTermId};

use crate::annotations::PyOrphaDisease;
use crate::term::PyHpoTerm;
use crate::{
    annotations::{PyGene, PyOmimDisease},
    get_ontology,
    information_content::PyInformationContentKind,
};
use crate::{pyterm_from_id, term_from_id, term_from_query, PyQuery, TermOrId};

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
///     Ontology()
///     s = HPOSet([1, 118])
///     len(s)  
///     # >> 2
///
#[pymethods]
impl PyHpoSet {
    /// Instantiates a new ``HPOSet``
    ///
    /// Parameters
    /// ----------
    /// terms: List[int | :class:`pyhpo.HPOTerm`]
    ///     The terms that make up the set
    ///
    /// Returns
    /// -------
    /// :class:`pyhpo.HPOSet`
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
    /// KeyError
    ///     (only when ``int`` are used as input): HPOTerm does not exist
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block: python
    ///
    ///     from pyhpo import Ontology, HPOSet
    ///     Ontology()
    ///     s = HPOSet([1, 118])
    ///     len(s)  
    ///     # >> 2
    ///
    #[new]
    fn new(terms: Vec<TermOrId>) -> PyResult<Self> {
        let mut ids = HpoGroup::new();
        for id in terms {
            match id {
                TermOrId::Id(x) => {
                    _ = term_from_id(x)?;
                    ids.insert(x)
                }
                TermOrId::Term(x) => ids.insert(x.hpo_term_id().as_u32()),
            };
        }
        Ok(Self { ids })
    }

    /// Add an HPOTerm to the HPOSet
    ///
    /// Parameters
    /// ----------
    /// term: :class:`HPOTerm` or int
    ///     The term to add, either as actual ``HPOTerm``
    ///     or the integer representation
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
    /// KeyError
    ///     (only when ``int`` are used as input): HPOTerm does not exist
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology, HPOSet
    ///     Ontology()
    ///     my_set = HPOSet([])
    ///     my_set.add(Ontology.hpo(118))
    ///     len(my_set) # >> 1
    ///     my_set.add(2650)
    ///     len(my_set) # >> 2
    ///
    fn add(&mut self, term: TermOrId) -> PyResult<()> {
        match term {
            TermOrId::Id(x) => {
                _ = term_from_id(x)?;
                self.ids.insert(x)
            }
            TermOrId::Term(x) => self.ids.insert(x.hpo_term_id().as_u32()),
        };
        Ok(())
    }

    /// Returns a new HPOSet that does not contain ancestor terms
    ///
    /// If a set contains HPOTerms that are ancestors of other
    /// terms in the set, they will be removed. This method is useful
    /// to create a set that contains only the most specific terms.
    ///
    /// Returns
    /// -------
    /// :class:`pyhpo.HPOSet`
    ///     A new ``HPOSet`` that contains only the most specific terms
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
    ///     from pyhpo import Ontology, HPOSet
    ///
    ///     my_set = HPOSet.from_queries([
    ///         'HP:0002650',
    ///         'HP:0010674',
    ///         'HP:0000925',
    ///         'HP:0009121'
    ///     ])
    ///     
    ///     child_set = my_set.child_nodes()
    ///     
    ///     len(my_set) # >> 4
    ///     len(child_set) # >> 1
    ///
    fn child_nodes(&self) -> PyResult<Self> {
        let ont = get_ontology()?;
        Ok(HpoSet::new(ont, self.ids.clone()).child_nodes().into())
    }

    /// Returns a new HPOSet that does not contain any modifier terms
    ///
    /// This method removes all terms that are not children of
    /// ``HP:0000118 | Phenotypic abnormality``
    ///
    /// Returns
    /// -------
    /// :class:`pyhpo.HPOSet`
    ///     A new ``HPOSet`` that contains only phenotype terms
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
    ///     from pyhpo import Ontology, HPOSet
    ///
    ///     my_set = HPOSet.from_queries([
    ///         'HP:0002650',
    ///         'HP:0010674',
    ///         'HP:0000925',
    ///         'HP:0009121',
    ///         'HP:0012823',
    ///     ])
    ///     
    ///     pheno_set = my_set.remove_modifier()
    ///     
    ///     len(my_set) # >> 5
    ///     len(pheno_set) # >> 4
    ///
    fn remove_modifier(&self) -> PyResult<Self> {
        let ont = get_ontology()?;
        let mut new_set = HpoSet::new(ont, self.ids.clone());
        new_set.remove_modifier();
        Ok(new_set.into())
    }

    /// Returns a new HPOSet that replaces all obsolete terms with
    /// their replacement
    ///
    /// If an obsolete term has a replacement term defined
    /// it will be replaced, otherwise it will be removed.
    ///
    /// Returns
    /// -------
    /// :class:`pyhpo.HPOSet`
    ///     A new ``HPOSet`` that contains only phenotype terms
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
    ///     from pyhpo import Ontology, HPOSet
    ///
    ///     my_set = HPOSet.from_queries([
    ///         'HP:0002650',
    ///         'HP:0010674',
    ///         'HP:0000925',
    ///         'HP:0009121',
    ///         'HP:0410003',
    ///     ])
    ///     
    ///     active_set = my_set.replace_obsolete()
    ///     
    ///     len(my_set) # >> 5
    ///     len(active_set) # >> 5
    ///
    ///     Ontology.hpo(410003) in my_set
    ///     # >> True
    ///     
    ///     Ontology.hpo(410003) in active_set
    ///     # >> False
    ///
    fn replace_obsolete(&self) -> PyResult<Self> {
        let ont = get_ontology()?;
        let mut new_set = HpoSet::new(ont, self.ids.clone());
        new_set.replace_obsolete();
        new_set.remove_obsolete();
        Ok(new_set.into())
    }

    /// Returns a set of associated genes
    ///
    /// Returns
    /// -------
    /// set[:class:`pyhpo.Gene`]
    ///     The union of genes associated with terms
    ///     in the ``HPOSet``
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
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     disease = list(Ontology.omim_diseases)[0]
    ///     for gene in disease.all_genes():
    ///         print(gene.name)
    ///
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

    /// Returns a set of associated diseases
    ///
    /// Returns
    /// -------
    /// set[:class:`pyhpo.Omim`]
    ///     The union of Omim diseases associated with terms
    ///     in the ``HPOSet``
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
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     gene_set = list(Ontology.genes)[0].hpo_set()
    ///     for disease in gene_set.omim_diseases():
    ///         print(disease.name)
    ///
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

    /// Returns a set of associated diseases
    ///
    /// Returns
    /// -------
    /// set[:class:`pyhpo.Orpha`]
    ///     The union of Orpha diseases associated with terms
    ///     in the ``HPOSet``
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
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     gene_set = list(Ontology.genes)[0].hpo_set()
    ///     for disease in gene_set.orpha_diseases():
    ///         print(disease.name)
    ///
    fn orpha_diseases(&self) -> PyResult<HashSet<PyOrphaDisease>> {
        let ont = get_ontology()?;
        Ok(HpoSet::new(ont, self.ids.clone())
            .orpha_disease_ids()
            .iter()
            .fold(HashSet::new(), |mut set, disease_id| {
                set.insert(PyOrphaDisease::from(ont.orpha_disease(disease_id).expect(
                    "disease must be present in ontology if it is connected to a term",
                )));
                set
            }))
    }

    /// Returns basic information content stats about the
    /// HPOTerms within the set
    ///
    /// Parameters
    /// ----------
    /// kind: str, default: ``omim``
    ///     Which kind of information content should be calculated.
    ///     Options are ['omim', 'orpha', 'gene']
    ///
    /// Returns
    /// -------
    /// dict
    ///     Dict with the following items
    ///
    ///     * **mean** - float - Mean information content
    ///     * **max** - float - Maximum information content value
    ///     * **total** - float - Sum of all information content values
    ///     * **all** - list of float -
    ///       List with all information content values
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
    ///
    /// Examples
    /// --------
    ///
    /// .. code block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     
    ///     my_set = list(Ontology.genes)[0].hpo_set()
    ///     my_set.information_content()
    ///     # >> {
    ///     # >>     'mean': 3.0313432216644287,
    ///     # >>     'total': 357.698486328125,
    ///     # >>     'max': 8.308938026428223,
    ///     # >>     'all': [
    ///     # >>         0.7008119821548462,
    ///     # >>         0.00024631671840325
    ///     # >>         ...
    ///     # >>     ]
    ///     # >> }
    ///
    #[pyo3(signature = (kind = "omim"))]
    fn information_content<'a>(
        &'a self,
        py: Python<'a>,
        kind: &str,
    ) -> PyResult<Bound<'_, PyDict>> {
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

        let dict = PyDict::new_bound(py);
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

    /// Calculates the distances between all its term-pairs. It also provides
    /// basic calculations for variances among the pairs.
    ///
    /// Returns
    /// -------
    ///
    /// tuple of (int, int, int, list of int)
    ///     Tuple with the variance metrices
    ///
    ///     * **float** Average distance between pairs
    ///     * **int** Smallest distance between pairs
    ///     * **int** Largest distance between pairs
    ///     * **list of int** List of all distances between pairs
    fn variance(&self) -> Self {
        unimplemented!()
    }

    /// Helper generator function that returns all possible two-pair
    /// combination between all its terms
    ///
    /// This function is direction dependent. That means that every
    /// pair will appear twice. Once for each direction
    ///
    /// .. seealso:: :func:`pyhpo.HPOSet.combinations_one_way`
    ///
    /// Yields
    /// ------
    /// Tuple of :class:`pyhpo.HPOTerm`
    ///
    ///     Tuple containing the follow items
    ///     * **HPOTerm** 1 of the pair
    ///     * **HPOTerm** 2 of the pair
    ///
    fn combinations(&self) -> Self {
        unimplemented!()
    }

    /// Helper generator function that returns all possible two-pair
    /// combination between all its terms
    ///
    /// This methow will report each pair only once
    ///
    /// .. seealso:: :func:`pyhpo.HPOSet.combinations`
    ///
    /// Yields
    /// ------
    /// Tuple of :class:`term.HPOTerm`
    ///     Tuple containing the follow items
    ///
    ///     * **HPOTerm** instance 1 of the pair
    ///     * **HPOTerm** instance 2 of the pair
    fn combinations_one_way(&self) -> Self {
        unimplemented!()
    }

    /// Calculate similarity between this and another `HPOSet`
    ///
    /// This method runs parallelized on all avaible CPU
    ///
    /// Parameters
    /// ----------
    /// other: :class:`pyhpo.HPOSet`
    ///     The ``HPOSet`` to calculate the similarity to
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
    /// combine: str, default ``funSimAvg``
    ///     The method to combine individual term similarity
    ///     to HPOSet similarities.
    ///
    ///     Available options:
    ///
    ///     * **funSimAvg**
    ///     * **funSimMax**
    ///     * **BMA**
    ///
    /// Returns
    /// -------
    /// float
    ///     Similarity scores
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
    /// AttributeError
    ///     Invalid ``kind``
    /// RuntimeError
    ///     Invalid ``method`` or ``combine``
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
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

        let kind = PyInformationContentKind::try_from(kind)
            .map_err(|_| PyAttributeError::new_err("Invalid Information content"))?;

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
    /// Parameters
    /// ----------
    /// other: list[:class:`pyhpo.HPOSet`]
    ///     Calculate similarity between ``self`` and every provided ``HPOSet``
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
    /// combine: str, default ``funSimAvg``
    ///     The method to combine individual term similarity
    ///     to HPOSet similarities.
    ///
    ///     Available options:
    ///
    ///     * **funSimAvg**
    ///     * **funSimMax**
    ///     * **BMA**
    ///
    /// Returns
    /// -------
    /// list[float]
    ///     Similarity scores for every comparison
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
    /// KeyError
    ///     Invalid ``kind``
    /// RuntimeError
    ///     Invalid ``method`` or ``combine``
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     gene_sets = [g.hpo_set() for g in Ontology.genes]
    ///     similarities = gene_sets[0].similarity_scores(gene_sets)
    ///     similarities[0:4]
    ///     # >> [1.0, 0.5000048279762268, 0.29546087980270386, 0.5000059008598328]
    ///
    #[pyo3(signature =(other, kind = "omim", method = "graphic", combine = "funSimAvg"))]
    #[pyo3(text_signature = "($self, other, kind, method, combine)")]
    fn similarity_scores(
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

    /// Returns a dict/JSON representation the HPOSet
    ///
    /// Parameters
    /// ----------
    /// verbose: bool
    ///     Indicates if each HPOTerm should contain verbose information
    ///     see :func:`pyhpo.HpoTerm.toJSON`
    ///
    /// Returns
    /// -------
    /// Dict
    ///     Dict representation of all HPOTerms in the set
    ///     that can be used for JSON serialization
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
    /// KeyError
    ///     No HPO term is found for the provided query
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology, HPOSet
    ///     Ontology()
    ///     my_set = HPOSet.from_serialized("7+118+152+234+271+315+478+479+492+496")
    ///     my_set.toJSON()
    ///     # >> [
    ///     # >>     {'name': 'Autosomal recessive inheritance', 'id': 'HP:0000007', 'int': 7},
    ///     # >>     {'name': 'Phenotypic abnormality', 'id': 'HP:0000118', 'int': 118},
    ///     # >>     {'name': 'Abnormality of head or neck', 'id': 'HP:0000152', 'int': 152},
    ///     # >>     {'name': 'Abnormality of the head', 'id': 'HP:0000234', 'int': 234},
    ///     # >>     {'name': 'Abnormality of the face', 'id': 'HP:0000271', 'int': 271},
    ///     # >>     {'name': 'Abnormality of the orbital region', 'id': 'HP:0000315', 'int': 315},
    ///     # >>     {'name': 'Abnormality of the eye', 'id': 'HP:0000478', 'int': 478},
    ///     # >>     {'name': 'Abnormal retinal morphology', 'id': 'HP:0000479', 'int': 479},
    ///     # >>     {'name': 'Abnormal eyelid morphology', 'id': 'HP:0000492', 'int': 492},
    ///     # >>     {'name': 'Abnormality of eye movement', 'id': 'HP:0000496', 'int': 496}
    ///     # >> ]
    ///
    #[pyo3(signature = (verbose = false))]
    #[pyo3(text_signature = "($self, verbose)")]
    #[allow(non_snake_case)]
    fn toJSON<'a>(&'a self, py: Python<'a>, verbose: bool) -> PyResult<Vec<Bound<'_, PyDict>>> {
        self.ids
            .iter()
            .map(|id| {
                let dict = PyDict::new_bound(py);
                let term = term_from_id(id.as_u32())?;
                dict.set_item("name", term.name())?;
                dict.set_item("id", term.id().to_string())?;
                dict.set_item("int", term.id().as_u32())?;

                if verbose {
                    let ic = PyDict::new_bound(py);
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
            })
            .collect()
    }

    /// Returns a serialized string representing the HPOSet
    ///
    /// Returns
    /// -------
    /// str
    ///     A serialized string uniquely representing the HPOSet,
    ///     e.g.: ``3+118+2650```
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     gene_sets = [g.hpo_set() for g in Ontology.genes]
    ///     gene_sets[0].serialize()
    ///     # >> 7+118+152+234+271+315+478+479+492+496.....
    ///
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
    /// Returns
    /// -------
    /// list[:class:`pyhpo.HPOTerm`]
    ///     A list of every term in the set
    ///
    ///
    /// .. important::
    ///
    ///    The return type of this method will very likely change
    ///    into an Iterator of ``HPOTerm``. (:doc:`api_changes`)
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
    /// KeyError
    ///     No HPO term is found for the provided query
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     my_set = list(Ontology.genes)[0].hpo_set()
    ///     for term in my_set.terms():
    ///         print(term.name)
    ///
    fn terms(&self) -> PyResult<Vec<PyHpoTerm>> {
        self.ids
            .iter()
            .map(|id| pyterm_from_id(id.as_u32()))
            .collect()
    }

    /// Instantiate an HPOSet from various inputs
    ///
    /// This is the most common way to instantiate HPOSet
    /// because it can use all kind of different inputs.
    /// Callers must ensure that each query paramater
    /// matches a single HPOTerm.
    ///
    /// Parameters
    /// ----------
    /// queries: list[str or int]
    ///
    ///     * **str** HPO term (e.g.: ``Scoliosis``)
    ///     * **str** HPO-ID (e.g.: ``HP:0002650``)
    ///     * **int** HPO term id (e.g.: ``2650``)
    ///
    /// Returns
    /// -------
    /// :class:`pyhpo.HPOSet`
    ///     A new ``HPOSet``
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
    /// ValueError
    ///     query cannot be converted to HpoTermId
    /// RuntimeError
    ///     No HPO term is found for the provided query
    ///
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     my_set = HPOSet.from_queries([
    ///         "HP:0002650",
    ///         118,
    ///         "Thoracolumbar scoliosis"
    ///     ])
    ///     len(my_set)
    ///     # >> 3
    ///
    #[classmethod]
    fn from_queries(_cls: &Bound<'_, PyType>, queries: Vec<PyQuery>) -> PyResult<Self> {
        let mut ids: Vec<HpoTermId> = Vec::with_capacity(queries.len());
        for q in queries {
            ids.push(term_from_query(q)?.id());
        }
        Ok(ids.into_iter().collect::<PyHpoSet>())
    }

    /// Instantiate an HPOSet from a serialized HPOSet
    ///
    /// This method is used when you have a serialized
    /// form of the HPOSet to share between applications.
    /// See :func:`pyhpo.HPOSet.serialize`
    ///
    /// Parameters
    /// ----------
    /// pickle: str
    ///     A pickled string of all HPOTerms, e.g. ``118+2650``
    ///
    /// Returns
    /// -------
    /// :class:`pyhpo.HPOSet`
    ///     A new ``HPOSet``
    ///
    /// Raises
    /// ------
    /// NameError
    ///     Ontology not yet constructed
    /// ValueError
    ///     pickled item cannot be converted to HpoTermId
    /// KeyError
    ///     No HPO term is found for the provided query
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     my_set = HPOSet.from_serialized("7+118+152+234+271+315+478+479+492+496")
    ///     len(my_set
    ///     # >> 10
    ///
    #[classmethod]
    fn from_serialized(_cls: &Bound<'_, PyType>, pickle: &str) -> PyResult<Self> {
        let ids: HpoGroup = pickle
            .split('+')
            .map(|id| id.parse::<u32>())
            .collect::<Result<Vec<u32>, ParseIntError>>()?
            .iter()
            .map(|id| {
                // in theory, we could simply call HpoTermId::from(*id)
                // here, but then we would not check for invalid input.
                // Instead we ensure we'll fail during instantiation
                // already
                Ok(term_from_id(*id)?.id().as_u32())
            })
            .collect::<PyResult<Vec<u32>>>()?
            .into();

        Ok(Self { ids })
    }

    /// Instantiate an HPOSet from a Gene
    ///
    /// Parameters
    /// ----------
    /// gene: :class:`pyhpo.Gene`
    ///     A gene from the ontology
    ///
    /// Returns
    /// -------
    /// :class:`pyhpo.HPOSet`
    ///     A new ``HPOSet``
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
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     gene_set = HPOSet.from_gene(Ontology.genes[0])
    ///     len(gene_set)
    ///     # >> 118
    ///
    #[classmethod]
    pub fn from_gene(_cls: &Bound<'_, PyType>, gene: &PyGene) -> PyResult<Self> {
        Self::try_from(gene)
    }

    /// Deprecated since 1.3.0
    /// Use :func:`pyhpo.HPOSet.from_omim_disease` instead
    #[classmethod]
    pub fn from_disease(_cls: &Bound<'_, PyType>, disease: &PyOmimDisease) -> PyResult<Self> {
        Self::try_from(disease)
    }

    /// Instantiate an HPOSet from an Omim disease
    ///
    /// Parameters
    /// ----------
    /// gene: :class:`pyhpo.Omim`
    ///     An Omim disease from the ontology
    ///
    /// Returns
    /// -------
    /// :class:`pyhpo.HPOSet`
    ///     A new ``HPOSet``
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
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     disease_set = HPOSet.from_omim_disease(Ontology.omim_diseases[0])
    ///     len(disease_set)
    ///     # >> 18
    ///
    #[classmethod]
    pub fn from_omim_disease(_cls: &Bound<'_, PyType>, disease: &PyOmimDisease) -> PyResult<Self> {
        Self::try_from(disease)
    }

    /// Instantiate an HPOSet from an Orpha disease
    ///
    /// Parameters
    /// ----------
    /// gene: :class:`pyhpo.Orpha`
    ///     An Orpha disease from the ontology
    ///
    /// Returns
    /// -------
    /// :class:`pyhpo.HPOSet`
    ///     A new ``HPOSet``
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
    ///     from pyhpo import Ontology
    ///     Ontology()
    ///     disease_set = HPOSet.from_orpha_disease(Ontology.orpha_diseases[0])
    ///     len(disease_set)
    ///     # >> 18
    ///
    #[classmethod]
    pub fn from_orpha_disease(
        _cls: &Bound<'_, PyType>,
        disease: &PyOrphaDisease,
    ) -> PyResult<Self> {
        Self::try_from(disease)
    }

    fn __len__(&self) -> usize {
        self.ids.len()
    }

    fn __repr__(&self) -> String {
        format!(
            "HPOSet.from_serialized(\"{}\")",
            self.ids
                .iter()
                .map(|i| i.as_u32().to_string())
                .collect::<Vec<String>>()
                .join("+")
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

    fn __iter__(&self) -> Iter {
        Iter::new(&self.ids)
    }

    fn __contains__(&self, term: &PyHpoTerm) -> bool {
        self.ids.contains(&term.hpo_term_id())
    }
}

impl<'a> PyHpoSet {
    pub fn set(&'a self, ont: &'a Ontology) -> HpoSet {
        HpoSet::new(ont, self.ids.clone())
    }
}

impl TryFrom<&PyGene> for PyHpoSet {
    type Error = PyErr;
    /// Tries to create a `PyHpoSet` from a `PyGene`
    ///
    /// # Errors
    /// - PyNameError: Ontology not yet created
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
    /// Tries to create a `PyHpoSet` from a `PyOmimDisease`
    ///
    /// # Errors
    /// - PyNameError: Ontology not yet created
    fn try_from(disease: &PyOmimDisease) -> Result<Self, Self::Error> {
        let ont = get_ontology()?;
        Ok(ont
            .omim_disease(&disease.id().into())
            .expect("ontology must. be present and gene must be included")
            .to_hpo_set(ont)
            .into())
    }
}

impl TryFrom<&PyOrphaDisease> for PyHpoSet {
    type Error = PyErr;
    /// Tries to create a `PyHpoSet` from a `PyOrphaDisease`
    ///
    /// # Errors
    /// - PyNameError: Ontology not yet created
    fn try_from(disease: &PyOrphaDisease) -> Result<Self, Self::Error> {
        let ont = get_ontology()?;
        Ok(ont
            .orpha_disease(&disease.id().into())
            .expect("ontology must. be present and gene must be included")
            .to_hpo_set(ont)
            .into())
    }
}

#[pyclass(name = "SetIterator")]
struct Iter {
    ids: VecDeque<HpoTermId>,
}

impl Iter {
    fn new(ids: &HpoGroup) -> Self {
        Self {
            ids: ids.iter().collect(),
        }
    }
}

#[pymethods]
impl Iter {
    #[allow(clippy::self_named_constructors)]
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<PyHpoTerm> {
        slf.ids
            .pop_front()
            .map(|id| pyterm_from_id(id.as_u32()).unwrap())
    }
}

#[pyclass(name = "BasicHPOSet")]
#[derive(Clone, Default, Debug)]
pub(crate) struct BasicPyHpoSet;

impl BasicPyHpoSet {
    fn build<I: IntoIterator<Item = HpoTermId>>(ids: I) -> Result<PyHpoSet, PyErr> {
        let ont = get_ontology().expect("Ontology must be initialized");
        let mut group = HpoGroup::new();
        for id in ids {
            group.insert(id);
        }
        let set = HpoSet::new(ont, group);
        let mut set = set.child_nodes();
        set.replace_obsolete();
        set.remove_obsolete();
        set.remove_modifier();
        PyHpoSet::new(
            set.iter()
                .map(|term| TermOrId::Id(term.id().as_u32()))
                .collect(),
        )
    }
}

#[pymethods]
impl BasicPyHpoSet {
    fn __call__(&self, terms: Vec<u32>) -> PyResult<PyHpoSet> {
        BasicPyHpoSet::build(terms.iter().map(|id| HpoTermId::from_u32(*id)))
    }

    #[classmethod]
    fn from_queries(_cls: &Bound<'_, PyType>, queries: Vec<PyQuery>) -> PyResult<PyHpoSet> {
        let mut ids: Vec<HpoTermId> = Vec::with_capacity(queries.len());
        for q in queries {
            ids.push(term_from_query(q)?.id());
        }
        BasicPyHpoSet::build(ids)
    }

    #[classmethod]
    fn from_serialized(_cls: &Bound<'_, PyType>, pickle: &str) -> PyResult<PyHpoSet> {
        BasicPyHpoSet::build(
            pickle
                .split('+')
                .map(|id| id.parse::<u32>())
                .collect::<Result<Vec<u32>, ParseIntError>>()?
                .iter()
                .map(|id| HpoTermId::from_u32(*id)),
        )
    }

    #[classmethod]
    pub fn from_gene(_cls: &Bound<'_, PyType>, gene: &PyGene) -> PyResult<PyHpoSet> {
        BasicPyHpoSet::build(gene.hpo()?.iter().map(|id| HpoTermId::from_u32(*id)))
    }

    /// Deprecated since 1.3.0
    #[classmethod]
    pub fn from_disease(_cls: &Bound<'_, PyType>, disease: &PyOmimDisease) -> PyResult<PyHpoSet> {
        BasicPyHpoSet::build(disease.hpo()?.iter().map(|id| HpoTermId::from_u32(*id)))
    }

    #[classmethod]
    pub fn from_omim_disease(
        _cls: &Bound<'_, PyType>,
        disease: &PyOmimDisease,
    ) -> PyResult<PyHpoSet> {
        BasicPyHpoSet::build(disease.hpo()?.iter().map(|id| HpoTermId::from_u32(*id)))
    }

    #[classmethod]
    pub fn from_orpha_disease(
        _cls: &Bound<'_, PyType>,
        disease: &PyOrphaDisease,
    ) -> PyResult<PyHpoSet> {
        BasicPyHpoSet::build(disease.hpo()?.iter().map(|id| HpoTermId::from_u32(*id)))
    }
}

#[pyclass(name = "HPOPhenoSet")]
#[derive(Clone, Default, Debug)]
pub(crate) struct PhenoSet;

impl PhenoSet {
    fn build<I: IntoIterator<Item = HpoTermId>>(ids: I) -> Result<PyHpoSet, PyErr> {
        let ont = get_ontology().expect("Ontology must be initialized");
        let mut group = HpoGroup::new();
        for id in ids {
            group.insert(id);
        }
        let mut set = HpoSet::new(ont, group);
        set.replace_obsolete();
        set.remove_obsolete();
        set.remove_modifier();
        PyHpoSet::new(
            set.iter()
                .map(|term| TermOrId::Id(term.id().as_u32()))
                .collect(),
        )
    }
}

#[pymethods]
impl PhenoSet {
    fn __call__(&self, terms: Vec<u32>) -> PyResult<PyHpoSet> {
        PhenoSet::build(terms.iter().map(|id| HpoTermId::from_u32(*id)))
    }

    #[classmethod]
    fn from_queries(_cls: &Bound<'_, PyType>, queries: Vec<PyQuery>) -> PyResult<PyHpoSet> {
        let mut ids: Vec<HpoTermId> = Vec::with_capacity(queries.len());
        for q in queries {
            ids.push(term_from_query(q)?.id());
        }
        PhenoSet::build(ids)
    }

    #[classmethod]
    fn from_serialized(_cls: &Bound<'_, PyType>, pickle: &str) -> PyResult<PyHpoSet> {
        PhenoSet::build(
            pickle
                .split('+')
                .map(|id| id.parse::<u32>())
                .collect::<Result<Vec<u32>, ParseIntError>>()?
                .iter()
                .map(|id| HpoTermId::from_u32(*id)),
        )
    }

    #[classmethod]
    pub fn from_gene(_cls: &Bound<'_, PyType>, gene: &PyGene) -> PyResult<PyHpoSet> {
        PhenoSet::build(gene.hpo()?.iter().map(|id| HpoTermId::from_u32(*id)))
    }

    /// Deprecated since 1.3.0
    #[classmethod]
    pub fn from_disease(_cls: &Bound<'_, PyType>, disease: &PyOmimDisease) -> PyResult<PyHpoSet> {
        PhenoSet::build(disease.hpo()?.iter().map(|id| HpoTermId::from_u32(*id)))
    }

    #[classmethod]
    pub fn from_omim_disease(
        _cls: &Bound<'_, PyType>,
        disease: &PyOmimDisease,
    ) -> PyResult<PyHpoSet> {
        PhenoSet::build(disease.hpo()?.iter().map(|id| HpoTermId::from_u32(*id)))
    }

    #[classmethod]
    pub fn from_orpha_disease(
        _cls: &Bound<'_, PyType>,
        disease: &PyOrphaDisease,
    ) -> PyResult<PyHpoSet> {
        PhenoSet::build(disease.hpo()?.iter().map(|id| HpoTermId::from_u32(*id)))
    }
}
