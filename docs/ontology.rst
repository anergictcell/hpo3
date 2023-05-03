Ontology
========

The Ontology is provided as a singleton. It must be called once to load all terms and annotations. Afterwards, the complete Ontology is available from the global scope across all submodules.


Construct
---------
Arguments
~~~~~~~~~
path: str
    Path to the source files (default: `./ontology.hpo`)
binary: bool
    Whether the input format is binary (default true)

Examples
~~~~~~~~

.. code-block:: python

    from pyhpo import Ontology
    
    ont = Ontology("path/to/ontology.hpo")
    
    term = ont.hpo(11968)
    term.name()  # ==> 'Feeding difficulties'
    term.id()    # ==> 'HP:0011968'
    int(tern)    # ==> 11968


The following code with multiple modules works, because the Ontology must only be loaded once:

**File main.py**

.. code-block:: python

    from pyhpo import Ontology

    import submodule
    from submodule import foo
    
    ont = Ontology("path/to/ontology.hpo")
    
    foo()
    submodule.bar()


**File submodule.py**

.. code-block:: python

    from pyhpo import Ontology

    def foo():
        print(len(Ontology))

    def bar():
        print(len(Ontology))


Attributes
----------
.. autoattribute:: pyhpo.Ontology.__class__.genes
.. autoattribute:: pyhpo.Ontology.__class__.omim_diseases

Methods
-------
.. autofunction:: pyhpo.Ontology.__class__.__call__
.. autofunction:: pyhpo.Ontology.__class__.get_hpo_object
.. autofunction:: pyhpo.Ontology.__class__.match
.. autofunction:: pyhpo.Ontology.__class__.path
.. autofunction:: pyhpo.Ontology.__class__.search
.. autofunction:: pyhpo.Ontology.__class__.hpo
