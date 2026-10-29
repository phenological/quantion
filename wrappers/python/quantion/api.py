from __future__ import annotations

import ctypes
import math
import os
from ctypes import POINTER, c_double, c_float, c_int32, c_size_t, c_uint8, c_uint32
from typing import Any, Callable, Dict, List, Mapping, Optional, Sequence, Tuple

import numpy as np

from quantion._bridge import (
    PeakOptions,
    READ_RANGE,
    _ABI,
    _Buf,
    buf_to_bytes,
    buf_to_f64,
    buf_to_image,
    buf_to_records,
    buf_to_scans,
)
from quantion._errors import QuantionError, _check, _invalid, _missing, _unknown_option
from quantion._options import (
    BASELINE,
    DRAW_PEAKS,
    EIC,
    FIND_FEATURES,
    FIND_PEAKS,
    FIND_TARGETS,
    FIND_TRANSITIONS,
    FIT_PEAK,
    GET_PEAK,
    GET_SCANS,
    ION_IMAGE,
    MATCH_FEATURES,
    PARSE_ION,
    WRITE_ION,
    WRITE_MZML,
    get_count,
    get_flag,
    get_number,
    get_shape,
    get_time_unit,
    pack_peak_options,
    read_options,
)
from quantion._types import (
    CalculateBaselineOptions,
    DataXY,
    DrawPeaksOptions,
    CalculateEicOptions,
    Feature,
    FindFeaturesOptions,
    FindPeaksOptions,
    FindTargetsOptions,
    FindTransitionsOptions,
    FitPeakOptions,
    FromTo,
    GetPeakOptions,
    GetScansOptions,
    IonImage,
    GetIonImageOptions,
    MatchFeaturesOptions,
    NoiseLevel,
    ParseIonOptions,
    Peak,
    PeakParameters,
    PredictedRt,
    Scan,
    Target,
    Transition,
    WriteIonOptions,
    WriteMzmlOptions,
)
from quantion.file import SampleFile
from quantion.model import Model
from quantion.warp_model import WarpModel

_abi: Optional[_ABI] = None

_QUERY_RT_RANGE = 0
_QUERY_CLOSEST_RT = 1
_QUERY_MZ_RANGE = 2
_QUERY_CLOSEST_MZ = 3
_IMAGE_SCANS_PER_STEP = 256
_MAX_LEVEL = 255


def _get_abi() -> _ABI:
    if _abi is None:
        raise QuantionError("internal", "quantion", "the native library is not loaded")
    return _abi


def _set_abi(abi: _ABI) -> None:
    global _abi
    _abi = abi


def _as_f64_ptr(array: np.ndarray):
    return array.ctypes.data_as(POINTER(c_double))


def _as_u32_ptr(array: np.ndarray):
    return array.ctypes.data_as(POINTER(c_uint32))


def _as_u8_ptr(data: bytes):
    array = (c_uint8 * len(data)).from_buffer_copy(data)
    return ctypes.cast(array, POINTER(c_uint8)), array


def _get_numbers(function: str, argument: str, values: Sequence) -> np.ndarray:
    try:
        return np.ascontiguousarray(values, dtype=np.float64).reshape(-1)
    except (TypeError, ValueError):
        raise _invalid(function, argument, "must be a sequence of numbers") from None


def _get_level(function: str, argument: str, value: Any) -> int:
    level = get_count(function, argument, value)
    if level > _MAX_LEVEL:
        raise _invalid(function, argument, f"must be between 0 and {_MAX_LEVEL}, got {value!r}")
    return level


def _read_record(
    function: str,
    argument: str,
    record: Any,
    required: Sequence[str],
    optional: Sequence[str] = (),
) -> Dict[str, Any]:
    if not isinstance(record, Mapping):
        raise _invalid(function, argument, "must be a dict")
    for key in record:
        if key not in required and key not in optional:
            raise _unknown_option(function, f"{argument}.{key}")
    for key in required:
        if record.get(key) is None:
            raise _missing(function, f"{argument}.{key}")
    return dict(record)


def _encode_ids(ids: Sequence[str]) -> Tuple[np.ndarray, np.ndarray, bytes]:
    offsets = np.zeros(len(ids), dtype=np.uint32)
    lengths = np.zeros(len(ids), dtype=np.uint32)
    parts: List[bytes] = []
    cursor = 0
    for index, text in enumerate(ids):
        encoded = (text or "").encode("utf-8")
        offsets[index] = cursor
        lengths[index] = len(encoded)
        parts.append(encoded)
        cursor += len(encoded)
    return offsets, lengths, b"".join(parts)


