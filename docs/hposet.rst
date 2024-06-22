HPOSet
======

An ``HPOSet`` is a collection of ``HPOTerm`` that can be used to document the clinical information of a patient. At the same time, the phenotypes associated with genes and diseases are also HPOSets.
``HPOSet`` can be instantiated in multiple ways, depending on the available data types. Whichever way you choose, you must instantiate the :doc:`ontology` beforehand.

Instantiation
-------------

.. autofunction:: pyhpo.HPOSet.from_queries
.. autofunction:: pyhpo.HPOSet.from_serialized
.. autofunction:: pyhpo.HPOSet.from_gene
.. autofunction:: pyhpo.HPOSet.from_disease
.. autofunction:: pyhpo.HPOSet.from_omim_disease
.. autofunction:: pyhpo.HPOSet.from_orpha_disease


Instance methods
----------------
.. autoclass:: pyhpo.HPOSet
    :members:   add, child_nodes, remove_modifier, replace_obsolete, terms, all_genes, omim_diseases, orpha_diseases, information_content, similarity, similarity_scores, toJSON, serialize


Not yet implemented
-------------------

The following instance methods are not yet implemented for :class:`pyhpo.HPOSet`

.. autofunction:: pyhpo.HPOSet.variance
.. autofunction:: pyhpo.HPOSet.combinations
.. autofunction:: pyhpo.HPOSet.combinations_one_way


BasicHPOSet
===========
A ``BasicHPOSet`` is like a normal :class:`pyhpo.HPOSet`,  but:

* only child terms are retained, non-specific parent terms are removed
* a obsolete terms are replaced or removed
* all modifier terms are removed

HPOPhenoSet
===========
A ``BasicHPOSet`` is like a normal :class:`pyhpo.HPOSet`,  but:

* a obsolete terms are replaced or removed
* all modifier terms are removed

======== ======  =========== ===========
Term     HPOSet  BasicHPOSet HPOPhenoSet
======== ======  =========== ===========
obsolete   ✅        ❌          ❌
modifier   ✅        ❌          ❌
parents    ✅        ❌          ✅
======== ======  =========== ===========

