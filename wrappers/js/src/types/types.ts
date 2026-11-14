export type BinaryInput = Uint8Array | ArrayBuffer;
export type IonInput = BinaryInput | URL | string;

/**
 * TimeUnit · Enums
 *
 * Unit of the retention times you give to a function. Results are always in minutes.
 *
 * Fields:
 * - `MINUTES`: Minutes.
 * - `SECONDS`: Seconds.
 * @see {@link https://phenological.github.io/quantion/?language=js#TimeUnit | Documentation}
 */
export const TimeUnit = { MINUTES: "MINUTES", SECONDS: "SECONDS" } as const;
export type TimeUnit = (typeof TimeUnit)[keyof typeof TimeUnit];

/**
 * PeakShape · Enums
 *
 * Curve used to fit or check a peak.
 *
 * Fields:
 * - `GAUSSIAN`: Symmetric bell curve.
 * - `EMG`: Exponentially modified Gaussian: a bell curve with a tail on the right.
 * @see {@link https://phenological.github.io/quantion/?language=js#PeakShape | Documentation}
 */
export const PeakShape = { GAUSSIAN: "GAUSSIAN", EMG: "EMG" } as const;
export type PeakShape = (typeof PeakShape)[keyof typeof PeakShape];

/**
 * FromTo · Inputs
 *
 * A range with a start and an end, used for retention-time windows and m/z ranges.
 *
 * Fields:
 * - `from`: Start of the range.
 * - `to`: End of the range.
 *
 * @example
 * ```ts
 * const window: FromTo = { from: 1.0, to: 2.5 };
 * ```
 * @see {@link https://phenological.github.io/quantion/?language=js#FromTo | Documentation}
 */
export interface FromTo {
  from: number;
  to: number;
}

/**
 * Target · Inputs
 *
 * One compound to look for, with its m/z and expected retention time. range limits the search in find_targets, and a missing or 0 range searches the whole EIC. predict_rts and calibrate do not use range.
 *
 * Fields:
 * - `id`: Your name for the compound. It is not used in the search. Optional.
 * - `mz`: The m/z of the ion to look for (Da), not the neutral mass of the compound. For example, for the protonated molecule [M+H]+ in positive mode, use M + 1.00728, and for the deprotonated molecule [M−H]− in negative mode, use M − 1.00728, where M is the monoisotopic mass.
 * - `rt`: Expected retention time, in minutes by default (or in the time unit set in the EIC options).
 * - `range`: Half-width of the search window around rt, in the same unit as rt. Missing or 0 searches the whole EIC. Optional.
 *
 * @example
 * ```ts
 * const target: Target = { id: "compound_1", mz: 260.103, rt: 2.383, range: 0.05 };
 * ```
 * @see {@link https://phenological.github.io/quantion/?language=js#Target | Documentation}
 */
export interface Target {
  id?: string;
  mz: number;
  rt: number;
  range?: number;
}

/**
 * Transition · Inputs
 *
 * One SRM or MRM transition, with its precursor (Q1) and product (Q3) m/z.
 *
 * Fields:
 * - `id`: Your name for the transition. Optional.
 * - `precursor`: Precursor (Q1) m/z.
 * - `product`: Product (Q3) m/z.
 * - `rt`: Expected retention time, in minutes.
 * - `range`: Half-width of the search window around rt, in minutes.
 *
 * @example
 * ```ts
 * const transition: Transition = { id: "compound_1", precursor: 89.04768, product: 89.04768, rt: 2.385, range: 0.1 };
 * ```
 * @see {@link https://phenological.github.io/quantion/?language=js#Transition | Documentation}
 */
export interface Transition {
  id?: string;
  precursor: number;
  product: number;
  rt: number;
  range: number;
}

/**
 * Header · Inputs
 *
 * One HTTP header sent with an upload in write_ion, for example a token. Values are never printed. A line break, or the names Host, Content-Length or Transfer-Encoding, give an invalid_argument error.
 *
 * Fields:
 * - `name`: Name of the header, such as Authorization.
 * - `value`: Value of the header.
 *
 * @example
 * ```ts
 * const header: Header = { name: "Authorization", value: "Bearer <token>" };
 * ```
 * @see {@link https://phenological.github.io/quantion/?language=js#Header | Documentation}
 */
export interface Header {
  name: string;
  value: string;
}