def _read_ids(function: str, argument: str, records: Sequence[Mapping[str, Any]]):
    ids = []
    for index, record in enumerate(records):
        value = record.get("id")
        if value is not None and not isinstance(value, str):
            raise _invalid(function, f"{argument}[{index}].id", "must be text")
        ids.append(value or "")
    offsets, lengths, text = _encode_ids(ids)
    if not text:
        return None, None, None, 0, None
    text_pointer, text_array = _as_u8_ptr(text)
    return _as_u32_ptr(offsets), _as_u32_ptr(lengths), text_pointer, len(text), (offsets, lengths, text_array)


def _get_eic(function: str, eic: Mapping[str, Any], prefix: str = "") -> Tuple[float, float, float, float, int]:
    return (
        get_number(function, f"{prefix}from", eic["from"]),
        get_number(function, f"{prefix}to", eic["to"]),
        get_number(function, f"{prefix}ppm_tolerance", eic["ppm_tolerance"]),
        get_number(function, f"{prefix}mz_tolerance", eic["mz_tolerance"]),
        get_time_unit(function, f"{prefix}time_unit", eic["time_unit"]),
    )


def _get_grid(function: str, grid: Mapping[str, Any]) -> Tuple[float, float, float]:
    return (
        get_number(function, "grid.start", grid["start"]),
        get_number(function, "grid.end", grid["end"]),
        get_number(function, "grid.step_size", grid["step_size"]),
    )


def _unpack_ranges(packed: bytes) -> List[Tuple[int, int]]:
    return [
        (int.from_bytes(packed[index:index + 8], "little"),
         int.from_bytes(packed[index + 8:index + 16], "little"))
        for index in range(0, len(packed), 16)
    ]


def _prefetch(file: SampleFile, function: str, plan: Callable[[Any], int]) -> None:
    source = getattr(file, "_remote_source", None)
    if source is None:
        return
    buffer = _Buf()
    _check(file._abi, function, plan(ctypes.byref(buffer)))
    source.prefetch(_unpack_ranges(buf_to_bytes(file._abi, buffer)))


def _is_url(text: str) -> bool:
    return text.lower().startswith(("http://", "https://"))


def _open_file(function: str, abi: _ABI, call: Callable[[Any], int], is_ion: bool) -> SampleFile:
    handle = ctypes.c_void_p()
    _check(abi, function, call(ctypes.byref(handle)))
    file = SampleFile(handle, abi)
    file._is_ion = is_ion
    return file


def parse_ion(source, options: Optional[ParseIonOptions] = None) -> SampleFile:
    """Open an .ion file from a path, an http(s) URL or bytes. Paths and URLs are read lazily."""
    function = "parse_ion"
    settings = read_options(function, options, PARSE_ION)
    cache_size = get_count(function, "max_cache_size", settings["max_cache_size"])
    abi = _get_abi()
    if isinstance(source, (str, os.PathLike)):
        text = os.fspath(source)
        if _is_url(text):
            return _parse_ion_remote(text, cache_size)
        return _open_file(function, abi, lambda out: abi.parse_ion_path(
            os.fsencode(text), c_size_t(cache_size), out), True)
    if isinstance(source, (bytes, bytearray, memoryview)):
        pointer, array = _as_u8_ptr(bytes(source))
        return _open_file(function, abi, lambda out: abi.parse_bin(
            pointer, c_size_t(len(array)), c_size_t(cache_size), out), True)
    raise _invalid(function, "source", "must be a path, an http(s) URL or bytes")


def parse_mzml(source) -> SampleFile:
    """Open an mzML file from a path or bytes. It is loaded fully into memory; URLs are refused."""
    function = "parse_mzml"
    abi = _get_abi()
    if isinstance(source, (str, os.PathLike)):
        text = os.fsencode(os.fspath(source))
        return _open_file(function, abi, lambda out: abi.parse_mzml_path(text, out), False)
    if isinstance(source, (bytes, bytearray, memoryview)):
        pointer, array = _as_u8_ptr(bytes(source))
        return _open_file(function, abi, lambda out: abi.parse_mzml(pointer, len(array), out), False)
    raise _invalid(function, "source", "must be a path or bytes")


def _encode_out(function: str, out: Any) -> Optional[bytes]:
    if out is None:
        return None
    if not isinstance(out, (str, os.PathLike)):
        raise _invalid(function, "out", "must be a path or a URL")
    return os.fsencode(os.fspath(out))


def _encode_headers(function: str, headers: Any) -> bytes:
    if not isinstance(headers, (list, tuple)):
        raise _invalid(function, "headers", "must be a list of {\"name\", \"value\"}")
    parts = []
    for index, header in enumerate(headers):
        record = _read_record(function, f"headers[{index}]", header, ("name", "value"))
        for key in ("name", "value"):
            text = record[key]
            if not isinstance(text, str):
                raise _invalid(function, f"headers[{index}].{key}", "must be text")
            if "\0" in text:
                raise _invalid(function, f"headers[{index}].{key}", "must not contain a NUL character")
            parts.append(text.encode("utf-8") + b"\0")
    return b"".join(parts)


