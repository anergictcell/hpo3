Helper functions
================

For a lack of a better name, ``hpo3`` comes with a ``helper`` submodule that contains
some methods that fully utilize Rust's multithreading for batchwise large operations.
This is especially useful for large set data analysis. 


Methods
-------
.. autofunction:: pyhpo.helper.batch_similarity
.. autofunction:: pyhpo.helper.batch_set_similarity
.. autofunction:: pyhpo.helper.batch_disease_enrichment
.. autofunction:: pyhpo.helper.batch_omim_disease_enrichment
.. autofunction:: pyhpo.helper.batch_orpha_disease_enrichment
.. autofunction:: pyhpo.helper.batch_gene_enrichment
