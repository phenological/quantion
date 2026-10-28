from __future__ import annotations

from quantion._bridge import _ABI, load_library
from quantion import api as _api
from quantion.api import (
    parse_mzml,
    parse_ion,
    ion_to_mzml,
    mzml_to_ion,
    mzml_to_ion_file,
    calculate_eic,
    get_scans,
    find_peaks,
    get_peak,
    fit_peak,
    draw_peak,
    find_noise_level,
    calculate_baseline,
    get_peaks_from_eic,
    get_peaks_from_chrom,
    find_features,
    find_feature,
    get_features,
    load_model,
    load_warp,
)
from quantion.file import SampleFile
from quantion.model import Model
from quantion.warp_model import WarpModel
from quantion._pack import pack_peak_options, encode_target_ids, unpack_targets
from quantion._shared import to_threads
from quantion._types import ChromTarget, FromTo, GroupedFeature, Feature, Peak, Target, TargetPeak

__version__ = "0.1.6"

__all__ = [
    "Quantion",
    "parse_mzml",
    "parse_ion",
    "ion_to_mzml",
    "mzml_to_ion",
    "mzml_to_ion_file",
    "calculate_eic",
    "get_scans",
    "find_peaks",
    "get_peak",
    "fit_peak",
    "draw_peak",
    "find_noise_level",
    "calculate_baseline",
    "get_peaks_from_eic",
    "get_peaks_from_chrom",
    "find_features",
    "find_feature",
    "get_features",
    "SampleFile",
    "load_model",
    "load_warp",
    "Model",
    "WarpModel",
    "FromTo",
    "Target",
    "ChromTarget",
    "Peak",
    "TargetPeak",
    "Feature",
    "GroupedFeature",
    "pack_peak_options",
    "encode_target_ids",
    "unpack_targets",
    "to_threads",
]


class Quantion:
    def __init__(self, abi: _ABI) -> None:
        self._abi = abi
        _api._set_abi(abi)

    def parse_mzml(self, data: bytes) -> SampleFile:
        return parse_mzml(data)

    def parse_ion(self, source, max_cache_size: int = 0) -> SampleFile:
        return parse_ion(source, max_cache_size)

    def ion_to_mzml(self, file: SampleFile) -> str:
        return ion_to_mzml(file)

    def mzml_to_ion(self, file: SampleFile, level: int = 12, f32_compress: bool = False) -> bytes:
        return mzml_to_ion(file, level, f32_compress)

    def mzml_to_ion_file(self, input_path: str, output_path: str, level: int = 12, f32_compress: bool = False) -> None:
        return mzml_to_ion_file(input_path, output_path, level, f32_compress)

    def calculate_eic(self, file: SampleFile, target_mz: float, from_to: FromTo, ppm_tol: float = 20.0, mz_tol: float = 0.005):
        return calculate_eic(file, target_mz, from_to, ppm_tol, mz_tol)

    def get_scans(self, file: SampleFile, *, rt=None, mz=None, level: int = 1):
        return get_scans(file, rt=rt, mz=mz, level=level)

    def find_peaks(self, x, y, options=None):
        return find_peaks(x, y, options)

    def get_peak(self, x, y, rt: float, range: float, options=None):
        return get_peak(x, y, rt, range, options)

    def fit_peak(self, x, y, rt: float, intensity: float, shape: str = "emg"):
        return fit_peak(x, y, rt, intensity, shape)

    def draw_peak(self, x, params):
        return draw_peak(x, params)

    def find_noise_level(self, y) -> dict:
        return find_noise_level(y)

    def calculate_baseline(self, y, lambda_: int = 0, max_iterations: int = 0):
        return calculate_baseline(y, lambda_, max_iterations)

    def get_peaks_from_eic(self, file: SampleFile, targets, from_to: FromTo = {"from": 0.5, "to": 5.0}, options=None, threads: int = 1):
        return get_peaks_from_eic(file, targets, from_to, options, threads)

    def get_peaks_from_chrom(self, file: SampleFile, targets, options=None, threads: int = 1):
        return get_peaks_from_chrom(file, targets, options, threads)

    def find_features(self, file: SampleFile, from_to: FromTo, *, eic=None, grid=None, options=None, threads: int = 1):
        return find_features(file, from_to, eic=eic, grid=grid, options=options, threads=threads)

    def find_feature(self, file: SampleFile, targets, *, scan_eic=None, eic=None, options=None, threads: int = 1):
        return find_feature(file, targets, scan_eic=scan_eic, eic=eic, options=options, threads=threads)

    def get_features(self, directory_path: str, from_to: FromTo, *, eic=None, grid=None, grouping=None, options=None, threads: int = 1):
        return get_features(directory_path, from_to, eic=eic, grid=grid, grouping=grouping, options=options, threads=threads)

    def load_model(self, source) -> Model:
        return load_model(source)

    def load_warp(self, model: Model) -> WarpModel:
        return load_warp(model)

    def __repr__(self) -> str:
        return "<Quantion ready>"


def __getattr__(name):
    if name == "MsUtils":
        import warnings
        warnings.warn(
            "MsUtils is renamed to Quantion and will be removed in the next breaking release",
            DeprecationWarning,
            stacklevel=2,
        )
        return Quantion
    raise AttributeError(name)


_, _abi = load_library()
_api._set_abi(_abi)