def write_ion(file: SampleFile, options: Optional[WriteIonOptions] = None) -> Optional[bytes]:
    """Write a sample as .ion: bytes when out is not given, a file for a path, an upload for a URL."""
    function = "write_ion"
    settings = read_options(function, options, WRITE_ION)
    out = _encode_out(function, settings["out"])
    level = _get_level(function, "level", settings["level"])
    headers = _encode_headers(function, settings["headers"])
    headers_pointer, headers_array = _as_u8_ptr(headers) if headers else (None, None)
    abi = file._abi
    buffer = _Buf()
    _check(abi, function, abi.write_ion(
        file.handle, out,
        c_uint8(get_flag(function, "overwrite", settings["overwrite"])),
        c_uint8(level),
        c_uint8(get_flag(function, "f32_compress", settings["f32_compress"])),
        headers_pointer, c_size_t(len(headers)), ctypes.byref(buffer),
    ))
    if out is not None and _is_url(os.fsdecode(out)):
        _upload_ion(os.fsdecode(out), settings["headers"], buf_to_bytes(abi, buffer))
        return None
    return buf_to_bytes(abi, buffer) if out is None else None


def _upload_ion(url: str, headers: Sequence[Mapping[str, str]], data: bytes) -> None:
    from urllib.error import HTTPError, URLError
    from urllib.request import Request, urlopen

    fields = {header["name"].strip(): header["value"] for header in headers}
    request = Request(url, data=data, method="PUT", headers=fields)
    try:
        with urlopen(request):
            pass
    except HTTPError as error:
        raise QuantionError("upload_failed", "write_ion", f"PUT returned {error.code}") from None
    except URLError as error:
        raise QuantionError("upload_failed", "write_ion", f"PUT failed: {error.reason}") from None


def write_mzml(file: SampleFile, options: Optional[WriteMzmlOptions] = None) -> Optional[bytes]:
    """Write a sample as mzML: bytes when out is not given, otherwise a file at the path."""
    function = "write_mzml"
    settings = read_options(function, options, WRITE_MZML)
    out = _encode_out(function, settings["out"])
    abi = file._abi
    buffer = _Buf()
    _check(abi, function, abi.write_mzml(
        file.handle, out, c_uint8(get_flag(function, "overwrite", settings["overwrite"])), ctypes.byref(buffer)))
    return buf_to_bytes(abi, buffer) if out is None else None


def _get_scan_query(function: str, name: str, value: Any, range_query: int, closest_query: int):
    if isinstance(value, Mapping):
        window = _read_record(function, name, value, ("from", "to"))
        return (range_query,
                get_number(function, f"{name}.from", window["from"]),
                get_number(function, f"{name}.to", window["to"]))
    return closest_query, get_number(function, name, value), math.nan


def get_scans(
    file: SampleFile,
    *,
    rt: "float | FromTo | None" = None,
    mz: "float | FromTo | None" = None,
    options: Optional[GetScansOptions] = None,
) -> List[Scan]:
    """Read scans by retention time or precursor m/z. A range gives every scan in it; one number gives the closest scan."""
    function = "get_scans"
    settings = read_options(function, options, GET_SCANS)
    if rt is None and mz is None:
        raise _missing(function, "rt")
    if rt is not None and mz is not None:
        raise _invalid(function, "mz", "cannot be given together with rt")
    if rt is not None:
        query, start, end = _get_scan_query(function, "rt", rt, _QUERY_RT_RANGE, _QUERY_CLOSEST_RT)
    else:
        query, start, end = _get_scan_query(function, "mz", mz, _QUERY_MZ_RANGE, _QUERY_CLOSEST_MZ)
    level = _get_level(function, "level", settings["level"])
    time_unit = get_time_unit(function, "time_unit", settings["time_unit"])
    abi = file._abi
    _prefetch(file, function, lambda out: abi.plan_scans(
        file.handle, c_uint8(query), c_double(start), c_double(end), c_uint8(level), c_uint8(time_unit), out))
    buffer = _Buf()
    _check(abi, function, abi.get_scans(
        file.handle, c_uint8(query), c_double(start), c_double(end), c_uint8(level), c_uint8(time_unit),
        ctypes.byref(buffer)))
    return buf_to_scans(abi, buffer)


def calculate_eic(file: SampleFile, mz: float, options: Optional[CalculateEicOptions] = None) -> DataXY:
    """Extract the ion chromatogram of one m/z between from and to (the whole run by default)."""
    function = "calculate_eic"
    settings = read_options(function, options, EIC)
    start, end, ppm_tolerance, mz_tolerance, time_unit = _get_eic(function, settings)
    mz = get_number(function, "mz", mz)
    abi = file._abi
    _prefetch(file, function, lambda out: abi.plan_eic(
        file.handle, c_double(mz), c_double(start), c_double(end),
        c_double(ppm_tolerance), c_double(mz_tolerance), c_uint8(time_unit), out))
    x_buffer, y_buffer = _Buf(), _Buf()
    _check(abi, function, abi.calculate_eic(
        file.handle, c_double(mz), c_double(start), c_double(end),
        c_double(ppm_tolerance), c_double(mz_tolerance), c_uint8(time_unit),
        ctypes.byref(x_buffer), ctypes.byref(y_buffer)))
    return {"x": buf_to_f64(abi, x_buffer), "y": buf_to_f64(abi, y_buffer)}


