from __future__ import annotations

from enum import Enum
from typing import Callable, Dict, List, Optional, TypedDict, Union

import numpy as np


class TimeUnit(str, Enum):
    """TimeUnit · Enums

    Unit of the retention times you give to a function. Results are always in minutes.

    Fields:
        MINUTES: Minutes.
        SECONDS: Seconds.

    Documentation: https://phenological.github.io/quantion/?language=python#TimeUnit
    """
    MINUTES = "MINUTES"
    SECONDS = "SECONDS"

    def __str__(self) -> str:
        return self.value


class PeakShape(str, Enum):
    """PeakShape · Enums

    Curve used to fit or check a peak.

    Fields:
        GAUSSIAN: Symmetric bell curve.
        EMG: Exponentially modified Gaussian: a bell curve with a tail on the right.

    Documentation: https://phenological.github.io/quantion/?language=python#PeakShape
    """
    GAUSSIAN = "GAUSSIAN"
    EMG = "EMG"

    def __str__(self) -> str:
        return self.value


FromTo = TypedDict("FromTo", {"from": float, "to": float})
FromTo.__doc__ = """FromTo · Inputs

A range with a start and an end, used for retention-time windows and m/z ranges.

Fields:
    from: Start of the range.
    to: End of the range.

Example:
    window: FromTo = {"from": 1.0, "to": 2.5}

Documentation: https://phenological.github.io/quantion/?language=python#FromTo
"""


class _TargetFields(TypedDict):
    mz: float
    rt: float


class Target(_TargetFields, total=False):
    """Target · Inputs

    One compound to look for, with its m/z and expected retention time. range limits the search in find_targets, and a missing or 0 range searches the whole EIC. predict_rts and calibrate do not use range.

    Fields:
        id: Your name for the compound. It is not used in the search. Optional.
        mz: The m/z of the ion to look for (Da), not the neutral mass of the compound. For example, for the protonated molecule [M+H]+ in positive mode, use M + 1.00728, and for the deprotonated molecule [M−H]− in negative mode, use M − 1.00728, where M is the monoisotopic mass.
        rt: Expected retention time, in minutes by default (or in the time unit set in the EIC options).
        range: Half-width of the search window around rt, in the same unit as rt. Missing or 0 searches the whole EIC. Optional.

    Example:
        target: Target = {"id": "compound_1", "mz": 260.103, "rt": 2.383, "range": 0.05}

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

    One SRM or MRM transition, with its precursor (Q1) and product (Q3) m/z.

    Fields:
        id: Your name for the transition. Optional.
        precursor: Precursor (Q1) m/z.
        product: Product (Q3) m/z.
        rt: Expected retention time, in minutes.
        range: Half-width of the search window around rt, in minutes.

    Example:
        transition: Transition = {"id": "compound_1", "precursor": 89.04768, "product": 89.04768, "rt": 2.385, "range": 0.1}

    Documentation: https://phenological.github.io/quantion/?language=python#Transition
    """
    id: str


class Header(TypedDict):
    """Header · Inputs

    One HTTP header sent with an upload in write_ion, for example a token. Values are never printed. A line break, or the names Host, Content-Length or Transfer-Encoding, give an invalid_argument error.

    Fields:
        name: Name of the header, such as Authorization.
        value: Value of the header.

    Example:
        header: Header = {"name": "Authorization", "value": "Bearer <token>"}

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

Options for find_peaks. Every field is optional.

Fields:
    min_integral: Drop peaks whose area (integral) is below this value. Off when not given. Default off.
    min_intensity: Drop peaks whose apex intensity is below this value. Raise it to ignore small peaks. Default 500.
    min_peak_width_points: Drop peaks with fewer data points than this. Default 3.
    noise: Fixed noise level (intensity), used only when auto_noise is false. Default off.
    auto_noise: Estimate the noise from the chromatogram. Turn off and set noise to use a fixed value. Default True.
    auto_baseline: Subtract an airPLS baseline before finding peaks. Turn off for windows shorter than 1 minute: they do not have enough points to estimate the baseline and can give misleading results. Default True.
    lambda: Smoothness of the airPLS baseline. A larger value gives a smoother baseline that follows only slow drift. Default 20.
    max_iterations: Maximum number of airPLS iterations used to estimate the baseline. Default 4.
    allow_overlap: Let neighbouring peaks share points at their border. When off, overlapping peaks are split at the valley between them. Default False.
    min_snr: Minimum signal-to-noise ratio. A peak’s apex must be above min_snr × noise. Default 2.
    min_r2: Minimum fit quality (0–1) of the peak shape. Peaks that fit the shape worse are dropped. 0 keeps every peak. Default 0.
    shape: Curve used to check the peak shape for min_r2: EMG or GAUSSIAN. Default EMG.
    kernel_size: Savitzky–Golay smoothing window, in points, to smooth noisy chromatograms. 0 = no smoothing. A larger value gives a smoother chromatogram. Default 0.

Documentation: https://phenological.github.io/quantion/?language=python#FindPeaksOptions
"""


