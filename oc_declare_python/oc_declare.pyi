# This is a Python stub file (.pyi) for the 'oc_declare' module.
# It is generated based on the provided Rust (PyO3) source code.
# It enables static type checking and IDE autocompletion.

from typing import List, Literal, Optional

__all__ = [
    "ProcessedOCEL",
    "OCDeclareArc",
    "import_ocel2",
    "discover",
    "check_conformance",
]

class ProcessedOCEL:
    """Pre-Processed OCEL"""
    # This is an opaque class, typically instantiated by `import_ocel2`
    ...

class OCDeclareArc:
    """An individual OC-DECLARE constraint arc"""

    def __init__(
        self,
        from_act: str,
        to_act: str,
        arc_type: Literal['AS', 'EF', 'EP', 'DF', 'DP'],
        min_count: Optional[int],
        max_count: Optional[int],
        /,
        all_ots: List[str] = ...,
        each_ots: List[str] = ...,
        any_ots: List[str] = ...
    ) -> None:
        """Construct a new OC-DECLARE arc"""
        ...

    def to_string(self) -> str:
        """Get string representation of OC-DECLARE arc"""
        ...

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

    @property
    def from_activity(self) -> str:
        """Get the source activity of the arc."""
        ...

    @from_activity.setter
    def from_activity(self, from_act: str) -> None:
        """Set the source activity of the arc."""
        ...

    @property
    def to_activity(self) -> str:
        """Get the target activity of the arc."""
        ...

    @to_activity.setter
    def to_activity(self, to_act: str) -> None:
        """Set the target activity of the arc."""
        ...

    @property
    def arc_type_name(self) -> str:
        """Get the type of the arc (e.g., "EF", "DF", "AS")."""
        ...

    @arc_type_name.setter
    def arc_type_name(self, arc_type: str) -> None:
        """Set the type of the arc (e.g., "EF", "DF", "AS")."""
        ...

    @property
    def all_ots(self) -> List[str]:
        """Get the object types involved with the 'ALL' quantifier."""
        ...

    @all_ots.setter
    def all_ots(self, all_ots: List[str]) -> None:
        """Set the object types involved with the 'ALL' quantifier."""
        ...

    @property
    def each_ots(self) -> List[str]:
        """Get the object types involved with the 'EACH' quantifier."""
        ...

    @each_ots.setter
    def each_ots(self, each_ots: List[str]) -> None:
        """Set the object types involved with the 'EACH' quantifier."""
        ...

    @property
    def any_ots(self) -> List[str]:
        """Get the object types involved with the 'ANY' quantifier."""
        ...

    @any_ots.setter
    def any_ots(self, any_ots: List[str]) -> None:
        """Set the object types involved with the 'ANY' quantifier."""
        ...

    @property
    def min_count(self) -> Optional[int]:
        """Get the minimum count for the arc."""
        ...

    @min_count.setter
    def min_count(self, min_count: Optional[int]) -> None:
        """Set the minimum count for the arc."""
        ...

    @property
    def max_count(self) -> Optional[int]:
        """Get the maximum count for the arc."""
        ...

    @max_count.setter
    def max_count(self, max_count: Optional[int]) -> None:
        """Set the maximum count for the arc."""
        ...


def import_ocel2(path: str, /) -> ProcessedOCEL:
    """Import an OCEL 2.0 file (.xml or .json) and preprocess it for use with OC-DECLARE"""
    ...

def discover(
    processed_ocel: ProcessedOCEL,
    /,
    noise_thresh: float = ...,
    acts_to_use: Optional[List[str]] = ...,
    o2o_mode: Optional[Literal['None', 'Direct', 'Reversed', 'Bidirectional']] = ...
) -> List[OCDeclareArc]:
    """Discover OC-DECLARE constraints given a pre-processed OCEL and a noise threshold"""
    ...

def check_conformance(
    processed_ocel: ProcessedOCEL, constraint: OCDeclareArc, /
) -> float:
    """
    Evaluate an OC-DECLARE constraint given a pre-processed OCEL
    yielding the fraction of relevant event satisfying the constraints

    Returns 1 if all source events fulfill the constraint and 0 if all source events violate the constraint.
    """
    ...