def get_ion_image(file: SampleFile, mz: float, options: Optional[GetIonImageOptions] = None) -> IonImage:
    """Build the imaging map of one m/z. Only .ion files from parse_ion."""
    function = "get_ion_image"
    settings = read_options(function, options, ION_IMAGE)
    mz = get_number(function, "mz", mz)
    tolerance = get_number(function, "tolerance", settings["tolerance"])
    level = _get_level(function, "level", settings["level"])
    on_progress = settings["on_progress"]
    if on_progress is not None and not callable(on_progress):
        raise _invalid(function, "on_progress", "must be a function")
    abi = file._abi
    if getattr(file, "_is_ion", False):
        _prefetch(file, function, lambda out: abi.plan_image(
            file.handle, c_double(mz), c_double(tolerance), c_uint8(level), out))
    if on_progress is None or not getattr(file, "_is_ion", False):
        buffer = _Buf()
        _check(abi, function, abi.get_ion_image(
            file.handle, c_double(mz), c_double(tolerance), c_uint8(level), ctypes.byref(buffer)))
        return buf_to_image(abi, buffer)
    return _get_ion_image_in_steps(file, mz, tolerance, level, on_progress)


def _get_ion_image_in_steps(file: SampleFile, mz: float, tolerance: float, level: int, on_progress) -> IonImage:
    function = "get_ion_image"
    abi = file._abi
    session = c_size_t(0)
    _check(abi, function, abi.image_begin(
        file.handle, c_double(mz), c_double(tolerance), c_uint8(level), ctypes.byref(session)))
    pointer = ctypes.c_void_p(session.value)
    try:
        total = c_size_t(0)
        _check(abi, function, abi.image_scan_count(pointer, ctypes.byref(total)))
        done = 0
        while done < total.value:
            count = min(_IMAGE_SCANS_PER_STEP, total.value - done)
            _check(abi, function, abi.image_fold(file.handle, pointer, c_size_t(done), c_size_t(count)))
            done += count
            on_progress(done, total.value, 0)
        buffer = _Buf()
        _check(abi, function, abi.image_finish(pointer, ctypes.byref(buffer)))
        return buf_to_image(abi, buffer)
    finally:
        abi.image_free(pointer)


def _get_trace(function: str, x: Sequence, y: Sequence) -> Tuple[np.ndarray, np.ndarray]:
    return _get_numbers(function, "x", x), _get_numbers(function, "y", y)


def find_peaks(x: Sequence, y: Sequence, options: Optional[FindPeaksOptions] = None) -> List[Peak]:
    """Find every peak in one trace."""
    function = "find_peaks"
    settings = read_options(function, options, FIND_PEAKS)
    packed = pack_peak_options(function, settings, "")
    x_values, y_values = _get_trace(function, x, y)
    _check_lengths(function, x_values, y_values)
    abi = _get_abi()
    buffer = _Buf()
    _check(abi, function, abi.find_peaks(
        _as_f64_ptr(x_values), _as_f64_ptr(y_values), c_size_t(len(x_values)),
        ctypes.byref(packed), ctypes.byref(buffer)))
    return buf_to_records(abi, buffer)


def _check_lengths(function: str, x_values: np.ndarray, y_values: np.ndarray) -> None:
    if len(x_values) != len(y_values):
        raise _invalid(function, "y", f"must have the same length as x ({len(x_values)}), got {len(y_values)}")


def get_peak(x: Sequence, y: Sequence, rt: float, options: Optional[GetPeakOptions] = None) -> Peak:
    """Find the one peak near rt. A range of 0 (the default) searches the whole trace."""
    function = "get_peak"
    settings = read_options(function, options, GET_PEAK)
    packed = pack_peak_options(function, settings["find_peaks"])
    x_values, y_values = _get_trace(function, x, y)
    _check_lengths(function, x_values, y_values)
    abi = _get_abi()
    buffer = _Buf()
    _check(abi, function, abi.get_peak(
        _as_f64_ptr(x_values), _as_f64_ptr(y_values), c_size_t(len(x_values)),
        c_double(get_number(function, "rt", rt)), c_double(get_number(function, "range", settings["range"])),
        ctypes.byref(packed), ctypes.byref(buffer)))
    return buf_to_records(abi, buffer)[0]