export interface RtQuery {
  rt: number | FromTo;
}

export interface MzQuery {
  mz: number | FromTo;
}

/**
 * ScanQuery · Inputs
 *
 * Which scans get_scans returns, by retention time, by precursor m/z, or by both. Ranges give every scan inside them, and one value gives the closest scan.
 *
 * Fields:
 * - `rt`: Retention time, as a range, as one value for the closest scan, or nothing.
 * - `mz`: Precursor m/z, as a range, as one value for the closest scan, or nothing.
 *
 * @example
 * ```ts
 * const query: ScanQuery = { rt: { from: 2.3, to: 2.4 }, mz: 300.0 };
 * ```
 * @see {@link https://phenological.github.io/quantion/?language=js#ScanQuery | Documentation}
 */
export type ScanQuery = RtQuery | MzQuery;

/**
 * FindPeaksOptions · Options
 *
 * Options for find_peaks. Every field is optional.
 *
 * Fields:
 * - `minIntegral`: Drop peaks whose area (integral) is below this value. Off when not given. Default `off`.
 * - `minIntensity`: Drop peaks whose apex intensity is below this value. Raise it to ignore small peaks. Default `500`.
 * - `minPeakWidthPoints`: Drop peaks with fewer data points than this. Default `3`.
 * - `noise`: Fixed noise level (intensity), used only when auto_noise is false. Default `off`.
 * - `autoNoise`: Estimate the noise from the chromatogram. Turn off and set noise to use a fixed value. Default `true`.
 * - `autoBaseline`: Subtract an airPLS baseline before finding peaks. Turn off for windows shorter than 1 minute: they do not have enough points to estimate the baseline and can give misleading results. Default `true`.
 * - `lambda`: Smoothness of the airPLS baseline. A larger value gives a smoother baseline that follows only slow drift. Default `20`.
 * - `maxIterations`: Maximum number of airPLS iterations used to estimate the baseline. Default `4`.
 * - `allowOverlap`: Let neighbouring peaks share points at their border. When off, overlapping peaks are split at the valley between them. Default `false`.
 * - `minSnr`: Minimum signal-to-noise ratio. A peak’s apex must be above min_snr × noise. Default `2`.
 * - `minR2`: Minimum fit quality (0–1) of the peak shape. Peaks that fit the shape worse are dropped. 0 keeps every peak. Default `0`.
 * - `shape`: Curve used to check the peak shape for min_r2: EMG or GAUSSIAN. Default `"EMG"`.
 * - `kernelSize`: Savitzky–Golay smoothing window, in points, to smooth noisy chromatograms. 0 = no smoothing. A larger value gives a smoother chromatogram. Default `0`.
 * @see {@link https://phenological.github.io/quantion/?language=js#FindPeaksOptions | Documentation}
 */
export interface FindPeaksOptions {
  minIntegral?: number;
  minIntensity?: number;
  minPeakWidthPoints?: number;
  noise?: number;
  autoNoise?: boolean;
  autoBaseline?: boolean;
  lambda?: number;
  maxIterations?: number;
  allowOverlap?: boolean;
  minSnr?: number;
  minR2?: number;
  shape?: PeakShape;
  kernelSize?: number;
}

/**
 * GetPeakOptions · Options
 *
 * Options for get_peak, including the search window and the peak-finding settings.
 *
 * Fields:
 * - `range`: Half-width of the search window, in minutes. Only peaks within rt ± range are considered. 0 searches the whole trace. Default `0`.
 * - `findPeaks`: Options for find_peaks function. See {@link https://phenological.github.io/quantion/?language=js#FindPeaksOptions | FindPeaksOptions}.
 * @see {@link https://phenological.github.io/quantion/?language=js#GetPeakOptions | Documentation}
 */
export interface GetPeakOptions {
  range?: number;
  findPeaks?: FindPeaksOptions;
}

/**
 * FitPeakOptions · Options
 *
 * Options for fit_peak function.
 *
 * Fields:
 * - `shape`: Curve to fit: EMG (with a right tail) or GAUSSIAN (symmetric). Default `"EMG"`.
 * - `intensity`: Apex intensity to start the fit from. 0 = use the apex of the peak found at rt. Default `0`.
 * @see {@link https://phenological.github.io/quantion/?language=js#FitPeakOptions | Documentation}
 */
export interface FitPeakOptions {
  shape?: PeakShape;
  intensity?: number;
}

