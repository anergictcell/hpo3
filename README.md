# HPO3

`HPO3` is a Rust based drop-in replacement of [`PyHPO`](https://pypi.org/project/pyhpo/). It is based on the [`hpo`](https://crates.io/crates/hpo) Rust library which is a performance optimzied implementation of `PyHPO`.

Using the Rust-based `hpo` library increases performance easily 100 fold for many operations. It enables developers to utilize multithreading, further improving performance greatly.

`HPO3` aims to use the exact same API and methods as PyHPO to allow a very simple replacement for all analysis and statistics methods. It does not allow customization and modification of the ontology or individual terms, genes etc.

## Current status
The library is being actively developed right now and many things might change. Most functionality is present and working, though not extensively tested. If you require correct data and stability, keep using PyHPO. If you need performance and speed for rapid experiments, give `HPO3` a try.
Similarity calculations are implemented and working both for single terms and for HPOSets.
Enrichment method (e.g. hypergeometic enrichment) is not yet implemented.
I'm also planning to add some batchwise processing methods that can take full use of parallel processing in Rust, further improving the speed.

## Installation
The library is not available as pre-build binaries, so you must build it yourself. For this you need a stable Rust toolchain:

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
from hpo3 import Ontology
Ontology()
for term in Ontology:
    print(term.name)
```

If you need a full `PyHPO` drop-in replacement, you can create a mock `pyhpo` library using the following commands:

```bash
mkdir pyhpo
echo "from hpo3 import Ontology, HPOSet" > pyhpo/__init__.py
echo "from hpo3 import Omim, Gene" > pyhpo/annotations.py
```

If all this worked, you should be able to run the examples from the [PyHPO documentation](https://centogene.github.io/pyhpo/):

```python
from pyhpo import Ontology, HPOSet

# initilize the Ontology
_ = Ontology()

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
```


## Documentation
I'm in the process adding proper documentation, but the process is not yet that far. You can check in python by using `help(Ontology)` or `help(methodname)` to get some documentation.
Otherwise, use the [PyHPO documentation](https://centogene.github.io/pyhpo/) for now.