def fit_peak(x: Sequence, y: Sequence, rt: float, options: Optional[FitPeakOptions] = None) -> Optional[PeakParameters]:
    """Fit a Gaussian or EMG curve to the peak at rt. None when no peak is found or the fit fails."""
    function = "fit_peak"
    settings = read_options(function, options, FIT_PEAK)
    x_values, y_values = _get_trace(function, x, y)
    _check_lengths(function, x_values, y_values)
    abi = _get_abi()
    buffer = _Buf()
    _check(abi, function, abi.fit_peak(
        _as_f64_ptr(x_values), _as_f64_ptr(y_values), c_size_t(len(x_values)),
        c_double(get_number(function, "rt", rt)),
        c_int32(get_shape(function, "shape", settings["shape"])),
        c_double(get_number(function, "intensity", settings["intensity"])),
        ctypes.byref(buffer)))
    rows = buf_to_records(abi, buffer)
    return rows[0] if rows else None


def draw_peaks(params: Sequence[PeakParameters], options: Optional[DrawPeaksOptions] = None) -> DataXY:
    """Draw peak curves into one chromatogram; where they overlap, intensities add up."""
    function = "draw_peaks"
    settings = read_options(function, options, DRAW_PEAKS)
    if not isinstance(params, (list, tuple)):
        raise _invalid(function, "params", "must be a list of peak parameters")
    columns = {key: np.zeros(len(params), dtype=np.float64) for key in ("height", "center", "fwhm", "tail")}
    for index, peak in enumerate(params):
        record = _read_record(function, f"params[{index}]", peak, ("height", "center", "fwhm", "tail"), ("r2",))
        for key, column in columns.items():
            column[index] = get_number(function, f"params[{index}].{key}", record[key])
    abi = _get_abi()
    x_buffer, y_buffer = _Buf(), _Buf()
    _check(abi, function, abi.draw_peaks(
        _as_f64_ptr(columns["height"]), _as_f64_ptr(columns["center"]),
        _as_f64_ptr(columns["fwhm"]), _as_f64_ptr(columns["tail"]), c_size_t(len(params)),
        c_double(get_number(function, "from", settings["from"])),
        c_double(get_number(function, "to", settings["to"])),
        c_size_t(get_count(function, "points", settings["points"])),
        ctypes.byref(x_buffer), ctypes.byref(y_buffer)))
    return {"x": buf_to_f64(abi, x_buffer), "y": buf_to_f64(abi, y_buffer)}


def find_noise_level(y: Sequence) -> NoiseLevel:
    """Estimate the noise level of a signal."""
    function = "find_noise_level"
    values = np.ascontiguousarray(_get_numbers(function, "y", y), dtype=np.float32)
    abi = _get_abi()
    width, intensity = c_size_t(0), c_double(0.0)
    _check(abi, function, abi.find_noise_level(
        values.ctypes.data_as(POINTER(c_float)), c_size_t(len(values)), ctypes.byref(width), ctypes.byref(intensity)))
    return {"width": int(width.value), "intensity": float(intensity.value)}


def calculate_baseline(y: Sequence, options: Optional[CalculateBaselineOptions] = None) -> np.ndarray:
    """Estimate a signal's baseline with airPLS."""
    function = "calculate_baseline"
    settings = read_options(function, options, BASELINE)
    values = _get_numbers(function, "y", y)
    abi = _get_abi()
    buffer = _Buf()
    _check(abi, function, abi.calculate_baseline(
        _as_f64_ptr(values), c_size_t(len(values)),
        c_double(get_number(function, "lambda", settings["lambda"])),
        c_size_t(get_count(function, "max_iterations", settings["max_iterations"])),
        ctypes.byref(buffer)))
    return buf_to_f64(abi, buffer)


def _read_targets(function: str, targets: Sequence[Target]):
    if not isinstance(targets, (list, tuple)):
        raise _invalid(function, "targets", "must be a list of targets")
    records = [_read_record(function, f"targets[{index}]", target, ("mz", "rt"), ("id", "range"))
               for index, target in enumerate(targets)]
    mzs = np.array([get_number(function, f"targets[{index}].mz", record["mz"])
                    for index, record in enumerate(records)], dtype=np.float64)
    rts = np.array([get_number(function, f"targets[{index}].rt", record["rt"])
                    for index, record in enumerate(records)], dtype=np.float64)
    ranges = np.array([math.nan if record.get("range") is None
                       else get_number(function, f"targets[{index}].range", record["range"])
                       for index, record in enumerate(records)], dtype=np.float64)
    return records, mzs, rts, ranges