/**
 * DrawPeaksOptions · Options
 *
 * Options for draw_peaks, including the time range and the number of points.
 *
 * Fields:
 * - `from`: Start of the simulated chromatogram, in minutes. Leave from and to at 0 to cover all the peaks automatically. Default `0`.
 * - `to`: End of the simulated chromatogram, in minutes. Default `0`.
 * - `points`: Number of points drawn from from to to. Default `1000`.
 * @see {@link https://phenological.github.io/quantion/?language=js#DrawPeaksOptions | Documentation}
 */
export interface DrawPeaksOptions {
  from?: number;
  to?: number;
  points?: number;
}

/**
 * CalculateBaselineOptions · Options
 *
 * Options for calculate_baseline, which estimates the baseline with airPLS.
 *
 * Fields:
 * - `lambda`: Smoothness of the baseline. A larger value gives a smoother baseline that follows only slow drift. Default `20`.
 * - `maxIterations`: Maximum number of iterations used to estimate the baseline. Default `4`.
 * @see {@link https://phenological.github.io/quantion/?language=js#CalculateBaselineOptions | Documentation}
 */
export interface CalculateBaselineOptions {
  lambda?: number;
  maxIterations?: number;
}

/**
 * CalculateEicOptions · Options
 *
 * Options for calculate_eic, including the retention-time range to read and the m/z window. The same options are used by find_targets, find_features and match_features. The wider of the ppm and Da windows is used.
 *
 * Fields:
 * - `from`: Start of the retention-time range to read. Default `0`.
 * - `to`: End of the retention-time range to read. By default, the run is read to the end. Default `Infinity`.
 * - `ppmTolerance`: Half-width of the m/z window, in ppm of the m/z. Default `20`.
 * - `mzTolerance`: Half-width of the m/z window, in Da. Default `0.005`.
 * - `timeUnit`: Unit of from and to, minutes or seconds. Default `"MINUTES"`.
 * @see {@link https://phenological.github.io/quantion/?language=js#CalculateEicOptions | Documentation}
 */
export interface CalculateEicOptions {
  from?: number;
  to?: number;
  ppmTolerance?: number;
  mzTolerance?: number;
  timeUnit?: TimeUnit;
}

/**
 * GridOptions · Options
 *
 * The m/z grid that find_features and match_features scan, from start to end in steps of step_size.
 *
 * Fields:
 * - `start`: Lowest m/z of the grid. Default `40`.
 * - `end`: Highest m/z of the grid. Default `1000`.
 * - `stepSize`: Distance between two grid m/z values, in Da. A smaller step is finer but slower. Default `0.005`.
 * @see {@link https://phenological.github.io/quantion/?language=js#GridOptions | Documentation}
 */
export interface GridOptions {
  start?: number;
  end?: number;
  stepSize?: number;
}

/**
 * GroupingOptions · Options
 *
 * How match_features decides that features from different samples are the same feature.
 *
 * Fields:
 * - `ppmTolerance`: Features are the same when their m/z is within this many ppm, or within mz_tolerance. Default `5`.
 * - `mzTolerance`: Features are the same when their m/z is within this many Da, or within ppm_tolerance. Default `0.0025`.
 * - `rtTolerance`: Features are the same only when their rt is also within this many minutes. Default `0.05`.
 * - `minFrequency`: Keep a feature only if it is found in at least this fraction of samples, from 0 to 1. For example, 0.8 keeps features seen in 80% of the samples. Default `0`.
 * @see {@link https://phenological.github.io/quantion/?language=js#GroupingOptions | Documentation}
 */
export interface GroupingOptions {
  ppmTolerance?: number;
  mzTolerance?: number;
  rtTolerance?: number;
  minFrequency?: number;
}

/**
 * FindTargetsOptions · Options
 *
 * Options for find_targets, including the EIC window, the peak-finding settings and the threads.
 *
 * Fields:
 * - `eic`: Options for the EICs, with the retention-time range (from, to), the m/z tolerances and the time unit. See {@link https://phenological.github.io/quantion/?language=js#CalculateEicOptions | CalculateEicOptions}.
 * - `findPeaks`: Options for find_peaks, used for every target. See {@link https://phenological.github.io/quantion/?language=js#FindPeaksOptions | FindPeaksOptions}.
 * - `threads`: Number of threads (CPU cores) used to process the targets in parallel. Default `1`.
 * @see {@link https://phenological.github.io/quantion/?language=js#FindTargetsOptions | Documentation}
 */
