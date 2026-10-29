from typing import Literal, TypedDict

FromTo = TypedDict("FromTo", {"from": float, "to": float})


class FindPeaksOptions(TypedDict, total=False):
    min_integral: float
    min_intensity: float
    min_peak_width_points: int
    noise: float
    auto_noise: bool
    auto_baseline: bool
    lambda_: int
    max_iterations: int
    allow_overlap: bool
    min_snr: float
    min_r2: float
    shape: Literal["gaussian", "emg"]
    kernel_size: int


class Transition(TypedDict):
    id: str
    precursor: float
    product: float
    rt: float
    range: float


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

_FeatureFields = TypedDict("_FeatureFields", {
    "mz": float,
    "rt": float,
    "from": float,
    "to": float,
    "intensity": float,
    "integral": float,
    "n_points": int,
    "noise": float,
})


class Feature(_FeatureFields, total=False):
    frequency: float