def find_targets(file: SampleFile, targets: Sequence[Target], options: Optional[FindTargetsOptions] = None) -> List[Peak]:
    """Find each target's peak in its EIC. One Peak per target, in order; no peak gives every field 0."""
    function = "find_targets"
    settings = read_options(function, options, FIND_TARGETS)
    start, end, ppm_tolerance, mz_tolerance, time_unit = _get_eic(function, settings["eic"], "eic.")
    packed = pack_peak_options(function, settings["find_peaks"])
    threads = get_count(function, "threads", settings["threads"])
    records, mzs, rts, ranges = _read_targets(function, targets)
    offsets, lengths, text, text_length, keep = _read_ids(function, "targets", records)
    abi = file._abi
    _prefetch(file, function, lambda out: abi.plan_find_targets(
        file.handle, _as_f64_ptr(mzs), c_size_t(len(mzs)), c_double(start), c_double(end),
        c_double(ppm_tolerance), c_double(mz_tolerance), c_uint8(time_unit), out))
    buffer = _Buf()
    _check(abi, function, abi.find_targets(
        file.handle, _as_f64_ptr(rts), _as_f64_ptr(mzs), _as_f64_ptr(ranges),
        offsets, lengths, text, c_size_t(text_length), c_size_t(len(records)),
        c_double(start), c_double(end), c_double(ppm_tolerance), c_double(mz_tolerance), c_uint8(time_unit),
        ctypes.byref(packed), c_size_t(threads), ctypes.byref(buffer)))
    return buf_to_records(abi, buffer)


def find_transitions(
    file: SampleFile, targets: Sequence[Transition], options: Optional[FindTransitionsOptions] = None
) -> List[Peak]:
    """Find each SRM transition's peak in its stored chromatogram. Needs the whole file."""
    function = "find_transitions"
    settings = read_options(function, options, FIND_TRANSITIONS)
    packed = pack_peak_options(function, settings["find_peaks"])
    threads = get_count(function, "threads", settings["threads"])
    if not isinstance(targets, (list, tuple)):
        raise _invalid(function, "targets", "must be a list of transitions")
    keys = ("precursor", "product", "rt", "range")
    records = [_read_record(function, f"targets[{index}]", target, keys, ("id",))
               for index, target in enumerate(targets)]
    columns = {key: np.array([get_number(function, f"targets[{index}].{key}", record[key])
                              for index, record in enumerate(records)], dtype=np.float64) for key in keys}
    offsets, lengths, text, text_length, keep = _read_ids(function, "targets", records)
    abi = file._abi
    buffer = _Buf()
    _check(abi, function, abi.find_transitions(
        file.handle, _as_f64_ptr(columns["precursor"]), _as_f64_ptr(columns["product"]),
        _as_f64_ptr(columns["rt"]), _as_f64_ptr(columns["range"]),
        offsets, lengths, text, c_size_t(text_length), c_size_t(len(records)),
        ctypes.byref(packed), c_size_t(threads), ctypes.byref(buffer)))
    return buf_to_records(abi, buffer)


def find_features(file: SampleFile, options: Optional[FindFeaturesOptions] = None) -> List[Feature]:
    """Find every feature in one file by scanning an m/z grid."""
    function = "find_features"
    settings = read_options(function, options, FIND_FEATURES)
    start, end, ppm_tolerance, mz_tolerance, time_unit = _get_eic(function, settings["eic"], "eic.")
    grid_start, grid_end, grid_step_size = _get_grid(function, settings["grid"])
    packed = pack_peak_options(function, settings["find_peaks"])
    threads = get_count(function, "threads", settings["threads"])
    abi = file._abi
    _prefetch(file, function, lambda out: abi.plan_find_features(
        file.handle, c_double(start), c_double(end), c_double(ppm_tolerance), c_double(mz_tolerance),
        c_uint8(time_unit), c_double(grid_start), c_double(grid_end), c_double(grid_step_size), out))
    buffer = _Buf()
    _check(abi, function, abi.find_features(
        file.handle, c_double(start), c_double(end), c_double(ppm_tolerance), c_double(mz_tolerance),
        c_uint8(time_unit), c_double(grid_start), c_double(grid_end), c_double(grid_step_size),
        ctypes.byref(packed), c_size_t(threads), ctypes.byref(buffer)))
    return buf_to_records(abi, buffer)


def match_features(directory, options: Optional[MatchFeaturesOptions] = None) -> List[Feature]:
    """Find features in every file of a folder and match them across samples."""
    function = "match_features"
    settings = read_options(function, options, MATCH_FEATURES)
    if not isinstance(directory, (str, os.PathLike)):
        raise _invalid(function, "directory", "must be a path")
    start, end, ppm_tolerance, mz_tolerance, time_unit = _get_eic(function, settings["eic"], "eic.")
    grid_start, grid_end, grid_step_size = _get_grid(function, settings["grid"])
    grouping = settings["grouping"]
    packed = pack_peak_options(function, settings["find_peaks"])
    threads = get_count(function, "threads", settings["threads"])
    abi = _get_abi()
    buffer = _Buf()
    _check(abi, function, abi.match_features(
        os.fsencode(os.fspath(directory)),
        c_double(start), c_double(end), c_double(ppm_tolerance), c_double(mz_tolerance), c_uint8(time_unit),
        c_double(grid_start), c_double(grid_end), c_double(grid_step_size),
        c_double(get_number(function, "grouping.ppm_tolerance", grouping["ppm_tolerance"])),
        c_double(get_number(function, "grouping.mz_tolerance", grouping["mz_tolerance"])),
        c_double(get_number(function, "grouping.rt_tolerance", grouping["rt_tolerance"])),
        c_double(get_number(function, "grouping.min_frequency", grouping["min_frequency"])),
        ctypes.byref(packed), c_size_t(threads), ctypes.byref(buffer)))
    return buf_to_records(abi, buffer)