export interface FindTargetsOptions {
  eic?: CalculateEicOptions;
  findPeaks?: FindPeaksOptions;
  threads?: number;
}

/**
 * FindTransitionsOptions · Options
 *
 * Options for find_transitions, including the peak-finding settings and the threads.
 *
 * Fields:
 * - `findPeaks`: Options for find_peaks, used for every transition. See {@link https://phenological.github.io/quantion/?language=js#FindPeaksOptions | FindPeaksOptions}.
 * - `threads`: Number of threads (CPU cores) used to process the transitions in parallel. Default `1`.
 * @see {@link https://phenological.github.io/quantion/?language=js#FindTransitionsOptions | Documentation}
 */
export interface FindTransitionsOptions {
  findPeaks?: FindPeaksOptions;
  threads?: number;
}

/**
 * FindFeaturesOptions · Options
 *
 * Options for find_features, including the EIC window, the m/z grid, the peak-finding settings and the threads.
 *
 * Fields:
 * - `eic`: Options for the EICs, with the retention-time range (from, to), the m/z tolerances and the time unit. See {@link https://phenological.github.io/quantion/?language=js#CalculateEicOptions | CalculateEicOptions}.
 * - `grid`: The m/z grid to scan. See {@link https://phenological.github.io/quantion/?language=js#GridOptions | GridOptions}.
 * - `findPeaks`: Options for find_peaks, used for every grid m/z. See {@link https://phenological.github.io/quantion/?language=js#FindPeaksOptions | FindPeaksOptions}.
 * - `threads`: Number of threads (CPU cores) used to scan the grid in parallel. Default `1`.
 * @see {@link https://phenological.github.io/quantion/?language=js#FindFeaturesOptions | Documentation}
 */
export interface FindFeaturesOptions {
  eic?: CalculateEicOptions;
  grid?: GridOptions;
  findPeaks?: FindPeaksOptions;
  threads?: number;
}

/**
 * DeisotopeFeaturesOptions · Options
 *
 * Options for deisotope_features, which say how close an isotope peak must be.
 *
 * Fields:
 * - `rtTolerance`: An isotope belongs to M only when its rt is within this many minutes of M. Default `0.02`.
 * - `mzTolerance`: A feature is an isotope only when its m/z is within this many Da of the expected isotope position. Default `0.005`.
 * @see {@link https://phenological.github.io/quantion/?language=js#DeisotopeFeaturesOptions | Documentation}
 */
export interface DeisotopeFeaturesOptions {
  rtTolerance?: number;
  mzTolerance?: number;
}

/**
 * MatchFeaturesOptions · Options
 *
 * Options for match_features. They are the find_features options plus the grouping across samples.
 *
 * Fields:
 * - `eic`: Options for the EICs, with the retention-time range (from, to), the m/z tolerances and the time unit. See {@link https://phenological.github.io/quantion/?language=js#CalculateEicOptions | CalculateEicOptions}.
 * - `grid`: The m/z grid to scan. See {@link https://phenological.github.io/quantion/?language=js#GridOptions | GridOptions}.
 * - `grouping`: How features are matched across samples. See {@link https://phenological.github.io/quantion/?language=js#GroupingOptions | GroupingOptions}.
 * - `findPeaks`: Options for find_peaks, used for every grid m/z. See {@link https://phenological.github.io/quantion/?language=js#FindPeaksOptions | FindPeaksOptions}.
 * - `threads`: Number of threads (CPU cores) used to process the files in parallel. Default `1`.
 * @see {@link https://phenological.github.io/quantion/?language=js#MatchFeaturesOptions | Documentation}
 */
export interface MatchFeaturesOptions {
  eic?: CalculateEicOptions;
  grid?: GridOptions;
  grouping?: GroupingOptions;
  findPeaks?: FindPeaksOptions;
  threads?: number;
}

