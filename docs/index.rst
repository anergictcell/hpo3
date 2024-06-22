=============
:math:`HPO_3`
=============

.. toctree::
    :maxdepth: 1
    :hidden:
    :caption: 🚀 Getting started:

    getting_started

.. toctree::
    :maxdepth: 1
    :hidden:
    :caption: 🖥️ Examples:

    examples

.. toctree::
    :maxdepth: 1
    :hidden:
    :caption: 📄 API documentation:

    ontology
    hpoterm
    hposet
    annotations
    stats
    helper

.. toctree::
    :maxdepth: 1
    :hidden:
    :caption: 💡 Other infos:

    pyhpo
    api_changes


Table of Contents
=================

* `HPO3`_
* :doc:`🚀 Getting started <getting_started>`
* :doc:`🖥️ Examples <examples>`
* :doc:`📄 Further Info <pyhpo>`

HPO3
====

:math:`HPO_3`. is a Python library to work with the Human Phenotype Ontology (HPO). It can calculate similarities between individual terms or between sets of terms. It can also calculate the enrichment of gene or disease associations to a set of HPO terms.


Main features
=============

* 👫 Identify patient cohorts based on clinical features
* 👨‍👧‍👦 Cluster patients or other clinical information for GWAS
* 🩻→🧬 Phenotype to Genotype studies
* 🍎🍊 HPO similarity analysis
* 🕸️ Graph based analysis of phenotypes, genes and diseases
* 🔗 Linkage calculation for dendogram analysis


The library is helpful for discovery of novel gene-disease associations and GWAS data analysis studies. At the same time, it can be used for oragnize clinical information of patients in research or diagnostic settings.

**hpo3** aims to be a drop-in replacement for `pyhpo <https://pypi.org/project/pyhpo/>`_, but is written in Rust and thus much much faster. Batchwise operations can also utilize multithreading, increasing the performance even more.

.. hint::

   You can also check out the `documentation <https://pyhpo.readthedocs.io>`_ of `pyhpo <https://pypi.org/project/pyhpo/>`_, the API is almost 100% identical.

   hpo3 does have some extra functionality in the :doc:`helper` module.

Installation
============
**hpo3** is provided as binary wheels for most platforms on PyPI, so in most cases you can just run

.. code-block:: bash

   pip install hpo3

**hpo3** ships with a prebuilt HPO Ontology by default, so you can start right away.

Examples
========

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
   # >> {'enrichment': 5.453829934109905e-05, 'fold': 33.67884615384615, 'count': 3, 'item': <Gene (PLOD1)>}


Examples for multithreading
===========================

**hpo3** shines even more when it comes to batchwise operations:


**Calculate the pairwise similarity of the HPOSets from all genes**

.. code-block:: python

   import itertools
   from pyhpo import Ontology, HPOSet, helper

   Ontology()

   gene_sets = [g.hpo_set() for g in Ontology.genes]
   gene_set_combinations = [
      (a[0], a[1]) for a in itertools.combinations(gene_sets, 2)
   ]

   similarities = helper.batch_set_similarity(
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

   similarities = helper.batch_set_similarity(
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

   # >> The most enriched disease for 730 | C7 is {
   # >>     'enrichment': 3.6762699175625894e-42,
   # >>     'fold': 972.9444444444443,
   # >>     'count': 13,
   # >>     'item': <OmimDisease (610102)>
   # >> }


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

   # >> The most enriched gene for 619510 | Immunodeficiency 85 and autoimmunity is {
   # >>     'enrichment': 7.207370728788139e-45,
   # >>     'fold': 66.0867924528302,
   # >>     'count': 24,
   # >>     'item': <Gene (TOM1)>
   # >> }
