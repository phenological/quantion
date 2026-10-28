from typing import TypedDict

FromTo = TypedDict("FromTo", {"from": float, "to": float})


class Target(TypedDict):
    id: str
    mz: float
    rt: float
    range: float


Peak = TypedDict("Peak", {
    "rt": float,
    "from": float,
    "to": float,
    "intensity": float,
    "integral": float,
    "n_points": int,
    "noise": float,
    "r2": float,
})

TargetPeak = TypedDict("TargetPeak", {
    "id": str,
    "mz": float,
    "ort": float,
    "rt": float,
    "from": float,
    "to": float,
    "intensity": float,
    "integral": float,
    "noise": float,
})

Feature = TypedDict("Feature", {
    "mz": float,
    "rt": float,
    "from": float,
    "to": float,
    "intensity": float,
    "integral": float,
    "n_points": int,
    "noise": float,
})

GroupedFeature = TypedDict("GroupedFeature", {
    "mz": float,
    "rt": float,
    "from": float,
    "to": float,
    "intensity": float,
    "integral": float,
    "frequency": float,
})
