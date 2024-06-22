Installation
============
hpo3 is provided as binary wheels for most platforms on PyPI, so in most cases you can just run

.. code-block:: bash

   pip install hpo3

(For macOS, only Python 3.10 and 3.11 are supported, for both x64 and arm at the moment.)


Initializing the Ontology
=========================

hpo3 ships with a prebuilt HPO Ontology by default, so you can start right away.

.. code-block:: python

    from pyhpo import Ontology
    Ontology()

    # iterating all HPO terms
    for term in Ontology:
        print(term.name)

    # get a single term based on their HPO ID
    term = Ontology.hpo(118)

    # or by using the full Term ID
    term = Ontology.get_hpo_object("HP:0000118")


Updating the Ontology
=====================

While I try to keep the HPO ontology version updated, it might become outdated at some point. You can always check the used version:

.. code-block:: python

    from pyhpo import Ontology
    Ontology()
    Ontology.version()
    # => '2023-04-05'


If you require a newer version of the ontology, you can download the following files from JAX directly and load them
manually:

- ``hp.obo`` (from https://hpo.jax.org/app/data/ontology)
- ``genes_to_phenotype.txt`` (from https://hpo.jax.org/app/data/annotations)
- ``phenotype.hpoa`` (from https://hpo.jax.org/app/data/annotations)

Alternatively you can use ``phenotype_to_genes.txt`` (from https://hpo.jax.org/app/data/annotations),
that way the HPOTerms are transitively linked to genes. See https://github.com/anergictcell/hpo/issues/44 for details.


.. code-block:: python

    from pyhpo import Ontology
    Ontology("/path/to/folder/with/ontology/")

