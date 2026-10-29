from __future__ import annotations

from enum import Enum
from typing import Callable, List, TypedDict, Union

import numpy as np


class TimeUnit(str, Enum):
    MINUTES = "MINUTES"
    SECONDS = "SECONDS"

    def __str__(self) -> str:
        return self.value


class PeakShape(str, Enum):
    GAUSSIAN = "GAUSSIAN"
    EMG = "EMG"

    def __str__(self) -> str:
        return self.value


FromTo = TypedDict("FromTo", {"from": float, "to": float})


class _TargetFields(TypedDict):
    mz: float
    rt: float


class Target(_TargetFields, total=False):
    id: str
    range: float


class _TransitionFields(TypedDict):
    precursor: float
    product: float
    rt: float
    range: float


class Transition(_TransitionFields, total=False):
    id: str


class Header(TypedDict):
    name: str
    value: str


FindPeaksOptions = TypedDict("FindPeaksOptions", {
    "min_integral": float,
    "min_intensity": float,
    "min_peak_width_points": int,
    "noise": float,
    "auto_noise": bool,
    "auto_baseline": bool,
    "lambda": float,
    "max_iterations": int,
    "allow_overlap": bool,
    "min_snr": float,
    "min_r2": float,
    "shape": Union[PeakShape, str],
    "kernel_size": int,
}, total=False)


class GetPeakOptions(TypedDict, total=False):
    range: float
    find_peaks: FindPeaksOptions


class FitPeakOptions(TypedDict, total=False):
    shape: Union[PeakShape, str]
    intensity: float


DrawPeaksOptions = TypedDict("DrawPeaksOptions", {
    "from": float,
    "to": float,
    "points": int,
}, total=False)


CalculateBaselineOptions = TypedDict("CalculateBaselineOptions", {
    "lambda": float,
    "max_iterations": int,
}, total=False)


CalculateEicOptions = TypedDict("CalculateEicOptions", {
    "from": float,
    "to": float,
    "ppm_tolerance": float,
    "mz_tolerance": float,
    "time_unit": Union[TimeUnit, str],
}, total=False)


class GridOptions(TypedDict, total=False):
    start: float
    end: float
    step_size: float


class GroupingOptions(TypedDict, total=False):
    ppm_tolerance: float
    mz_tolerance: float
    rt_tolerance: float
    min_frequency: float


class FindTargetsOptions(TypedDict, total=False):
    eic: CalculateEicOptions
    find_peaks: FindPeaksOptions
    threads: int


class FindTransitionsOptions(TypedDict, total=False):
    find_peaks: FindPeaksOptions
    threads: int


class FindFeaturesOptions(TypedDict, total=False):
    eic: CalculateEicOptions
    grid: GridOptions
    find_peaks: FindPeaksOptions
    threads: int


class MatchFeaturesOptions(TypedDict, total=False):
    eic: CalculateEicOptions
    grid: GridOptions
    grouping: GroupingOptions
    find_peaks: FindPeaksOptions
    threads: int


class GetScansOptions(TypedDict, total=False):
    level: int
    time_unit: Union[TimeUnit, str]


class ParseIonOptions(TypedDict, total=False):
    max_cache_size: int


class WriteIonOptions(TypedDict, total=False):
    out: str
    overwrite: bool
    level: int
    f32_compress: bool
    headers: List[Header]


class WriteMzmlOptions(TypedDict, total=False):
    out: str
    overwrite: bool


class GetIonImageOptions(TypedDict, total=False):
    tolerance: float
    level: int
    on_progress: Callable[[int, int, int], None]


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


class DataXY(TypedDict):
    x: np.ndarray
    y: np.ndarray


class NoiseLevel(TypedDict):
    width: int
    intensity: float


class _PeakParametersFields(TypedDict):
    height: float
    center: float
    fwhm: float
    tail: float


class PeakParameters(_PeakParametersFields, total=False):
    r2: float


class PredictedRt(TypedDict):
    rt: float
    uncertainty: float


class Scan(TypedDict):
    rt: float
    mz: np.ndarray
    intensity: np.ndarray
    level: int
    precursor_mz: float
    base_peak_mz: float
    base_peak_intensity: float
    total_ion_current: float
    polarity: int
    x: int
    y: int
    z: int


class IonImage(TypedDict):
    width: int
    height: int
    min_x: int
    min_y: int
    min_z: int
    max_z: int
    data: np.ndarray
    counts: np.ndarray
