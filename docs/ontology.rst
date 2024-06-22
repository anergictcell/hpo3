Ontology
========

Concept
-------
The :class:`pyhpo.Ontology` is the main component of **hpo3**, it contains references to all :class:`pyhpo.HPOTerm`\s, :class:`pyhpo.Gene` and Diseases (:class:`pyhpo.Omim`\, :class:`pyhpo.Orpha`). It is provided as a singleton and must be instantiated once to load all terms and annotations. Afterwards, the complete Ontology is available in the global scope across all submodules.

**hpo3** ships with a default provided Ontology that contains all terms, genes and diseases. You can also use your own version of custom annotations.


Instantiation
-------------
The `Ontology` must be instantiated once in every running program. This loads all HPO terms, their connections and annotation into memory.

Omitting all arguments will automatically load the built-in version. Alternatively, you can specify a binary data file or a folder that contains the JAX standard HPO data files.

Ontology()
~~~~~~~~~~
:Parameters:
    :data_folder: *(str)*
        Path to the source files (default: ``None``)
        Leave blank to load the builtin Ontology (recommended)
    :from_obo_file: *(bool)*
        Whether the input format is the standard from Jax HPO (default ``True``).
        Set to ``False`` to load a binary data source.
    :transitive: *(bool)*
        Load the ontology transitive, i.e. use the `phenotype_to_genes.txt` source instead to link
        terms to genes. This means that HPO-terms are transitively added to each gene.
        (default ``False``)
:Returns:
    ``None`` (calling ``Ontology`` instatiates the global ``Ontology`` singleton)


Examples
~~~~~~~~

.. code-block:: python

    from pyhpo import Ontology
    
    # load built-in default ontology
    Ontology()
    
    # check the release date of the HPO
    Ontology.version()
    # ==> '2024-04-26'

    term = Ontology.hpo(11968)
    term.name()  # ==> 'Feeding difficulties'
    term.id()    # ==> 'HP:0011968'
    int(tern)    # ==> 11968


.. code-block:: python

    from pyhpo import Ontology

    # load custom data from a local directory
    Ontology("/path/to/folder/")



The following code with multiple modules works, because the Ontology must only be loaded once:

**File main.py**

.. code-block:: python

    from pyhpo import Ontology

    import submodule
    from submodule import foo
    
    Ontology()
    
    foo()
    submodule.bar()


**File submodule.py**

.. code-block:: python

    from pyhpo import Ontology

    def foo():
        print(len(Ontology))

    def bar():
        print(len(Ontology))



API
---

Due to a limitation of Sphinx (or my understanding of it), the `Ontology` object here is written as `_Ontology`. Please disregard the underscore :)

.. autoclass:: pyhpo._Ontology
   :members:
   :inherited-members:


Iterating
---------
You can iterate all HPOTerms in the ontology. The iteration occurs in random order.

.. code-block:: python

    from pyhpo import Ontology
    Ontology()

    for term in Ontology:
        term.name()  # ==> 'Feeding difficulties'
        term.id()    # ==> 'HP:0011968'
        int(tern)    # ==> 11968

Length
------
The length of the Ontology indicates the number of HPOTerms within

.. code-block:: python

    from pyhpo import Ontology
    Ontology()

    len(Ontology)
    # ==> 18961
