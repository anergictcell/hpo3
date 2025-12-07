from pyhpo.pyhpo import EnrichmentModel
from pyhpo.pyhpo import linkage

class HPOEnrichment:
    """
    Not implemented
    """
    def __init__(self, *args, **kwargs):
        pass

    def enrichment(self, *args, **kwargs):
        raise NotImplementedError


__all__ = (
    "EnrichmentModel",
    "linkage",
    "HPOEnrichment",
)
