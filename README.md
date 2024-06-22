[![Documentation](https://readthedocs.org/projects/hpo3/badge/?version=stable)](https://hpo3.readthedocs.io/en/stable/)
[![PyPi downloads](https://img.shields.io/pypi/dm/hpo3.svg?label=Pypi%20downloads)](https://pypi.org/project/hpo3/)
[![Latest release](https://img.shields.io/pypi/v/hpo3?label=Latest%20Release)](https://pypi.org/project/hpo3/)


# HPO3

**hpo3** is a Rust based drop-in replacement of [**PyHPO**](https://pypi.org/project/pyhpo/). It is based on the [**hpo**](https://crates.io/crates/hpo) Rust library which is a performance optimzied implementation of `PyHPO`.


## Main Features

- 👫 Identify patient cohorts based on clinical features
- 👨‍👧‍👦 Cluster patients or other clinical information for GWAS
- 🩻→🧬 Phenotype to Genotype studies
- 🍎🍊 HPO similarity analysis
- 🕸️ Graph based analysis of phenotypes, genes and diseases
- 🔬 Enrichment analysis of genes or diseases

**hpo3** allows working on individual terms ``HPOTerm``, a set of terms ``HPOSet`` and the full ``Ontology``.

The library is helpful for discovery of novel gene-disease associations and GWAS data analysis studies. At the same time, it can be used for oragnize clinical information of patients in research or diagnostic settings.

Using the Rust-based [**hpo**](https://crates.io/crates/hpo) library gives super fast performance that allows large analyses. It enables developers to utilize multithreading, further improving performance greatly.

**hpo3** aims to use the exact same API and methods as [PyHPO](https://pypi.org/project/pyhpo/) to allow a very simple replacement for all analysis and statistics methods. However, it does not allow customization and modification of the ontology or individual terms, genes etc.


## Installation
HPO3 is provided as binary wheels for most platforms on [PyPI](https://pypi.org/project/hpo3/), so in most cases you can just run
```bash
pip install hpo3
```
(For macOS, only Python 3.10 and 3.11 are supported, for both x64 and arm at the moment.)

hpo3 ships with a prebuilt HPO Ontology by default, so you can start right away.

## Examples

There are also more examples in the documentation of both [PyHPO](https://pyhpo.readthedocs.io/) and [hpo3](https://hpo3.readthedocs.io/)

```python
from pyhpo import Ontology, HPOSet

# initilize the Ontology
Ontology()

for term in Ontology:
    print(f"{term.id} | {term.name}")

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

# Get a list of all direct parent terms
for p in term.parents:
    print(p)
#> HP:0010674 | Abnormality of the curvature of the vertebral column

# Get a list of all ancestor (direct + indirect parent) terms
for p in term.all_parents:
    print(p)
#> HP:0000001 | All
#> HP:0011842 | Abnormal skeletal morphology
#> HP:0009121 | Abnormal axial skeleton morphology
#> HP:0033127 | Abnormality of the musculoskeletal system
#> HP:0010674 | Abnormality of the curvature of the vertebral column
#> HP:0000118 | Phenotypic abnormality
#> HP:0000924 | Abnormality of the skeletal system
#> HP:0000925 | Abnormality of the vertebral column

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

# Show the categories a term belongs to
for term in Ontology.hpo(10049).categories:
    print(term)
"""
HP:0033127 | Abnormality of the musculoskeletal system
HP:0040064 | Abnormality of limbs
"""

```


## Documentation
Check out the [hpo3 documentation](https://hpo3.readthedocs.io/en/latest/)

## Parallel processing
**hpo3** is using Rust as backend, so it's able to fully utilize parallel processing. To benefit from this even greater, `hpo3` provides some special helper functions for parallel batch processing in the `helper` submodule

### Similarity scores of HPOSets
Pairwise similarity comparison of `HPOSet`s. Specify a list of comparisons to run and `hpo3` calculates the result using all available CPUs.

Assume you want to compare the clinical information of a patient to the clinical information of 1000s of other patients:

```python
from pyhpo.helper import set_batch_similarity
from pyhpo import Ontology, HPOSet

Ontology()

main_patient = HPOSet.from_queries([
    'HP:0002943',
    'HP:0008458',
    'HP:0100884',
    'HP:0002944',
    'HP:0002751'
])

# 2 column table with
# - Patient Identifier
# - Comma separated HPO-terms
patient_source = """\
Patient_000001\tHP:0007587,HP:4000044,HP:0001845,HP:0041249,HP:0032648
Patient_000002\tHP:0034338,HP:0031955,HP:0003311,HP:0032564,HP:0100238
Patient_000003\tHP:0031096,HP:0410280,HP:0009899,HP:0002088,HP:0100204
Patient_000004\tHP:0030782,HP:0011439,HP:0009751,HP:0001433,HP:0030336
Patient_000005\tHP:0025029,HP:0033643,HP:0000957,HP:0005593,HP:0012486
Patient_000006\tHP:0009344,HP:0430016,HP:0005621,HP:0010043,HP:0030974
Patient_000007\tHP:0010760,HP:0009331,HP:0100119,HP:0012871,HP:0003653
Patient_000008\tHP:0001636,HP:0000561,HP:0009990,HP:3000075,HP:0007333
Patient_000009\tHP:0011675,HP:0011730,HP:0032729,HP:0032169,HP:0002888
Patient_000010\tHP:0004900,HP:0010761,HP:0020212,HP:0001806,HP:0033372
Patient_000011\tHP:0033336,HP:0025134,HP:0033815,HP:0032290,HP:0032472
Patient_000012\tHP:0004286,HP:0010543,HP:0007258,HP:0009582,HP:0005871
Patient_000013\tHP:0000273,HP:0031967,HP:0033305,HP:0010862,HP:0031750
Patient_000014\tHP:0031403,HP:0020134,HP:0011260,HP:0000826,HP:0030739
Patient_000015\tHP:0009966,HP:0034101,HP:0100736,HP:0032385,HP:0030152
Patient_000016\tHP:0011398,HP:0002165,HP:0000512,HP:0032028,HP:0007807
Patient_000017\tHP:0007465,HP:0031214,HP:0002575,HP:0007765,HP:0100404
Patient_000018\tHP:0033278,HP:0006937,HP:0008726,HP:0012142,HP:0100185
Patient_000019\tHP:0008365,HP:0033377,HP:0032463,HP:0033014,HP:0009338
Patient_000020\tHP:0012431,HP:0004415,HP:0001285,HP:0010747,HP:0008344
Patient_000021\tHP:0008722,HP:0003436,HP:0007313,HP:0031362,HP:0007236
Patient_000022\tHP:0000883,HP:0007542,HP:0012653,HP:0009411,HP:0031773
Patient_000023\tHP:0001083,HP:0030031,HP:0100349,HP:0001120,HP:0010835
Patient_000024\tHP:0410210,HP:0009341,HP:0100811,HP:0032710,HP:0410064
Patient_000025\tHP:0001056,HP:0005561,HP:0003690,HP:0040157,HP:0100059
Patient_000026\tHP:0010651,HP:0500020,HP:0100603,HP:0033443,HP:0008288
Patient_000027\tHP:0012330,HP:0034395,HP:0004066,HP:0000554,HP:0002257
Patient_000028\tHP:0031484,HP:0100423,HP:0030487,HP:0033538,HP:0003172
Patient_000029\tHP:0030901,HP:0025136,HP:0034367,HP:0034101,HP:0045017
Patient_000030\tHP:0100957,HP:0010027,HP:0010806,HP:0020185,HP:0001421
Patient_000031\tHP:0001671,HP:0003885,HP:0001464,HP:0000243,HP:0009549
Patient_000032\tHP:0003521,HP:0003109,HP:0000433,HP:0030647,HP:0100280
Patient_000033\tHP:0006394,HP:0031598,HP:0032199,HP:0010428,HP:0000108
Patient_000034\tHP:0001468,HP:0008689,HP:0410030,HP:0012226,HP:0011388
Patient_000035\tHP:0003536,HP:0001011,HP:0033262,HP:0009978,HP:0025586
Patient_000036\tHP:0031849,HP:0005244,HP:0001664,HP:0041233,HP:0030921
Patient_000037\tHP:0005616,HP:0003874,HP:0011744,HP:0033751,HP:0007971
Patient_000038\tHP:0012836,HP:0033858,HP:0003427,HP:0033880,HP:0030481
Patient_000039\tHP:0100369,HP:0040317,HP:0010561,HP:0010522,HP:0011339
Patient_000040\tHP:0005338,HP:0040179,HP:0004258,HP:0030589,HP:0032981
Patient_000041\tHP:0011758,HP:0033519,HP:0032010,HP:0030710,HP:0010419
Patient_000042\tHP:0002642,HP:0006335,HP:0009895,HP:0001928,HP:0003779
Patient_000043\tHP:0002867,HP:0030404,HP:0033495,HP:0011143,HP:0012642
Patient_000044\tHP:0033432,HP:0005195,HP:0009062,HP:0100617,HP:0033586
Patient_000045\tHP:0011740,HP:0100159,HP:0033480,HP:3000069,HP:0011394
Patient_000046\tHP:0033350,HP:0009840,HP:0040247,HP:0040204,HP:0033099
Patient_000047\tHP:0030323,HP:0032005,HP:0033675,HP:0033869,HP:0010850
Patient_000048\tHP:0003411,HP:0100953,HP:0005532,HP:0032119,HP:0012157
Patient_000049\tHP:0030592,HP:0011691,HP:0010498,HP:0030196,HP:0006414
Patient_000050\tHP:0001549,HP:0040258,HP:0007078,HP:0000657,HP:3000066
"""

comparisons = []

for patient in patient_source.splitlines():
    _, terms = patient.split("\t")
    comparisons.append(
        (
            main_patient,
            HPOSet.from_queries(terms.split(","))
        )
    )

similarities = set_batch_similarity(
    comparisons,
    kind="omim",
    method="graphic",
    combine="funSimMax"
)
```
(This functionality works well with dataframes, such as `pandas` or `polars`, adding the similarity scores as a new series)

### Gene and Disease enrichments in HPOSets

Calculate the gene enrichment in several HPOSets in parallel

```python
from pyhpo.helper import batch_gene_enrichment
from pyhpo.helper import batch_disease_enrichment
from pyhpo import Ontology, HPOSet

Ontology()

# 2 column table with
# - Patient Identifier
# - Comma separated HPO-terms
patient_source = """\
Patient_000001\tHP:0007587,HP:4000044,HP:0001845,HP:0041249,HP:0032648
Patient_000002\tHP:0034338,HP:0031955,HP:0003311,HP:0032564,HP:0100238
Patient_000003\tHP:0031096,HP:0410280,HP:0009899,HP:0002088,HP:0100204
Patient_000004\tHP:0030782,HP:0011439,HP:0009751,HP:0001433,HP:0030336
Patient_000005\tHP:0025029,HP:0033643,HP:0000957,HP:0005593,HP:0012486
Patient_000006\tHP:0009344,HP:0430016,HP:0005621,HP:0010043,HP:0030974
Patient_000007\tHP:0010760,HP:0009331,HP:0100119,HP:0012871,HP:0003653
Patient_000008\tHP:0001636,HP:0000561,HP:0009990,HP:3000075,HP:0007333
Patient_000009\tHP:0011675,HP:0011730,HP:0032729,HP:0032169,HP:0002888
Patient_000010\tHP:0004900,HP:0010761,HP:0020212,HP:0001806,HP:0033372
Patient_000011\tHP:0033336,HP:0025134,HP:0033815,HP:0032290,HP:0032472
Patient_000012\tHP:0004286,HP:0010543,HP:0007258,HP:0009582,HP:0005871
Patient_000013\tHP:0000273,HP:0031967,HP:0033305,HP:0010862,HP:0031750
Patient_000014\tHP:0031403,HP:0020134,HP:0011260,HP:0000826,HP:0030739
Patient_000015\tHP:0009966,HP:0034101,HP:0100736,HP:0032385,HP:0030152
Patient_000016\tHP:0011398,HP:0002165,HP:0000512,HP:0032028,HP:0007807
Patient_000017\tHP:0007465,HP:0031214,HP:0002575,HP:0007765,HP:0100404
Patient_000018\tHP:0033278,HP:0006937,HP:0008726,HP:0012142,HP:0100185
Patient_000019\tHP:0008365,HP:0033377,HP:0032463,HP:0033014,HP:0009338
Patient_000020\tHP:0012431,HP:0004415,HP:0001285,HP:0010747,HP:0008344
Patient_000021\tHP:0008722,HP:0003436,HP:0007313,HP:0031362,HP:0007236
Patient_000022\tHP:0000883,HP:0007542,HP:0012653,HP:0009411,HP:0031773
Patient_000023\tHP:0001083,HP:0030031,HP:0100349,HP:0001120,HP:0010835
Patient_000024\tHP:0410210,HP:0009341,HP:0100811,HP:0032710,HP:0410064
Patient_000025\tHP:0001056,HP:0005561,HP:0003690,HP:0040157,HP:0100059
Patient_000026\tHP:0010651,HP:0500020,HP:0100603,HP:0033443,HP:0008288
Patient_000027\tHP:0012330,HP:0034395,HP:0004066,HP:0000554,HP:0002257
Patient_000028\tHP:0031484,HP:0100423,HP:0030487,HP:0033538,HP:0003172
Patient_000029\tHP:0030901,HP:0025136,HP:0034367,HP:0034101,HP:0045017
Patient_000030\tHP:0100957,HP:0010027,HP:0010806,HP:0020185,HP:0001421
Patient_000031\tHP:0001671,HP:0003885,HP:0001464,HP:0000243,HP:0009549
Patient_000032\tHP:0003521,HP:0003109,HP:0000433,HP:0030647,HP:0100280
Patient_000033\tHP:0006394,HP:0031598,HP:0032199,HP:0010428,HP:0000108
Patient_000034\tHP:0001468,HP:0008689,HP:0410030,HP:0012226,HP:0011388
Patient_000035\tHP:0003536,HP:0001011,HP:0033262,HP:0009978,HP:0025586
Patient_000036\tHP:0031849,HP:0005244,HP:0001664,HP:0041233,HP:0030921
Patient_000037\tHP:0005616,HP:0003874,HP:0011744,HP:0033751,HP:0007971
Patient_000038\tHP:0012836,HP:0033858,HP:0003427,HP:0033880,HP:0030481
Patient_000039\tHP:0100369,HP:0040317,HP:0010561,HP:0010522,HP:0011339
Patient_000040\tHP:0005338,HP:0040179,HP:0004258,HP:0030589,HP:0032981
Patient_000041\tHP:0011758,HP:0033519,HP:0032010,HP:0030710,HP:0010419
Patient_000042\tHP:0002642,HP:0006335,HP:0009895,HP:0001928,HP:0003779
Patient_000043\tHP:0002867,HP:0030404,HP:0033495,HP:0011143,HP:0012642
Patient_000044\tHP:0033432,HP:0005195,HP:0009062,HP:0100617,HP:0033586
Patient_000045\tHP:0011740,HP:0100159,HP:0033480,HP:3000069,HP:0011394
Patient_000046\tHP:0033350,HP:0009840,HP:0040247,HP:0040204,HP:0033099
Patient_000047\tHP:0030323,HP:0032005,HP:0033675,HP:0033869,HP:0010850
Patient_000048\tHP:0003411,HP:0100953,HP:0005532,HP:0032119,HP:0012157
Patient_000049\tHP:0030592,HP:0011691,HP:0010498,HP:0030196,HP:0006414
Patient_000050\tHP:0001549,HP:0040258,HP:0007078,HP:0000657,HP:3000066
"""

hpo_sets = []
for patient in patient_source.splitlines():
    _, terms = patient.split("\t")
    hpo_sets.append(HPOSet.from_queries(terms.split(",")))

gene_enrichments = batch_gene_enrichment(hpo_sets)
disease_enrichments = batch_disease_enrichment(hpo_sets)
```

## Development
**hpo3** is completely written in Rust, so you require a stable Rust toolchain:

Rust installation instructions as [on the official website](https://www.rust-lang.org/tools/install):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then clone this repository:
```bash
git clone https://github.com/anergictcell/hpo3
cd hpo3
```

Create a Python virtual environment and install maturin:
```bash
virtualenv venv
source venv/bin/activate
pip install maturin
```

And finally build and install the Python library
```bash
maturin develop -r
```

Aaaaand, you're done:
```bash
python
```

```python
from pyhpo import Ontology
Ontology()
for term in Ontology:
    print(term.name)
```
