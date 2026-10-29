export type BinaryInput = Uint8Array | ArrayBuffer;
export type IonInput = BinaryInput | URL | string;

/**
 * TimeUnit · Enums
 *
 * Unit of the retention times you give to a function. Results are always in minutes.
 * @see {@link https://phenological.github.io/quantion/?language=js#TimeUnit | Documentation}
 */
export const TimeUnit = { MINUTES: "MINUTES", SECONDS: "SECONDS" } as const;
export type TimeUnit = (typeof TimeUnit)[keyof typeof TimeUnit];

/**
 * PeakShape · Enums
 *
 * Curve used to fit a peak.
 * @see {@link https://phenological.github.io/quantion/?language=js#PeakShape | Documentation}
 */
export const PeakShape = { GAUSSIAN: "GAUSSIAN", EMG: "EMG" } as const;
export type PeakShape = (typeof PeakShape)[keyof typeof PeakShape];

/**
 * FromTo · Inputs
 *
 * A range: retention-time windows and m/z ranges.
 * @see {@link https://phenological.github.io/quantion/?language=js#FromTo | Documentation}
 */
export interface FromTo {
  from: number;
  to: number;
}

/**
 * Target · Inputs
 *
 * One compound. range limits the search in find_targets; missing or 0 searches the whole EIC. predict_rts and calibrate do not use range.
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
 * One SRM transition.
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
 * One HTTP header sent with an upload in write_ion. Values are never printed. A line break, or Host, Content-Length or Transfer-Encoding, gives an invalid_argument error.
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
 * Which scans get_scans returns.
 * @see {@link https://phenological.github.io/quantion/?language=js#ScanQuery | Documentation}
 */
export type ScanQuery = RtQuery | MzQuery;

/**
 * FindPeaksOptions · Options
 *
 * Peak-picking settings. Every field is optional.
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
 * Settings for get_peak.
 * @see {@link https://phenological.github.io/quantion/?language=js#GetPeakOptions | Documentation}
 */
export interface GetPeakOptions {
  range?: number;
  findPeaks?: FindPeaksOptions;
}

/**
 * FitPeakOptions · Options
 *
 * Settings for fit_peak.
 * @see {@link https://phenological.github.io/quantion/?language=js#FitPeakOptions | Documentation}
 */
export interface FitPeakOptions {
  shape?: PeakShape;
  intensity?: number;
}

/**
 * DrawPeaksOptions · Options
 *
 * Settings for draw_peaks.
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
 * airPLS baseline settings for calculate_baseline.
 * @see {@link https://phenological.github.io/quantion/?language=js#CalculateBaselineOptions | Documentation}
 */
export interface CalculateBaselineOptions {
  lambda?: number;
  maxIterations?: number;
}

/**
 * CalculateEicOptions · Options
 *
 * Which scans are read and how wide the m/z window is. Used by calculate_eic, and as the eic group of find_targets, find_features and match_features. The wider of the ppm and absolute m/z windows is used.
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
 * The m/z grid that find_features and match_features search.
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
 * How match_features matches features across samples.
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
 * Settings for find_targets.
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
 * Settings for find_transitions.
 * @see {@link https://phenological.github.io/quantion/?language=js#FindTransitionsOptions | Documentation}
 */
export interface FindTransitionsOptions {
  findPeaks?: FindPeaksOptions;
  threads?: number;
}

/**
 * FindFeaturesOptions · Options
 *
 * Settings for find_features.
 * @see {@link https://phenological.github.io/quantion/?language=js#FindFeaturesOptions | Documentation}
 */
export interface FindFeaturesOptions {
  eic?: CalculateEicOptions;
  grid?: GridOptions;
  findPeaks?: FindPeaksOptions;
  threads?: number;
}

/**
 * MatchFeaturesOptions · Options
 *
 * Settings for match_features: the find_features settings plus grouping.
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
 * GetScansOptions · Options
 *
 * Settings for get_scans.
 * @see {@link https://phenological.github.io/quantion/?language=js#GetScansOptions | Documentation}
 */
export interface GetScansOptions {
  level?: number;
  timeUnit?: TimeUnit;
}

/**
 * ParseIonOptions · Options
 *
 * Settings for parse_ion.
 * @see {@link https://phenological.github.io/quantion/?language=js#ParseIonOptions | Documentation}
 */
export interface ParseIonOptions {
  maxCacheSize?: number;
}

/**
 * WriteIonOptions · Options
 *
 * Settings for write_ion.
 * @see {@link https://phenological.github.io/quantion/?language=js#WriteIonOptions | Documentation}
 */
export interface WriteIonOptions {
  out?: string | URL;
  overwrite?: boolean;
  level?: number;
  f32Compress?: boolean;
  headers?: Header[];
}

/**
 * WriteMzmlOptions · Options
 *
 * Settings for write_mzml.
 * @see {@link https://phenological.github.io/quantion/?language=js#WriteMzmlOptions | Documentation}
 */
export interface WriteMzmlOptions {
  out?: string;
  overwrite?: boolean;
}

/**
 * GetIonImageOptions · Options
 *
 * Settings for get_ion_image.
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
 * One chromatographic peak. No peak gives every field 0.
 * @see {@link https://phenological.github.io/quantion/?language=js#Peak | Documentation}
 */
export interface Peak {
  rt: number;
  from: number;
  to: number;
  intensity: number;
  integral: number;
  nPoints: number;
  noise: number;
  r2: number;
}

/**
 * Feature · Results
 *
 * A peak at an m/z. match_features adds frequency.
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
  noise: number;
  frequency?: number;
}

/**
 * DataXY · Results
 *
 * A trace: x = retention times, y = intensities. Returned by calculate_eic, taken by find_peaks.
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
 * @see {@link https://phenological.github.io/quantion/?language=js#NoiseLevel | Documentation}
 */
export interface NoiseLevel {
  width: number;
  intensity: number;
}

/**
 * PeakParameters · Results
 *
 * One peak curve: returned by fit_peak, or written by hand to simulate. draw_peaks takes a list of them. tail = 0 is a Gaussian, tail above 0 an EMG.
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
 * One predicted retention time from predict_rts.
 * @see {@link https://phenological.github.io/quantion/?language=js#PredictedRt | Documentation}
 */
export interface PredictedRt {
  rt: number;
  uncertainty: number;
}

/**
 * Scan · Results
 *
 * One scan from get_scans: its retention time, centroided m/z and intensity values, and its details.
 * @see {@link https://phenological.github.io/quantion/?language=js#Scan | Documentation}
 */
export interface Scan {
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
}

/**
 * IonImage · Results
 *
 * An imaging map of one m/z from get_ion_image.
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