class GetPeakOptions(TypedDict, total=False):
    """GetPeakOptions · Options

    Options for get_peak, including the search window and the peak-finding settings.

    Fields:
        range: Half-width of the search window, in minutes. Only peaks within rt ± range are considered. 0 searches the whole trace. Default 0.
        find_peaks: Options for find_peaks function. See FindPeaksOptions: https://phenological.github.io/quantion/?language=python#FindPeaksOptions

    Documentation: https://phenological.github.io/quantion/?language=python#GetPeakOptions
    """
    range: float
    find_peaks: FindPeaksOptions


class FitPeakOptions(TypedDict, total=False):
    """FitPeakOptions · Options

    Options for fit_peak function.

    Fields:
        shape: Curve to fit: EMG (with a right tail) or GAUSSIAN (symmetric). Default EMG.
        intensity: Apex intensity to start the fit from. 0 = use the apex of the peak found at rt. Default 0.

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

Options for draw_peaks, including the time range and the number of points.

Fields:
    from: Start of the simulated chromatogram, in minutes. Leave from and to at 0 to cover all the peaks automatically. Default 0.
    to: End of the simulated chromatogram, in minutes. Default 0.
    points: Number of points drawn from from to to. Default 1000.

Documentation: https://phenological.github.io/quantion/?language=python#DrawPeaksOptions
"""


CalculateBaselineOptions = TypedDict("CalculateBaselineOptions", {
    "lambda": float,
    "max_iterations": int,
}, total=False)
CalculateBaselineOptions.__doc__ = """CalculateBaselineOptions · Options

Options for calculate_baseline, which estimates the baseline with airPLS.

Fields:
    lambda: Smoothness of the baseline. A larger value gives a smoother baseline that follows only slow drift. Default 20.
    max_iterations: Maximum number of iterations used to estimate the baseline. Default 4.

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

Options for calculate_eic, including the retention-time range to read and the m/z window. The same options are used by find_targets, find_features and match_features. The wider of the ppm and Da windows is used.

Fields:
    from: Start of the retention-time range to read. Default 0.
    to: End of the retention-time range to read. By default, the run is read to the end. Default inf.
    ppm_tolerance: Half-width of the m/z window, in ppm of the m/z. Default 20.
    mz_tolerance: Half-width of the m/z window, in Da. Default 0.005.
    time_unit: Unit of from and to, minutes or seconds. Default MINUTES.

Documentation: https://phenological.github.io/quantion/?language=python#CalculateEicOptions
"""


class GridOptions(TypedDict, total=False):
    """GridOptions · Options

    The m/z grid that find_features and match_features scan, from start to end in steps of step_size.

    Fields:
        start: Lowest m/z of the grid. Default 40.
        end: Highest m/z of the grid. Default 1000.
        step_size: Distance between two grid m/z values, in Da. A smaller step is finer but slower. Default 0.005.

    Documentation: https://phenological.github.io/quantion/?language=python#GridOptions
    """
    start: float
    end: float
    step_size: float


class GroupingOptions(TypedDict, total=False):
    """GroupingOptions · Options

    How match_features decides that features from different samples are the same feature.

    Fields:
        ppm_tolerance: Features are the same when their m/z is within this many ppm, or within mz_tolerance. Default 5.
        mz_tolerance: Features are the same when their m/z is within this many Da, or within ppm_tolerance. Default 0.0025.
        rt_tolerance: Features are the same only when their rt is also within this many minutes. Default 0.05.
        min_frequency: Keep a feature only if it is found in at least this fraction of samples, from 0 to 1. For example, 0.8 keeps features seen in 80% of the samples. Default 0.

    Documentation: https://phenological.github.io/quantion/?language=python#GroupingOptions
    """
    ppm_tolerance: float
    mz_tolerance: float
    rt_tolerance: float
    min_frequency: float