/**
 * GetScanMetadataOptions · Options
 *
 * Options for get_scan_metadata.
 *
 * Fields:
 * - `tags`: Which tags to keep, by name. A row is kept when the last tag of its path matches, so scan keeps spectrum/scanList/scan[0] but not its scanWindow. An empty list keeps every tag. The names are spectrum, referenceableParamGroupRef, spectrumDescription, scanList, scan, scanWindowList, scanWindow, precursorList, precursor, isolationWindow, selectedIonList, selectedIon, activation, productList, product, binaryDataArrayList and binaryDataArray. Default `[]`.
 * @see {@link https://phenological.github.io/quantion/?language=js#GetScanMetadataOptions | Documentation}
 */
export interface GetScanMetadataOptions {
  tags?: string[];
}

/**
 * GetScansOptions · Options
 *
 * Options for get_scans, including the MS level and which arrays to read.
 *
 * Fields:
 * - `level`: MS level of the scans, 1 for MS1 and 2 for MS2. 0 reads every level. Default `1`.
 * - `timeUnit`: Unit of the rt you give, minutes or seconds. Default `"MINUTES"`.
 * - `arrays`: Which data arrays to read from each scan, such as m/z and intensity, given as keys with accessions. The other fields of Scan always come back. Default `{ key: "mz", accession: "MS:1000514" }, { key: "intensity", accession: "MS:1000515" }`. See {@link https://phenological.github.io/quantion/?language=js#ScanArray | ScanArray}.
 * @see {@link https://phenological.github.io/quantion/?language=js#GetScansOptions | Documentation}
 */
export interface GetScansOptions {
  level?: number;
  timeUnit?: TimeUnit;
  arrays?: ScanArray[];
}

/**
 * ScanArray · Options
 *
 * One data array to read from each scan in get_scans, for example an extra array stored in the file.
 *
 * Fields:
 * - `key`: Name of the array in the result, as a field of the scan.
 * - `accession`: Accession of the array in the file, such as MS:1000514 for m/z.
 *
 * @example
 * ```ts
 * const array: ScanArray = { key: "mz", accession: "MS:1000514" };
 * ```
 * @see {@link https://phenological.github.io/quantion/?language=js#ScanArray | Documentation}
 */
export interface ScanArray {
  key: string;
  accession: string;
}

/**
 * ParseIonOptions · Options
 *
 * Options for parse_ion.
 *
 * Fields:
 * - `maxCacheSize`: Maximum memory, in bytes, kept for the parts of the file already read. 0 = no limit. Set it to limit memory on large or remote files. Default `0`.
 * @see {@link https://phenological.github.io/quantion/?language=js#ParseIonOptions | Documentation}
 */
export interface ParseIonOptions {
  maxCacheSize?: number;
}

/**
 * WriteIonOptions · Options
 *
 * Options for write_ion, including where to write and how to compress.
 *
 * Fields:
 * - `out`: Path or URL to write to. If it is not given, the bytes are returned. Default `undefined`.
 * - `overwrite`: Replace the file if it already exists. Default `false`.
 * - `level`: Compression level, 0 to 22. A higher level gives a smaller file but takes longer to write. Default `12`.
 * - `f32Compress`: Store intensities as 32-bit floats, for a smaller file with slightly less precision. Default `false`.
 * - `blockSize`: Size of each data block before compression, in MB. A whole number, 1 or more. Default `8`.
 * - `mzWindow`: Width of the m/z windows the spectra are split into, in Da, so a query reads only the windows it needs. 0 keeps each spectrum whole. It only changes how the file is stored. It is not the EIC window. Default `20`.
 * - `headers`: Extra HTTP headers for the upload, such as a token. Only used with a URL. Default `[]`.
 * @see {@link https://phenological.github.io/quantion/?language=js#WriteIonOptions | Documentation}
 */
export interface WriteIonOptions {
  out?: string | URL;
  overwrite?: boolean;
  level?: number;
  f32Compress?: boolean;
  blockSize?: number;
  mzWindow?: number;
  headers?: Header[];
}

/**
 * WriteMzmlOptions · Options
 *
 * Options for write_mzml.
 *
 * Fields:
 * - `out`: Path to write to. If it is not given, the bytes are returned. Default `undefined`.
 * - `overwrite`: Replace the file if it already exists. Default `false`.
 * @see {@link https://phenological.github.io/quantion/?language=js#WriteMzmlOptions | Documentation}
 */
export interface WriteMzmlOptions {
  out?: string;
  overwrite?: boolean;
}

