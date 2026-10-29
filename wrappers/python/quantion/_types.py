from __future__ import annotations

from enum import Enum
from typing import Callable, List, TypedDict, Union

import numpy as np


class TimeUnit(str, Enum):
    """TimeUnit · Enums

    Unit of the retention times you give to a function. Results are always in minutes.

    Documentation: https://phenological.github.io/quantion/?language=python#TimeUnit
    """
    MINUTES = "MINUTES"
    SECONDS = "SECONDS"

    def __str__(self) -> str:
        return self.value


class PeakShape(str, Enum):
    """PeakShape · Enums

    Curve used to fit a peak.

    Documentation: https://phenological.github.io/quantion/?language=python#PeakShape
    """
    GAUSSIAN = "GAUSSIAN"
    EMG = "EMG"

    def __str__(self) -> str:
        return self.value


FromTo = TypedDict("FromTo", {"from": float, "to": float})
FromTo.__doc__ = """FromTo · Inputs

A range: retention-time windows and m/z ranges.

Documentation: https://phenological.github.io/quantion/?language=python#FromTo
"""


class _TargetFields(TypedDict):
    mz: float
    rt: float


class Target(_TargetFields, total=False):
    """Target · Inputs

    One compound. range limits the search in find_targets; missing or 0 searches the whole EIC. predict_rts and calibrate do not use range.

    Documentation: https://phenological.github.io/quantion/?language=python#Target
    """
    id: str
    range: float


class _TransitionFields(TypedDict):
    precursor: float
    product: float
    rt: float
    range: float


class Transition(_TransitionFields, total=False):
    """Transition · Inputs

    One SRM transition.

    Documentation: https://phenological.github.io/quantion/?language=python#Transition
    """
    id: str


class Header(TypedDict):
    """Header · Inputs

    One HTTP header sent with an upload in write_ion. Values are never printed. A line break, or Host, Content-Length or Transfer-Encoding, gives an invalid_argument error.

    Documentation: https://phenological.github.io/quantion/?language=python#Header
    """
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
FindPeaksOptions.__doc__ = """FindPeaksOptions · Options

Peak-picking settings. Every field is optional.

Documentation: https://phenological.github.io/quantion/?language=python#FindPeaksOptions
"""


class GetPeakOptions(TypedDict, total=False):
    """GetPeakOptions · Options

    Settings for get_peak.

    Documentation: https://phenological.github.io/quantion/?language=python#GetPeakOptions
    """
    range: float
    find_peaks: FindPeaksOptions


class FitPeakOptions(TypedDict, total=False):
    """FitPeakOptions · Options

    Settings for fit_peak.

    Documentation: https://phenological.github.io/quantion/?language=python#FitPeakOptions
    """
    shape: Union[PeakShape, str]
    intensity: float


DrawPeaksOptions = TypedDict("DrawPeaksOptions", {
    "from": float,
    "to": float,
    "points": int,
}, total=False)
DrawPeaksOptions.__doc__ = """DrawPeaksOptions · Options

Settings for draw_peaks.

Documentation: https://phenological.github.io/quantion/?language=python#DrawPeaksOptions
"""


CalculateBaselineOptions = TypedDict("CalculateBaselineOptions", {
    "lambda": float,
    "max_iterations": int,
}, total=False)
CalculateBaselineOptions.__doc__ = """CalculateBaselineOptions · Options

airPLS baseline settings for calculate_baseline.

Documentation: https://phenological.github.io/quantion/?language=python#CalculateBaselineOptions
"""


CalculateEicOptions = TypedDict("CalculateEicOptions", {
    "from": float,
    "to": float,
    "ppm_tolerance": float,
    "mz_tolerance": float,
    "time_unit": Union[TimeUnit, str],
}, total=False)
CalculateEicOptions.__doc__ = """CalculateEicOptions · Options

Which scans are read and how wide the m/z window is. Used by calculate_eic, and as the eic group of find_targets, find_features and match_features. The wider of the ppm and absolute m/z windows is used.

Documentation: https://phenological.github.io/quantion/?language=python#CalculateEicOptions
"""


class GridOptions(TypedDict, total=False):
    """GridOptions · Options

    The m/z grid that find_features and match_features search.

    Documentation: https://phenological.github.io/quantion/?language=python#GridOptions
    """
    start: float
    end: float
    step_size: float


class GroupingOptions(TypedDict, total=False):
    """GroupingOptions · Options

    How match_features matches features across samples.

    Documentation: https://phenological.github.io/quantion/?language=python#GroupingOptions
    """
    ppm_tolerance: float
    mz_tolerance: float
    rt_tolerance: float
    min_frequency: float