def load_model(source) -> Model:
    """Load an ONNX model from a path, an http(s) URL or bytes."""
    function = "load_model"
    abi = _get_abi()
    if isinstance(source, (str, os.PathLike)):
        text = os.fspath(source)
        if not _is_url(text):
            path = os.fsencode(text)
            path_pointer, path_array = _as_u8_ptr(path)
            return _open_model(abi, lambda handle, message: abi.model_load_path(
                path_pointer, len(path), handle, message))
        from urllib.request import urlopen

        try:
            with urlopen(text) as response:
                source = response.read()
        except OSError as error:
            raise QuantionError("parse_failed", function,
                                f"the file could not be read: GET {text} failed: {error}", "source") from error
    if not isinstance(source, (bytes, bytearray, memoryview)):
        raise _invalid(function, "source", "must be a path, an http(s) URL or bytes")
    data = bytes(source)
    data_pointer, data_array = _as_u8_ptr(data)
    return _open_model(abi, lambda handle, message: abi.model_load_bytes(data_pointer, len(data), handle, message))


def _open_model(abi: _ABI, call) -> Model:
    handle, message = ctypes.c_void_p(), _Buf()
    code = call(ctypes.byref(handle), ctypes.byref(message))
    if message.ptr:
        buf_to_bytes(abi, message)
    _check(abi, "load_model", code)
    return Model(handle, abi)


def load_warp(model: Model) -> WarpModel:
    """Make a retention-time warp model from a loaded model. The model stays usable."""
    function = "load_warp"
    if not isinstance(model, Model):
        raise _invalid(function, "model", "must be a Model from load_model")
    abi = _get_abi()
    handle, message = ctypes.c_void_p(), _Buf()
    code = abi.warp_model_load(model.handle, ctypes.byref(handle), ctypes.byref(message))
    if message.ptr:
        buf_to_bytes(abi, message)
    _check(abi, function, code)
    return WarpModel(handle, abi, model._get_metadata())


def _get_model_metadata(model: Model) -> List[Dict[str, str]]:
    abi = _get_abi()
    buffer = _Buf()
    _check(abi, "load_model", abi.model_metadata(model.handle, ctypes.byref(buffer)))
    return buf_to_records(abi, buffer)


def _read_warp_targets(function: str, argument: str, targets: Sequence[Target]):
    if not isinstance(targets, (list, tuple)):
        raise _invalid(function, argument, "must be a list of targets")
    records = [_read_record(function, f"{argument}[{index}]", target, ("mz", "rt"), ("id", "range"))
               for index, target in enumerate(targets)]
    mzs = np.array([get_number(function, f"{argument}[{index}].mz", record["mz"])
                    for index, record in enumerate(records)], dtype=np.float64)
    rts = np.array([get_number(function, f"{argument}[{index}].rt", record["rt"])
                    for index, record in enumerate(records)], dtype=np.float64)
    return records, mzs, rts


def _prefetch_warp(warp: WarpModel, file: SampleFile, function: str, mzs: np.ndarray, rts: np.ndarray) -> None:
    if len(mzs) == 0:
        return
    abi = warp._abi
    _prefetch(file, function, lambda out: abi.plan_predict_rts(
        warp.handle, file.handle, _as_f64_ptr(mzs), _as_f64_ptr(rts), c_size_t(len(mzs)), out))


def _predict_rts(warp: WarpModel, file: SampleFile, targets: Sequence[Target]) -> List[PredictedRt]:
    function = "predict_rts"
    records, mzs, rts = _read_warp_targets(function, "targets", targets)
    if not records:
        return []
    _prefetch_warp(warp, file, function, mzs, rts)
    abi = warp._abi
    buffer = _Buf()
    _check(abi, function, abi.predict_rts_from_file(
        warp.handle, file.handle, _as_f64_ptr(mzs), _as_f64_ptr(rts), c_size_t(len(mzs)), ctypes.byref(buffer)))
    values = buf_to_f64(abi, buffer).reshape(-1, 2)
    return [{"rt": float(rt), "uncertainty": float(uncertainty)} for rt, uncertainty in values]