/**
 * GetIonImageOptions · Options
 *
 * Options for get_ion_image, including the m/z window and the MS level.
 *
 * Fields:
 * - `tolerance`: Half-width of the m/z window, in Da. There is no ppm option here. Default `0.125`.
 * - `level`: MS level of the scans used. Default `1`.
 * - `onProgress`: Function called while reading, with the bytes fetched, the total bytes and the bytes held in memory. Useful for a progress bar on remote files. Default `undefined`.
 * @see {@link https://phenological.github.io/quantion/?language=js#GetIonImageOptions | Documentation}
 */
export interface GetIonImageOptions {
  tolerance?: number;
  level?: number;
  onProgress?: (fetched: number, total: number, heldBytes: number) => void;
}

/**
 * Peak · Results
 *
 * One chromatographic peak, or an empty peak (every field is 0) when no peak is found.
 *
 * Fields:
 * - `rt`: Apex retention time, in minutes.
 * - `from`: Start of the peak, in minutes.
 * - `to`: End of the peak, in minutes.
 * - `intensity`: Apex intensity: the peak height.
 * - `integral`: Area (integral) of the peak, calculated with the trapezoid rule on the chromatogram trace between from and to.
 * - `nPoints`: Number of data points in the peak, from its start to its end.
 * @see {@link https://phenological.github.io/quantion/?language=js#Peak | Documentation}
 */
export interface Peak {
  rt: number;
  from: number;
  to: number;
  intensity: number;
  integral: number;
  nPoints: number;
}

/**
 * Feature · Results
 *
 * A peak at one m/z, found without a target list. match_features also adds its frequency across samples.
 *
 * Fields:
 * - `mz`: m/z of the feature (Da).
 * - `rt`: Apex retention time, in minutes.
 * - `from`: Start of the peak, in minutes.
 * - `to`: End of the peak, in minutes.
 * - `intensity`: Apex intensity (the peak height).
 * - `integral`: Area (integral) of the peak.
 * - `nPoints`: Number of data points in the peak, from its start to its end.
 * - `frequency`: Fraction of samples where the feature was found, from 0 to 1. Only match_features sets it. Optional.
 * @see {@link https://phenological.github.io/quantion/?language=js#Feature | Documentation}
 */
export interface Feature {
  mz: number;
  rt: number;
  from: number;
  to: number;
  intensity: number;
  integral: number;
  nPoints: number;
  frequency?: number;
}

/**
 * DataXY · Results
 *
 * A chromatogram (trace), with x as the retention times and y as the intensities. calculate_eic returns it, and find_peaks and get_peak take it.
 *
 * Fields:
 * - `x`: Retention times, in minutes.
 * - `y`: Intensities, one per retention time.
 * @see {@link https://phenological.github.io/quantion/?language=js#DataXY | Documentation}
 */
export interface DataXY {
  x: Float64Array;
  y: Float64Array;
}

/**
 * NoiseLevel · Results
 *
 * Result of find_noise_level.
 *
 * Fields:
 * - `width`: Max width, in points, of the noise spikes the estimate came from.
 * - `intensity`: Max intensity of the noise spikes, taken as the noise level.
 * @see {@link https://phenological.github.io/quantion/?language=js#NoiseLevel | Documentation}
 */
export interface NoiseLevel {
  width: number;
  intensity: number;
}

/**
 * PeakParameters · Results
 *
 * The parameters that describe the shape of one peak. fit_peak returns them, and you can also write them by hand to simulate a peak with draw_peaks. With tail = 0 the peak is a Gaussian, and with tail above 0 it is an EMG.
 *
 * Fields:
 * - `height`: Height of the curve at its apex (intensity).
 * - `center`: Apex retention time, in minutes.
 * - `fwhm`: Full width at half maximum of the Gaussian part, in minutes. With a tail, the drawn peak is a little wider.
 * - `tail`: Tailing to the right, in minutes. 0 = Gaussian, above 0 = EMG.
 * - `r2`: Fit quality (0–1), set by fit_peak. Not needed to draw. Optional.
 *
 * @example
 * ```ts
 * const peak: PeakParameters = { height: 9800, center: 2.38, fwhm: 0.021, tail: 0.006 };
 * ```
 * @see {@link https://phenological.github.io/quantion/?language=js#PeakParameters | Documentation}
 */
export interface PeakParameters {
  height: number;
  center: number;
  fwhm: number;
  tail: number;
  r2?: number;
}

