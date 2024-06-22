PyHPO vs hpo3
=============

**hpo3** is designed as a drop-in replacement of ``PyHPO``. It is written in Rust and thus much faster, which enables large scale data analysis. It is almost 100% feature identical, but some obscure features might be missing. If that's the case, please open a `Github Issue <https://github.com/anergictcell/hpo3/issues/>`_.

In addition to feature parity, **hpo3** also offers some additional functionality for batchwise operations that are 100s of times faster than similar implementations in PyHPO.
The Rust backend also offers additional safety guarantees, such as immutability, reducing potential nasty RuntimeErrors. This immutability might come as a disadvantage though, if you actually want to modify the Ontology or terms. In cases where you need mutability, you must continue using PyHPO.

All that being said, I strongly recommend to using **hpo3**, as it is vastly superior.

Additional functionality:
-------------------------

* :func:`pyhpo.HPOTerm.similarity_scores` : Calculate similarity to many other ``HPOTerm`` in parallel.
* :func:`pyhpo.HPOSet.similarity_scores` : Calculate similarity to many other ``HPOSet`` in parallel.
* :func:`pyhpo.stats.linkage` : Cluster and linkage matrix analysis of ``HPOSet``\s for dendograms.
* :func:`pyhpo.helper.batch_similarity` : Calculate similarity scores of ``HPOTerm``\s in parallel.
* :func:`pyhpo.helper.batch_set_similarity` : Calculate similarity scores of ``HPOSet``\s in parallel.
* :func:`pyhpo.helper.batch_disease_enrichment` : Calculate enrichment of diseases in many ``HPOSet``\s in parallel.
* :func:`pyhpo.helper.batch_gene_enrichment` : Calculate enrichment of genes in many ``HPOSet``\s in parallel.

Missing or different functionality:
-----------------------------------

* Association of Decipher diseases to ``HPOTerm``\s
* custom ``InformationContent`` calculations
* ``Ontology.search`` does not include synonyms
* ``HPOSet.combinations``
* ``HPOSet.combinations_one_way``
* ``HPOSet.variance``
* ``HPOTerm.synonym``, ``HPOTerm.xref``, ``HPOTerm.definition`` and ``HPOTerm.comment`` are not present
* ``HPOTerm.path_to_other`` (minor implementation detail difference)
* ``Ontology.path`` (minor implementation detail difference)