class FindTargetsOptions(TypedDict, total=False):
    """FindTargetsOptions · Options

    Options for find_targets, including the EIC window, the peak-finding settings and the threads.

    Fields:
        eic: Options for the EICs, with the retention-time range (from, to), the m/z tolerances and the time unit. See CalculateEicOptions: https://phenological.github.io/quantion/?language=python#CalculateEicOptions
        find_peaks: Options for find_peaks, used for every target. See FindPeaksOptions: https://phenological.github.io/quantion/?language=python#FindPeaksOptions
        threads: Number of threads (CPU cores) used to process the targets in parallel. Default 1.

    Documentation: https://phenological.github.io/quantion/?language=python#FindTargetsOptions
    """
    eic: CalculateEicOptions
    find_peaks: FindPeaksOptions
    threads: int


class FindTransitionsOptions(TypedDict, total=False):
    """FindTransitionsOptions · Options

    Options for find_transitions, including the peak-finding settings and the threads.

    Fields:
        find_peaks: Options for find_peaks, used for every transition. See FindPeaksOptions: https://phenological.github.io/quantion/?language=python#FindPeaksOptions
        threads: Number of threads (CPU cores) used to process the transitions in parallel. Default 1.

    Documentation: https://phenological.github.io/quantion/?language=python#FindTransitionsOptions
    """
    find_peaks: FindPeaksOptions
    threads: int


class FindFeaturesOptions(TypedDict, total=False):
    """FindFeaturesOptions · Options

    Options for find_features, including the EIC window, the m/z grid, the peak-finding settings and the threads.

    Fields:
        eic: Options for the EICs, with the retention-time range (from, to), the m/z tolerances and the time unit. See CalculateEicOptions: https://phenological.github.io/quantion/?language=python#CalculateEicOptions
        grid: The m/z grid to scan. See GridOptions: https://phenological.github.io/quantion/?language=python#GridOptions
        find_peaks: Options for find_peaks, used for every grid m/z. See FindPeaksOptions: https://phenological.github.io/quantion/?language=python#FindPeaksOptions
        threads: Number of threads (CPU cores) used to scan the grid in parallel. Default 1.

    Documentation: https://phenological.github.io/quantion/?language=python#FindFeaturesOptions
    """
    eic: CalculateEicOptions
    grid: GridOptions
    find_peaks: FindPeaksOptions
    threads: int


class DeisotopeFeaturesOptions(TypedDict, total=False):
    """DeisotopeFeaturesOptions · Options

    Options for deisotope_features, which say how close an isotope peak must be.

    Fields:
        rt_tolerance: An isotope belongs to M only when its rt is within this many minutes of M. Default 0.02.
        mz_tolerance: A feature is an isotope only when its m/z is within this many Da of the expected isotope position. Default 0.005.

    Documentation: https://phenological.github.io/quantion/?language=python#DeisotopeFeaturesOptions
    """
    rt_tolerance: float
    mz_tolerance: float


class MatchFeaturesOptions(TypedDict, total=False):
    """MatchFeaturesOptions · Options

    Options for match_features. They are the find_features options plus the grouping across samples.

    Fields:
        eic: Options for the EICs, with the retention-time range (from, to), the m/z tolerances and the time unit. See CalculateEicOptions: https://phenological.github.io/quantion/?language=python#CalculateEicOptions
        grid: The m/z grid to scan. See GridOptions: https://phenological.github.io/quantion/?language=python#GridOptions
        grouping: How features are matched across samples. See GroupingOptions: https://phenological.github.io/quantion/?language=python#GroupingOptions
        find_peaks: Options for find_peaks, used for every grid m/z. See FindPeaksOptions: https://phenological.github.io/quantion/?language=python#FindPeaksOptions
        threads: Number of threads (CPU cores) used to process the files in parallel. Default 1.

    Documentation: https://phenological.github.io/quantion/?language=python#MatchFeaturesOptions
    """
    eic: CalculateEicOptions
    grid: GridOptions
    grouping: GroupingOptions
    find_peaks: FindPeaksOptions
    threads: int


class ScanArray(TypedDict):
    """ScanArray · Options

    One data array to read from each scan in get_scans, for example an extra array stored in the file.

    Fields:
        key: Name of the array in the result, as a field of the scan.
        accession: Accession of the array in the file, such as MS:1000514 for m/z.

    Example:
        array: ScanArray = {"key": "mz", "accession": "MS:1000514"}

    Documentation: https://phenological.github.io/quantion/?language=python#ScanArray
    """
    key: str
    accession: str


