class InformationContent:
    gene: float
    omim: float
    orpha: float
    custom: float
    def __getitem__(self, key: str) -> float: ...
