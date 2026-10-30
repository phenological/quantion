from __future__ import annotations

import copy
import math
from typing import Any, Dict, Mapping, Optional

from quantion._errors import _invalid, _unknown_option
from quantion._types import PeakShape, TimeUnit

FIND_PEAKS = {
    "min_integral": None,
    "min_intensity": 500.0,
    "min_peak_width_points": 3,
    "noise": None,
    "auto_noise": True,
    "auto_baseline": True,
    "lambda": 20.0,
    "max_iterations": 4,
    "allow_overlap": False,
    "min_snr": 2.0,
    "min_r2": 0.0,
    "shape": "EMG",
    "kernel_size": 0,
}
EIC = {
    "from": 0.0,
    "to": math.inf,
    "ppm_tolerance": 20.0,
    "mz_tolerance": 0.005,
    "time_unit": "MINUTES",
}
GRID = {"start": 40.0, "end": 1000.0, "step_size": 0.005}
GROUPING = {"ppm_tolerance": 5.0, "mz_tolerance": 0.0025, "rt_tolerance": 0.05, "min_frequency": 0.0}
GET_PEAK = {"range": 0.0, "find_peaks": FIND_PEAKS}
FIT_PEAK = {"shape": "EMG", "intensity": 0.0}
DRAW_PEAKS = {"from": 0.0, "to": 0.0, "points": 1000}
BASELINE = {"lambda": 20.0, "max_iterations": 4}
FIND_TARGETS = {"eic": EIC, "find_peaks": FIND_PEAKS, "threads": 1}
FIND_TRANSITIONS = {"find_peaks": FIND_PEAKS, "threads": 1}
FIND_FEATURES = {"eic": EIC, "grid": GRID, "find_peaks": FIND_PEAKS, "threads": 1}
MATCH_FEATURES = {"eic": EIC, "grid": GRID, "grouping": GROUPING, "find_peaks": FIND_PEAKS, "threads": 1}
GET_SCANS = {"level": 1, "time_unit": "MINUTES"}
PARSE_ION = {"max_cache_size": 0}
WRITE_ION = {"out": None, "overwrite": False, "level": 12, "f32_compress": False, "headers": []}
WRITE_MZML = {"out": None, "overwrite": False}
ION_IMAGE = {"tolerance": 0.125, "level": 1, "on_progress": None}

_TIME_UNIT_CODES = {TimeUnit.MINUTES.value: 0, TimeUnit.SECONDS.value: 1}
_SHAPE_CODES = {PeakShape.GAUSSIAN.value: 0, PeakShape.EMG.value: 1}


def read_options(
    function: str,
    given: Optional[Mapping[str, Any]],
    defaults: Mapping[str, Any],
    prefix: str = "",
) -> Dict[str, Any]:
    if given is None:
        given = {}
    if not isinstance(given, Mapping):
        raise _invalid(function, prefix.rstrip(".") or "options", "must be a dict")
    for key in given:
        if key not in defaults:
            raise _unknown_option(function, f"{prefix}{key}", list(defaults))
    result: Dict[str, Any] = {}
    for key, default in defaults.items():
        if isinstance(default, Mapping):
            result[key] = read_options(function, given.get(key), default, f"{prefix}{key}.")
        elif key in given:
            result[key] = given[key]
        else:
            result[key] = copy.copy(default)
    return result


def get_number(function: str, argument: str, value: Any) -> float:
    if isinstance(value, bool):
        raise _invalid(function, argument, "must be a number, got a bool")
    try:
        return float(value)
    except (TypeError, ValueError):
        raise _invalid(function, argument, f"must be a number, got {value!r}") from None


def get_count(function: str, argument: str, value: Any) -> int:
    number = get_number(function, argument, value)
    if not math.isfinite(number) or number != int(number) or number < 0:
        raise _invalid(function, argument, f"must be a whole number 0 or more, got {value!r}")
    return int(number)


def get_flag(function: str, argument: str, value: Any) -> bool:
    if not isinstance(value, bool):
        raise _invalid(function, argument, f"must be True or False, got {value!r}")
    return value


def get_time_unit(function: str, argument: str, value: Any) -> int:
    text = value.value if isinstance(value, TimeUnit) else value
    if text not in _TIME_UNIT_CODES:
        raise _invalid(function, argument, f'must be "MINUTES" or "SECONDS", got {value!r}')
    return _TIME_UNIT_CODES[text]


def get_shape(function: str, argument: str, value: Any) -> int:
    text = value.value if isinstance(value, PeakShape) else value
    if text not in _SHAPE_CODES:
        raise _invalid(function, argument, f'must be "GAUSSIAN" or "EMG", got {value!r}')
    return _SHAPE_CODES[text]


def pack_peak_options(function: str, options: Mapping[str, Any], prefix: str = "find_peaks."):
    from quantion._bridge import PeakOptions

    def number(key: str) -> float:
        value = options[key]
        return math.nan if value is None else get_number(function, prefix + key, value)

    def count(key: str) -> int:
        return get_count(function, prefix + key, options[key])

    def flag(key: str) -> int:
        return int(get_flag(function, prefix + key, options[key]))

    packed = PeakOptions()
    packed.min_integral = number("min_integral")
    packed.min_intensity = number("min_intensity")
    packed.min_peak_width_points = count("min_peak_width_points")
    packed.shape = get_shape(function, prefix + "shape", options["shape"])
    packed.noise = number("noise")
    packed.auto_noise = flag("auto_noise")
    packed.auto_baseline = flag("auto_baseline")
    packed.lambda_ = number("lambda")
    packed.max_iterations = count("max_iterations")
    packed.allow_overlap = flag("allow_overlap")
    packed.padding = 0
    packed.min_snr = number("min_snr")
    packed.min_r2 = number("min_r2")
    packed.kernel_size = count("kernel_size")
    return packed