class GetScanMetadataOptions(TypedDict, total=False):
    """GetScanMetadataOptions · Options

    Options for get_scan_metadata.

    Fields:
        tags: Which tags to keep, by name. A row is kept when the last tag of its path matches, so scan keeps spectrum/scanList/scan[0] but not its scanWindow. An empty list keeps every tag. The names are spectrum, referenceableParamGroupRef, spectrumDescription, scanList, scan, scanWindowList, scanWindow, precursorList, precursor, isolationWindow, selectedIonList, selectedIon, activation, productList, product, binaryDataArrayList and binaryDataArray. Default [].

    Documentation: https://phenological.github.io/quantion/?language=python#GetScanMetadataOptions
    """
    tags: List[str]


class GetScansOptions(TypedDict, total=False):
    """GetScansOptions · Options

    Options for get_scans, including the MS level and which arrays to read.

    Fields:
        level: MS level of the scans, 1 for MS1 and 2 for MS2. 0 reads every level. Default 1.
        time_unit: Unit of the rt you give, minutes or seconds. Default MINUTES.
        arrays: Which data arrays to read from each scan, such as m/z and intensity, given as keys with accessions. The other fields of Scan always come back. Default {"key": "mz", "accession": "MS:1000514"}, {"key": "intensity", "accession": "MS:1000515"}. See ScanArray: https://phenological.github.io/quantion/?language=python#ScanArray

    Documentation: https://phenological.github.io/quantion/?language=python#GetScansOptions
    """
    level: int
    time_unit: Union[TimeUnit, str]
    arrays: List[ScanArray]


class AnnotateFeaturesOptions(TypedDict, total=False):
    """AnnotateFeaturesOptions · Options

    Options for annotate_features, including the library, the model, the tolerances and how many matches to keep.

    Fields:
        library: Spectral library, as a path or URL to an Ionic (.ion) file. Each spectrum has a precursor m/z, a vector array and the params the model names. Only Ionic (.ion) libraries work. Default "https://github.com/josoriom/cipher/releases/latest/download/library_positive.ion".
        model: Spectrum model, as a path or URL to an ONNX file. Default "https://github.com/josoriom/cipher/releases/latest/download/model.onnx".
        matches: How many matches to keep per file, best first. Default 3.
        ppm_tolerance: Precursor m/z tolerance in ppm. The larger of the ppm and Da tolerances is used. Default 20.
        mz_tolerance: Precursor m/z tolerance in Da. The larger of the ppm and Da tolerances is used. Default 0.005.
        rt_range: Largest distance, in minutes, from the peak apex to the MS2 scan. Default 0.3.
        min_relative_intensity: MS2 peaks below this share of the base peak are removed before matching, from 0 to 1. Default 0.02.

    Documentation: https://phenological.github.io/quantion/?language=python#AnnotateFeaturesOptions
    """
    library: str
    model: str
    matches: int
    ppm_tolerance: float
    mz_tolerance: float
    rt_range: float
    min_relative_intensity: float


class ParseIonOptions(TypedDict, total=False):
    """ParseIonOptions · Options

    Options for parse_ion.

    Fields:
        max_cache_size: Maximum memory, in bytes, kept for the parts of the file already read. 0 = no limit. Set it to limit memory on large or remote files. Default 0.

    Documentation: https://phenological.github.io/quantion/?language=python#ParseIonOptions
    """
    max_cache_size: int


class WriteIonOptions(TypedDict, total=False):
    """WriteIonOptions · Options

    Options for write_ion, including where to write and how to compress.

    Fields:
        out: Path or URL to write to. If it is not given, the bytes are returned. Default None.
        overwrite: Replace the file if it already exists. Default False.
        level: Compression level, 0 to 22. A higher level gives a smaller file but takes longer to write. Default 12.
        f32_compress: Store intensities as 32-bit floats, for a smaller file with slightly less precision. Default False.
        block_size: Size of each data block before compression, in MB. A whole number, 1 or more. Default 8.
        mz_window: Width of the m/z windows the spectra are split into, in Da, so a query reads only the windows it needs. 0 keeps each spectrum whole. It only changes how the file is stored. It is not the EIC window. Default 20.
        headers: Extra HTTP headers for the upload, such as a token. Only used with a URL. Default [].

    Documentation: https://phenological.github.io/quantion/?language=python#WriteIonOptions
    """
    out: str
    overwrite: bool
    level: int
    f32_compress: bool
    block_size: int
    mz_window: int
    headers: List[Header]