def _calibrate_rts(
    warp: WarpModel,
    anchor_samples: Sequence[SampleFile],
    target_lists: Sequence[Sequence[Target]],
) -> List[Target]:
    function = "calibrate"
    lists = [_read_warp_targets(function, f"target_lists[{index}]", targets)
             for index, targets in enumerate(target_lists)]
    for sample, (records, mzs, rts) in zip(anchor_samples, lists):
        _prefetch_warp(warp, sample, function, mzs, rts)
    records = [record for records, _, _ in lists for record in records]
    mzs = np.concatenate([mzs for _, mzs, _ in lists]) if lists else np.zeros(0)
    rts = np.concatenate([rts for _, _, rts in lists]) if lists else np.zeros(0)
    offsets, lengths, text, text_length, keep = _read_ids(function, "target_lists", records)
    files = (ctypes.c_void_p * len(anchor_samples))(*[sample.handle for sample in anchor_samples])
    list_lengths = (c_size_t * len(lists))(*[len(records) for records, _, _ in lists])
    abi = warp._abi
    buffer = _Buf()
    _check(abi, function, abi.calibrate_rts(
        warp.handle, files, c_size_t(len(anchor_samples)), list_lengths, c_size_t(len(lists)),
        offsets, lengths, text, c_size_t(text_length),
        _as_f64_ptr(mzs), _as_f64_ptr(rts), c_size_t(len(records)), ctypes.byref(buffer)))
    return buf_to_records(abi, buffer)


LARGEST_GAP = 131072


def gap_for(total: int) -> int:
    return min(LARGEST_GAP, total // 8)


class _RemoteSource:
    HEADER_BYTES = 1024

    def __init__(self, url: str) -> None:
        self.url = url
        self.cache: dict[tuple[int, int], bytes] = {}
        self.bytes_fetched = 0
        self.total = 0
        header = self.fetch(0, self.HEADER_BYTES)
        self.cache[(0, len(header))] = header
        planned = _plan_open(header)
        self.total = max((offset + length for offset, length in planned), default=0)
        self.prefetch(planned)

    def fetch(self, offset: int, length: int) -> bytes:
        from urllib.request import Request, urlopen

        last = offset + length - 1
        request = Request(self.url, headers={"Range": f"bytes={offset}-{last}"})
        with urlopen(request) as response:
            payload = response.read()
        self.bytes_fetched += len(payload)
        return payload

    def read_from_cache(self, offset: int, length: int) -> Optional[bytes]:
        exact = self.cache.get((offset, length))
        if exact is not None:
            return exact
        for (start, size), payload in self.cache.items():
            if offset >= start and offset + length <= start + size:
                begin = offset - start
                return payload[begin:begin + length]
        return None

    def read(self, offset: int, length: int) -> bytes:
        held = self.read_from_cache(offset, length)
        if held is not None:
            return held
        payload = self.fetch(offset, length)
        self.cache[(offset, length)] = payload
        return payload

    def prefetch(self, ranges: list[tuple[int, int]]) -> None:
        missing = [
            (offset, length) for offset, length in ranges
            if length > 0 and self.read_from_cache(offset, length) is None
        ]
        for offset, length in _merge_ranges(missing, gap_for(self.total)):
            if self.read_from_cache(offset, length) is None:
                self.cache[(offset, length)] = self.fetch(offset, length)


def _merge_ranges(
    ranges: list[tuple[int, int]], gap: int = LARGEST_GAP
) -> list[tuple[int, int]]:
    wanted = sorted((offset, length) for offset, length in ranges if length)
    merged: list[list[int]] = []
    for offset, length in wanted:
        if merged and offset - (merged[-1][0] + merged[-1][1]) <= gap:
            end = max(merged[-1][0] + merged[-1][1], offset + length)
            merged[-1][1] = end - merged[-1][0]
        else:
            merged.append([offset, length])
    return [(offset, length) for offset, length in merged]


def _plan_open(header: bytes) -> list[tuple[int, int]]:
    abi = _get_abi()
    buffer = _Buf()
    array = (c_uint8 * len(header)).from_buffer_copy(header)
    _check(abi, "parse_ion", abi.plan_open(
        ctypes.cast(array, POINTER(c_uint8)),
        c_size_t(len(header)),
        ctypes.byref(buffer),
    ))
    packed = buf_to_bytes(abi, buffer)
    ranges = []
    for index in range(0, len(packed), 16):
        offset = int.from_bytes(packed[index:index + 8], "little")
        length = int.from_bytes(packed[index + 8:index + 16], "little")
        ranges.append((offset, length))
    return ranges




def _parse_ion_remote(url: str, max_cache_size: int) -> SampleFile:
    source = _RemoteSource(url)

    def serve(context, offset, length, dest):
        try:
            payload = source.read(int(offset), int(length))
        except Exception:
            return -1
        if len(payload) != length:
            return -1
        ctypes.memmove(dest, payload, length)
        return 0

    callback = READ_RANGE(serve)
    abi = _get_abi()
    file = _open_file("parse_ion", abi, lambda out: abi.parse_ion_source(
        callback, None, c_size_t(max_cache_size), out), True)
    file._range_callback = callback
    file._remote_source = source
    return file
