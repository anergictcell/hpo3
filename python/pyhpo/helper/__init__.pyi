

from typing import Any, Dict, List, Tuple
from pyhpo.pyhpo import HPOSet

from pyhpo.pyhpo import HPOTerm


def batch_similarity(
    comparisons: List[Tuple[HPOTerm, HPOTerm]],
    kind:str,
    method: str
) -> List[float]: ...
def batch_set_similarity(
    comparisons: List[Tuple[HPOSet, HPOSet]],
    kind:str,
    method: str,
    combine: str
) -> List[float]: ...
def batch_gene_enrichment(hposets: List[HPOSet]) -> List[List[Dict[str, Any]]]: ...
def batch_disease_enrichment(hposets: List[HPOSet]) -> List[List[Dict[str, Any]]]: ...
def batch_omim_disease_enrichment(hposets: List[HPOSet]) -> List[List[Dict[str, Any]]]: ...
def batch_orpha_disease_enrichment(hposets: List[HPOSet]) -> List[List[Dict[str, Any]]]: ...