class WriteMzmlOptions(TypedDict, total=False):
    """WriteMzmlOptions · Options

    Options for write_mzml.

    Fields:
        out: Path to write to. If it is not given, the bytes are returned. Default None.
        overwrite: Replace the file if it already exists. Default False.

    Documentation: https://phenological.github.io/quantion/?language=python#WriteMzmlOptions
    """
    out: str
    overwrite: bool


class GetIonImageOptions(TypedDict, total=False):
    """GetIonImageOptions · Options

    Options for get_ion_image, including the m/z window and the MS level.

    Fields:
        tolerance: Half-width of the m/z window, in Da. There is no ppm option here. Default 0.125.
        level: MS level of the scans used. Default 1.
        on_progress: Function called while reading, with the bytes fetched, the total bytes and the bytes held in memory. Useful for a progress bar on remote files. Default None.

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
})
Peak.__doc__ = """Peak · Results

One chromatographic peak, or an empty peak (every field is 0) when no peak is found.

Fields:
    rt: Apex retention time, in minutes.
    from: Start of the peak, in minutes.
    to: End of the peak, in minutes.
    intensity: Apex intensity: the peak height.
    integral: Area (integral) of the peak, calculated with the trapezoid rule on the chromatogram trace between from and to.
    n_points: Number of data points in the peak, from its start to its end.

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
})


class Feature(_FeatureFields, total=False):
    """Feature · Results

    A peak at one m/z, found without a target list. match_features also adds its frequency across samples.

    Fields:
        mz: m/z of the feature (Da).
        rt: Apex retention time, in minutes.
        from: Start of the peak, in minutes.
        to: End of the peak, in minutes.
        intensity: Apex intensity (the peak height).
        integral: Area (integral) of the peak.
        n_points: Number of data points in the peak, from its start to its end.
        frequency: Fraction of samples where the feature was found, from 0 to 1. Only match_features sets it. Optional.

    Documentation: https://phenological.github.io/quantion/?language=python#Feature
    """
    frequency: float


class DataXY(TypedDict):
    """DataXY · Results

    A chromatogram (trace), with x as the retention times and y as the intensities. calculate_eic returns it, and find_peaks and get_peak take it.

    Fields:
        x: Retention times, in minutes.
        y: Intensities, one per retention time.

    Documentation: https://phenological.github.io/quantion/?language=python#DataXY
    """
    x: np.ndarray
    y: np.ndarray


class NoiseLevel(TypedDict):
    """NoiseLevel · Results

    Result of find_noise_level.

    Fields:
        width: Max width, in points, of the noise spikes the estimate came from.
        intensity: Max intensity of the noise spikes, taken as the noise level.

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

    The parameters that describe the shape of one peak. fit_peak returns them, and you can also write them by hand to simulate a peak with draw_peaks. With tail = 0 the peak is a Gaussian, and with tail above 0 it is an EMG.

    Fields:
        height: Height of the curve at its apex (intensity).
        center: Apex retention time, in minutes.
        fwhm: Full width at half maximum of the Gaussian part, in minutes. With a tail, the drawn peak is a little wider.
        tail: Tailing to the right, in minutes. 0 = Gaussian, above 0 = EMG.
        r2: Fit quality (0–1), set by fit_peak. Not needed to draw. Optional.

    Example:
        peak: PeakParameters = {"height": 9800, "center": 2.38, "fwhm": 0.021, "tail": 0.006}

    Documentation: https://phenological.github.io/quantion/?language=python#PeakParameters
    """
    r2: float


class PredictedRt(TypedDict):
    """PredictedRt · Results

    One predicted retention time, from predict_rts.

    Fields:
        rt: Predicted retention time, in minutes.
        uncertainty: How unsure the prediction is, in minutes.

    Documentation: https://phenological.github.io/quantion/?language=python#PredictedRt
    """
    rt: float
    uncertainty: float


class ScanParamKind(str, Enum):
    """ScanParamKind · Enums

    What a row from get_scan_metadata holds.

    Fields:
        ATTRIBUTE: An attribute of the tag, such as id or defaultArrayLength.
        CV: A cvParam, a param from the controlled vocabulary, with an accession.
        USER: A userParam, a param named by the software that wrote the file.

    Documentation: https://phenological.github.io/quantion/?language=python#ScanParamKind
    """
    ATTRIBUTE = "ATTRIBUTE"
    CV = "CV"
    USER = "USER"


