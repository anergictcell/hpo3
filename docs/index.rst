
:math:`HPO_3`
=============

.. currentmodule:: pyhpo

.. toctree::
   :maxdepth: 1
   :caption: Table of Contents:

   getting_started
   ontology
   hpoterm
   hposet
   annotations
   enrichment

HPO3
----

:math:`HPO_3`. is a Python module to work with the Human Phenotype Ontology (HPO). It can calculate similarities between individual terms or between sets of terms. It can also calculate the enrichment of gene or disease associations to a set of HPO terms.

This library aims to be a drop-in replacement for `pyhpo <https://pypi.org/project/pyhpo/>`_, but is written in Rust and thus much much faster. Batchwise operations can also utilize multithreading, increasing the performance even more.

For a user guide and API description, you can check out the documentation of `pyhpo <https://pypi.org/project/pyhpo/>`_, the API is almost 100% identical.

Installation
~~~~~~~~~~~~
HPO3 is provided as binary wheels for most platforms on PyPI, so in most cases you can just run

.. code-block:: bash

   bash
   pip install hpo3

(For macOS, only Python 3.10 and 3.11 are supported, for both x64 and arm at the moment.)

hpo3 ships with a prebuilt HPO Ontology by default, so you can start right away.

Examples
~~~~~~~~

.. code-block:: python

   from pyhpo import stats, Ontology, HPOSet, Gene

   # initilize the Ontology
   Ontology()

   # Declare the clinical information of the patients
   patient_1 = HPOSet.from_queries([
      'HP:0002943',
      'HP:0008458',
      'HP:0100884',
      'HP:0002944',
      'HP:0002751'
   ])

   patient_2 = HPOSet.from_queries([
      'HP:0002650',
      'HP:0010674',
      'HP:0000925',
      'HP:0009121'
   ])

   # and compare their similarity
   patient_1.similarity(patient_2)
   #> 0.7594183905785477

   # Retrieve a term e.g. via its HPO-ID
   term = Ontology.get_hpo_object('Scoliosis')

   print(term)
   #> HP:0002650 | Scoliosis

   # Get information content from Term <--> Omim associations
   term.information_content['omim']
   #> 2.29

   # Show how many genes are associated to the term
   # (Note that this includes indirect associations, associations
   # from children terms to genes.)
   len(term.genes)
   #> 1094

   # Show how many Omim Diseases are associated to the term
   # (Note that this includes indirect associations, associations
   # from children terms to diseases.)
   len(term.omim_diseases)
   #> 844

   # Get a list of all parent terms
   for p in term.parents:
      print(p)
   #> HP:0010674 | Abnormality of the curvature of the vertebral column

   # Get a list of all children terms
   for p in term.children:
      print(p)
   """
   HP:0002944 | Thoracolumbar scoliosis
   HP:0008458 | Progressive congenital scoliosis
   HP:0100884 | Compensatory scoliosis
   HP:0002944 | Thoracolumbar scoliosis
   HP:0002751 | Kyphoscoliosis
   """

   # Calculate the enrichment of genes in an HPOSet
   gene_model = stats.EnrichmentModel('gene')
   genes = gene_model.enrichment(method='hypergeom', hposet=patient_1)

   print(genes[0])
   # >> {'enrichment': 5.729299915113426e-05, 'fold': 33.12427184466019, 'count': 3, 'item': 5351}

   # get the `Gene` object from the `item` id field
   gene = Gene.get(genes[0]["item"])
   gene.name
   # >> 'PLOD1'


`HPO3` shines even more when it comes to batchwise operations:


**Calculate the pairwise similarity of the HPOSets from all genes**

.. code-block:: python

   import itertools
   from pyhpo import Ontology, HPOSet, helper

   Ontology()

   gene_sets = [g.hpo_set() for g in Ontology.genes]
   gene_set_combinations = [
      (a[0], a[1]) for a in itertools.combinations(gene_sets, 2)
   ]

   similarities = helper.set_batch_similarity(
      gene_set_combinations[0:1000],  # only calculating for for 1000 comparisons to save time
      kind="omim",
      method="graphic",
      combine="funSimAvg"
   )


**Calculate the similarity of of a patient's HPO term to all diseases**

.. code-block:: python

   import itertools
   from pyhpo import Ontology, HPOSet, helper

   Ontology()

   patient_1 = HPOSet.from_queries([
      'HP:0002943',
      'HP:0008458',
      'HP:0100884',
      'HP:0002944',
      'HP:0002751'
   ])

   # casting the gene set to a list to main order for later lookups
   genes = list(Ontology.genes)
   comparisons = [(patient_1, g.hpo_set()) for g in genes]

   similarities = helper.set_batch_similarity(
      comparisons,
      kind="omim",
      method="graphic",
      combine="funSimAvg"
   )

   # Get most similar gene
   top_score = max(similarities)
   genes[similarities.index(top_score)]
   # >> <Gene (POP1)>


**Calculate the disease enrichment for every gene's HPOSet**

.. code-block:: python

   import itertools
   from pyhpo import Ontology, HPOSet, helper

   Ontology()

   # casting the gene set to a list to main order for later lookups
   genes = list(Ontology.genes)
   gene_sets = [g.hpo_set() for g in genes]

   enrichments = helper.batch_disease_enrichment(gene_sets)
   print(f"The most enriched disease for {genes[0]} is {enrichments[0][0]}")


**Or vice versa (genes enriched for every disease)**

.. code-block:: python

   import itertools
   from pyhpo import Ontology, HPOSet, helper

   Ontology()

   # casting the gene set to a list to main order for later lookups
   diseases = list(Ontology.omim_diseases)
   disease_sets = [d.hpo_set() for d in diseases]

   enrichments = helper.batch_gene_enrichment(disease_sets)
   print(f"The most enriched gene for {diseases[0]} is {enrichments[0][0]}")