/**
 * PredictedRt · Results
 *
 * One predicted retention time, from predict_rts.
 *
 * Fields:
 * - `rt`: Predicted retention time, in minutes.
 * - `uncertainty`: How unsure the prediction is, in minutes.
 * @see {@link https://phenological.github.io/quantion/?language=js#PredictedRt | Documentation}
 */
export interface PredictedRt {
  rt: number;
  uncertainty: number;
}

/**
 * ScanParamKind · Enums
 *
 * What a row from get_scan_metadata holds.
 *
 * Fields:
 * - `ATTRIBUTE`: An attribute of the tag, such as id or defaultArrayLength.
 * - `CV`: A cvParam, a param from the controlled vocabulary, with an accession.
 * - `USER`: A userParam, a param named by the software that wrote the file.
 * @see {@link https://phenological.github.io/quantion/?language=js#ScanParamKind | Documentation}
 */
export const ScanParamKind = {
  ATTRIBUTE: "ATTRIBUTE",
  CV: "CV",
  USER: "USER",
} as const;
export type ScanParamKind = (typeof ScanParamKind)[keyof typeof ScanParamKind];

/**
 * ScanParam · Results
 *
 * One row from get_scan_metadata. It is an attribute, a cvParam or a userParam of one tag in a spectrum.
 *
 * Fields:
 * - `path`: Where the tag sits, from spectrum down. Repeated tags carry their position from 0, such as spectrum/precursorList/precursor[0]/selectedIonList/selectedIon[0].
 * - `kind`: What the row holds, ATTRIBUTE, CV or USER. See ScanParamKind.
 * - `accession`: Accession of a cvParam, such as MS:1000511. Missing for attributes and userParams.
 * - `name`: Name of the attribute or param, such as ms level.
 * - `value`: Value as text, such as 2. Missing when the file gives none.
 * - `unit`: Unit name, such as minute. Missing when there is none.
 * - `unitAccession`: Unit accession, such as UO:0000031. Missing when there is none.
 * @see {@link https://phenological.github.io/quantion/?language=js#ScanParam | Documentation}
 */
export interface ScanParam {
  path: string;
  kind: ScanParamKind;
  accession: string | null;
  name: string;
  value: string | null;
  unit: string | null;
  unitAccession: string | null;
}

/**
 * Scan · Results
 *
 * One scan (spectrum) from get_scans, with its retention time, its m/z and intensity values, and its details.
 *
 * Fields:
 * - `index`: Position of the scan in the file, from 0. Use it with get_scan_metadata.
 * - `rt`: Retention time, in minutes.
 * - `mz`: m/z values of the spectrum.
 * - `intensity`: Intensities, one per m/z value.
 * - `level`: MS level, 1 for MS1 and 2 for MS2.
 * - `precursorMz`: m/z of the precursor ion that was fragmented (MS2 and higher).
 * - `basePeakMz`: m/z of the most intense peak in the scan (base peak).
 * - `basePeakIntensity`: Intensity of the base peak.
 * - `totalIonCurrent`: Sum of all intensities in this scan. Plotted against rt over the whole injection, these values form the TIC chromatogram.
 * - `polarity`: Ion mode, 1 for positive, 2 for negative and 0 for unknown.
 * - `x`: x position in imaging files. 0 in other files.
 * - `y`: y position in imaging files. 0 in other files.
 * - `z`: z position in imaging files. 0 in other files.
 * @see {@link https://phenological.github.io/quantion/?language=js#Scan | Documentation}
 */
export interface Scan {
  index: number;
  rt: number;
  mz: Float64Array;
  intensity: Float64Array;
  level: number;
  precursorMz: number;
  basePeakMz: number;
  basePeakIntensity: number;
  totalIonCurrent: number;
  polarity: number;
  x: number;
  y: number;
  z: number;
  [array: string]: number | Float64Array;
}

/**
 * IonImage · Results
 *
 * An imaging map of one m/z, from get_ion_image.
 *
 * Fields:
 * - `width`: Number of pixels across.
 * - `height`: Number of pixels down.
 * - `minX`: x position of the first column.
 * - `minY`: y position of the first row.
 * - `minZ`: Lowest z position.
 * - `maxZ`: Highest z position.
 * - `data`: Summed intensity of each pixel, row by row (width × height values).
 * - `counts`: Number of scans in each pixel, in the same order as data.
 * @see {@link https://phenological.github.io/quantion/?language=js#IonImage | Documentation}
 */
