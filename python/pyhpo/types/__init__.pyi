class InformationContent:
    def gene(self) -> float: ...
    def omim(self) -> float: ...
    def orpha(self) -> float: ...
    def __getitem__(self, key: str) -> float: ...
