from typing import Any, List, Tuple, TypedDict
from pyhpo import HPOSet, HPOTerm
from pyhpo.annotations import Gene, Omim


class EnrichmentOutput(TypedDict):
    enrichment: float
    fold: float
    count: int
    item: Gene | Omim

class HpoEnrichmentOutput(TypedDict):
    hpo: HPOTerm
    count: int
    enrichment: float

class EnrichmentModel:
    def __init__(self, category: str): ...
    def enrichment(
        self,
        method: str,
        hposet: HPOSet
    ) -> List[EnrichmentOutput]: ...


class HPOEnrichment:
    def __init__(self, category: str): ...
    def enrichment(
        self,
        method: str,
        annotation_sets: List[Omim | Gene]
    ) -> List[HpoEnrichmentOutput]: ...


def linkage(
    sets: List[HPOSet],
    method: str,
    kind: str,
    similarity_method: str,
    combine: str
) -> List[Tuple[int, int, float, int]]: ...
