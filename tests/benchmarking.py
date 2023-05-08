# PYTHONPATH=./ python tests/benchmarking.py
import cProfile

from pyhpo import Ontology
from pyhpo import BasicHPOSet
from pyhpo.annotations import Omim


def diseases2basicsets(diseases):
    return [(
        d.name,
        BasicHPOSet.from_queries(list(d.hpo))
    ) for d in diseases]


def build_ontology():
    _ = Ontology()


def compare_set(diseases):
    gaucher = Omim.get(230800)
    set1 = BasicHPOSet.from_queries(list(gaucher.hpo))
    for x in diseases:
        set2 = BasicHPOSet.from_queries(list(x.hpo))
        _ = set1.similarity(set2)
    return None


print('===== LOADING ONTOLOGY ======')
cProfile.run('build_ontology()')

print('===== BUILDING DISEASE BasicSETS ======')
cProfile.run('diseases2basicsets(Ontology.omim_diseases)')

print('===== BUILDING GENE BasicSETS ======')
cProfile.run('diseases2basicsets(Ontology.genes)')

print('===== COMPARING SETS ======')
cProfile.run('compare_set(Ontology.omim_diseases)')