class ScanParam(TypedDict):
    """ScanParam · Results

    One row from get_scan_metadata. It is an attribute, a cvParam or a userParam of one tag in a spectrum.

    Fields:
        path: Where the tag sits, from spectrum down. Repeated tags carry their position from 0, such as spectrum/precursorList/precursor[0]/selectedIonList/selectedIon[0].
        kind: What the row holds, ATTRIBUTE, CV or USER. See ScanParamKind.
        accession: Accession of a cvParam, such as MS:1000511. Missing for attributes and userParams.
        name: Name of the attribute or param, such as ms level.
        value: Value as text, such as 2. Missing when the file gives none.
        unit: Unit name, such as minute. Missing when there is none.
        unit_accession: Unit accession, such as UO:0000031. Missing when there is none.

    Documentation: https://phenological.github.io/quantion/?language=python#ScanParam
    """
    path: str
    kind: ScanParamKind
    accession: Optional[str]
    name: str
    value: Optional[str]
    unit: Optional[str]
    unit_accession: Optional[str]


class CompoundMatch(TypedDict):
    """CompoundMatch · Results

    One library spectrum that matches a feature in one file.

    Fields:
        label: Name of the compound, from the library param the model names as label.
        id: Id of the spectrum in its database.
        database: Database the id comes from.
        score: Average of the model vector similarity and the cosine, from 0 to 1.
        cosine: Cosine similarity of the cleaned spectra, from 0 to 1.
        mz: m/z values of the library spectrum.
        intensity: Intensities of the library spectrum.
        metadata: The other params of the library spectrum, by name.

    Documentation: https://phenological.github.io/quantion/?language=python#CompoundMatch
    """
    label: str
    id: str
    database: str
    score: float
    cosine: float
    mz: np.ndarray
    intensity: np.ndarray
    metadata: Dict[str, str]


class FileAnnotation(TypedDict):
    """FileAnnotation · Results

    What one file says about a feature in annotate_features.

    Fields:
        file: Position of the file in files, from 0.
        scan_index: Index of the MS2 scan used, or none when the file has no peak or no MS2 scan close to it.
        matches: The best matches in this file, best first, at most matches of them.

    Documentation: https://phenological.github.io/quantion/?language=python#FileAnnotation
    """
    file: int
    scan_index: Optional[int]
    matches: List[CompoundMatch]


class FeatureAnnotation(TypedDict):
    """FeatureAnnotation · Results

    The name found for one feature. The label is the compound that is the best match in the most files, and ties go to the higher score.

    Fields:
        mz: m/z of the feature.
        rt: Retention time of the feature, in minutes.
        label: Name of the compound, or none when no file has a match.
        id: Id of the compound in its database, from the agreeing file with the highest score.
        database: Database the id comes from.
        frequency: Share of the files whose best match is this compound, from 0 to 1.
        score: Median score over the files that agree on the label.
        cosine: Median cosine similarity over the files that agree on the label.
        files: What each file says about the feature, in the same order as files.

    Documentation: https://phenological.github.io/quantion/?language=python#FeatureAnnotation
    """
    mz: float
    rt: float
    label: Optional[str]
    id: Optional[str]
    database: Optional[str]
    frequency: float
    score: Optional[float]
    cosine: Optional[float]
    files: List[FileAnnotation]


class Scan(TypedDict):
    """Scan · Results

    One scan (spectrum) from get_scans, with its retention time, its m/z and intensity values, and its details.

    Fields:
        index: Position of the scan in the file, from 0. Use it with get_scan_metadata.
        rt: Retention time, in minutes.
        mz: m/z values of the spectrum.
        intensity: Intensities, one per m/z value.
        level: MS level, 1 for MS1 and 2 for MS2.
        precursor_mz: m/z of the precursor ion that was fragmented (MS2 and higher).
        base_peak_mz: m/z of the most intense peak in the scan (base peak).
        base_peak_intensity: Intensity of the base peak.
        total_ion_current: Sum of all intensities in this scan. Plotted against rt over the whole injection, these values form the TIC chromatogram.
        polarity: Ion mode, 1 for positive, 2 for negative and 0 for unknown.
        x: x position in imaging files. 0 in other files.
        y: y position in imaging files. 0 in other files.
        z: z position in imaging files. 0 in other files.

    Documentation: https://phenological.github.io/quantion/?language=python#Scan
    """
    index: int
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

    An imaging map of one m/z, from get_ion_image.

    Fields:
        width: Number of pixels across.
        height: Number of pixels down.
        min_x: x position of the first column.
        min_y: y position of the first row.
        min_z: Lowest z position.
        max_z: Highest z position.
        data: Summed intensity of each pixel, row by row (width × height values).
        counts: Number of scans in each pixel, in the same order as data.

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