export interface IonImage {
  width: number;
  height: number;
  minX: number;
  minY: number;
  minZ: number;
  maxZ: number;
  data: Float64Array;
  counts: Uint32Array;
}

/**
 * AnnotateFeaturesOptions · Options
 *
 * Options for annotate_features, including the library, the model, the tolerances and how many matches to keep.
 *
 * Fields:
 * - `library`: Spectral library, as a path or URL to an Ionic (.ion) file. Each spectrum has a precursor m/z, a vector array and the params the model names. Only Ionic (.ion) libraries work. Default `"https://github.com/josoriom/cipher/releases/latest/download/library_positive.ion"`.
 * - `model`: Spectrum model, as a path or URL to an ONNX file. Default `"https://github.com/josoriom/cipher/releases/latest/download/model.onnx"`.
 * - `matches`: How many matches to keep per file, best first. Default `3`.
 * - `ppmTolerance`: Precursor m/z tolerance in ppm. The larger of the ppm and Da tolerances is used. Default `20`.
 * - `mzTolerance`: Precursor m/z tolerance in Da. The larger of the ppm and Da tolerances is used. Default `0.005`.
 * - `rtRange`: Largest distance, in minutes, from the peak apex to the MS2 scan. Default `0.3`.
 * - `minRelativeIntensity`: MS2 peaks below this share of the base peak are removed before matching, from 0 to 1. Default `0.02`.
 * @see {@link https://phenological.github.io/quantion/?language=js#AnnotateFeaturesOptions | Documentation}
 */
export interface AnnotateFeaturesOptions {
  library?: string;
  model?: string;
  matches?: number;
  ppmTolerance?: number;
  mzTolerance?: number;
  rtRange?: number;
  minRelativeIntensity?: number;
}

/**
 * CompoundMatch · Results
 *
 * One library spectrum that matches a feature in one file.
 *
 * Fields:
 * - `label`: Name of the compound, from the library param the model names as label.
 * - `id`: Id of the spectrum in its database.
 * - `database`: Database the id comes from.
 * - `score`: Average of the model vector similarity and the cosine, from 0 to 1.
 * - `cosine`: Cosine similarity of the cleaned spectra, from 0 to 1.
 * - `mz`: m/z values of the library spectrum.
 * - `intensity`: Intensities of the library spectrum.
 * - `metadata`: The other params of the library spectrum, by name.
 * @see {@link https://phenological.github.io/quantion/?language=js#CompoundMatch | Documentation}
 */
export interface CompoundMatch {
  label: string;
  id: string;
  database: string;
  score: number;
  cosine: number;
  mz: Float64Array;
  intensity: Float64Array;
  metadata: Record<string, string>;
}

/**
 * FileAnnotation · Results
 *
 * What one file says about a feature in annotate_features.
 *
 * Fields:
 * - `file`: Position of the file in files, from 0.
 * - `scanIndex`: Index of the MS2 scan used, or none when the file has no peak or no MS2 scan close to it.
 * - `matches`: The best matches in this file, best first, at most matches of them.
 * @see {@link https://phenological.github.io/quantion/?language=js#FileAnnotation | Documentation}
 */
export interface FileAnnotation {
  file: number;
  scanIndex: number | null;
  matches: CompoundMatch[];
}

/**
 * FeatureAnnotation · Results
 *
 * The name found for one feature. The label is the compound that is the best match in the most files, and ties go to the higher score.
 *
 * Fields:
 * - `mz`: m/z of the feature.
 * - `rt`: Retention time of the feature, in minutes.
 * - `label`: Name of the compound, or none when no file has a match.
 * - `id`: Id of the compound in its database, from the agreeing file with the highest score.
 * - `database`: Database the id comes from.
 * - `frequency`: Share of the files whose best match is this compound, from 0 to 1.
 * - `score`: Median score over the files that agree on the label.
 * - `cosine`: Median cosine similarity over the files that agree on the label.
 * - `files`: What each file says about the feature, in the same order as files.
 * @see {@link https://phenological.github.io/quantion/?language=js#FeatureAnnotation | Documentation}
 */
export interface FeatureAnnotation {
  mz: number;
  rt: number;
  label: string | null;
  id: string | null;
  database: string | null;
  frequency: number;
  score: number | null;
  cosine: number | null;
  files: FileAnnotation[];
}