class FindTargetsOptions(TypedDict, total=False):
    """FindTargetsOptions · Options

    Settings for find_targets.

    Documentation: https://phenological.github.io/quantion/?language=python#FindTargetsOptions
    """
    eic: CalculateEicOptions
    find_peaks: FindPeaksOptions
    threads: int


class FindTransitionsOptions(TypedDict, total=False):
    """FindTransitionsOptions · Options

    Settings for find_transitions.

    Documentation: https://phenological.github.io/quantion/?language=python#FindTransitionsOptions
    """
    find_peaks: FindPeaksOptions
    threads: int


class FindFeaturesOptions(TypedDict, total=False):
    """FindFeaturesOptions · Options

    Settings for find_features.

    Documentation: https://phenological.github.io/quantion/?language=python#FindFeaturesOptions
    """
    eic: CalculateEicOptions
    grid: GridOptions
    find_peaks: FindPeaksOptions
    threads: int


class MatchFeaturesOptions(TypedDict, total=False):
    """MatchFeaturesOptions · Options

    Settings for match_features: the find_features settings plus grouping.

    Documentation: https://phenological.github.io/quantion/?language=python#MatchFeaturesOptions
    """
    eic: CalculateEicOptions
    grid: GridOptions
    grouping: GroupingOptions
    find_peaks: FindPeaksOptions
    threads: int


class GetScansOptions(TypedDict, total=False):
    """GetScansOptions · Options

    Settings for get_scans.

    Documentation: https://phenological.github.io/quantion/?language=python#GetScansOptions
    """
    level: int
    time_unit: Union[TimeUnit, str]


class ParseIonOptions(TypedDict, total=False):
    """ParseIonOptions · Options

    Settings for parse_ion.

    Documentation: https://phenological.github.io/quantion/?language=python#ParseIonOptions
    """
    max_cache_size: int


class WriteIonOptions(TypedDict, total=False):
    """WriteIonOptions · Options

    Settings for write_ion.

    Documentation: https://phenological.github.io/quantion/?language=python#WriteIonOptions
    """
    out: str
    overwrite: bool
    level: int
    f32_compress: bool
    headers: List[Header]


class WriteMzmlOptions(TypedDict, total=False):
    """WriteMzmlOptions · Options

    Settings for write_mzml.

    Documentation: https://phenological.github.io/quantion/?language=python#WriteMzmlOptions
    """
    out: str
    overwrite: bool


class GetIonImageOptions(TypedDict, total=False):
    """GetIonImageOptions · Options

    Settings for get_ion_image.

    Documentation: https://phenological.github.io/quantion/?language=python#GetIonImageOptions
    """
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
Peak.__doc__ = """Peak · Results

One chromatographic peak. No peak gives every field 0.

Documentation: https://phenological.github.io/quantion/?language=python#Peak
"""


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
    """Feature · Results

    A peak at an m/z. match_features adds frequency.

    Documentation: https://phenological.github.io/quantion/?language=python#Feature
    """
    frequency: float


class DataXY(TypedDict):
    """DataXY · Results

    A trace: x = retention times, y = intensities. Returned by calculate_eic, taken by find_peaks.

    Documentation: https://phenological.github.io/quantion/?language=python#DataXY
    """
    x: np.ndarray
    y: np.ndarray


class NoiseLevel(TypedDict):
    """NoiseLevel · Results

    Result of find_noise_level.

    Documentation: https://phenological.github.io/quantion/?language=python#NoiseLevel
    """
    width: int
    intensity: float


class _PeakParametersFields(TypedDict):
    height: float
    center: float
    fwhm: float
    tail: float


class PeakParameters(_PeakParametersFields, total=False):
    """PeakParameters · Results

    One peak curve: returned by fit_peak, or written by hand to simulate. draw_peaks takes a list of them. tail = 0 is a Gaussian, tail above 0 an EMG.

    Documentation: https://phenological.github.io/quantion/?language=python#PeakParameters
    """
    r2: float


class PredictedRt(TypedDict):
    """PredictedRt · Results

    One predicted retention time from predict_rts.

    Documentation: https://phenological.github.io/quantion/?language=python#PredictedRt
    """
    rt: float
    uncertainty: float


class Scan(TypedDict):
    """Scan · Results

    One scan from get_scans: its retention time, centroided m/z and intensity values, and its details.

    Documentation: https://phenological.github.io/quantion/?language=python#Scan
    """
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
    """IonImage · Results

    An imaging map of one m/z from get_ion_image.

    Documentation: https://phenological.github.io/quantion/?language=python#IonImage
    """
    width: int
    height: int
    min_x: int
    min_y: int
    min_z: int
    max_z: int
    data: np.ndarray
    counts: np.ndarray
