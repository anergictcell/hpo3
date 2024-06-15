use pyo3::{exceptions::PyRuntimeError, prelude::*};
use rayon::prelude::*;

use hpo::similarity::{GroupSimilarity, StandardCombiner};
use hpo::stats::Linkage;
use hpo::utils::Combinations;
use hpo::HpoSet;

use crate::{get_ontology, information_content::PyInformationContentKind, set::PyHpoSet};

/// Crate a linkage matrix from a list of ``HpoSet``\s to use in dendograms
/// or other hierarchical cluster analyses
///
///
/// Arguments
/// ---------
/// sets: list[:class:`pyhpo.HPOSet`]
///     The ``HPOSet``\s for which the linkage should be calculated
/// method: `str`, default: ``single``
///     The algorithm to use for clustering
///     
///     Available options:
///
///     * **single** : The minimum distance of each cluster's nodes to the other
///       nodes is used as distance for newly formed clusters. This is also known as the Nearest Point Algorithm.
///     * **union** : Create a new `HpoSet` for each cluster based on the union of
///       both combined clusters. This method becomes slow with growing input data
///     * **complete** : The maximum distance of each cluster's nodes to the other
///       nodes is used as distance for newly formed clusters. This is also known by the Farthest Point Algorithm
///       or Voor Hees Algorithm.
///     * **average** : The mean distance of each cluster's nodes to the other
///       nodes is used as distance for newly formed clusters. This is also called the UPGMA algorithm.
///
/// kind: `str`, default: `omim`
///     Which kind of information content to use for similarity calculation
///     
///     Available options:
///
///     * **omim**
///     * **orpha**
///     * **gene**
///
/// similarity_method: `str`, default `graphic`
///     The method to use to calculate the similarity between HPOSets.
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
/// combine: string, default ``funSimAvg``
///     The method to combine similarity measures.
///
///     Available options:
///
///     * **funSimAvg** - Schlicker A, BMC Bioinformatics, (2006)
///     * **funSimMax** - Schlicker A, BMC Bioinformatics, (2006)
///     * **BMA** - Deng Y, et. al., PLoS One, (2015)
///
/// Raises
/// ------
/// NameError
///     Ontology not yet constructed
/// KeyError
///     Invalid ``kind``
/// RuntimeError
///     Invalid ``method`` or ``similarity_method`` or ``combine``
///
/// Examples
/// --------
///
/// .. code-block:: python
///
///     import pyhpo
///     from pyhpo import Ontology, HPOSet
///     Ontology()
///
///     # Using 100 diseases and creating a Tuple of (Disease Name, HPOSet) for each
///     diseases = [(d.name, HPOSet(list(d.hpo)).remove_modifier()) for d in list(Ontology.omim_diseases)[0:100]]
///
///     # Creating one list with all HPOSets
///     disease_sets = [d[1] for d in diseases[0:100]]
///     # And one list with the names of diseases
///     names = [d[0] for d in diseases[0:100]]
///
///     # Cluster the diseases using default settings
///     lnk = pyhpo.stats.linkage(disease_sets)
///
///     # For plotting, you can use `scipy`
///     import scipy
///
///     scipy.cluster.hierarchy.dendrogram(lnk)
///
#[pyfunction]
#[pyo3(signature = (sets, method = "single", kind = "omim", similarity_method = "graphic", combine = "funSimAvg"))]
#[pyo3(text_signature = "(sets, method, kind, similarity_method, combine)")]
pub(crate) fn linkage(
    sets: Vec<PyHpoSet>,
    method: &str,
    kind: &str,
    similarity_method: &str,
    combine: &str,
) -> PyResult<Vec<(usize, usize, f32, usize)>> {
    let kind = PyInformationContentKind::try_from(kind)?;

    let similarity = hpo::similarity::Builtins::new(similarity_method, kind.into())
        .map_err(|_| PyRuntimeError::new_err("Unknown method to calculate similarity"))?;
    let combiner = StandardCombiner::try_from(combine)
        .map_err(|_| PyRuntimeError::new_err("Invalid combine method specified"))?;

    let sim = GroupSimilarity::new(combiner, similarity);

    let distance = |combs: Combinations<HpoSet<'_>>| {
        let x: Vec<(&HpoSet, &HpoSet)> = combs.collect();
        x.par_iter()
            .map(|comp| 1.0 - sim.calculate(comp.0, comp.1))
            .collect()
    };
    let ont = get_ontology()?;

    let sets = sets.iter().map(|pyset| pyset.set(ont));

    let res = match method {
        "single" => Linkage::single(sets, distance),
        "union" => Linkage::union(sets, distance),
        "complete" => Linkage::complete(sets, distance),
        "average" => Linkage::average(sets, distance),
        _ => return Err(PyRuntimeError::new_err("Not yet implemented")),
    };
    Ok(res
        .cluster()
        .map(|cluster| {
            (
                cluster.lhs(),
                cluster.rhs(),
                cluster.distance(),
                cluster.len(),
            )
        })
        .collect())
}
