export type BinaryInput = Uint8Array | ArrayBuffer;
export type IonInput = BinaryInput | URL | string;

export const TimeUnit = { MINUTES: "MINUTES", SECONDS: "SECONDS" } as const;
export type TimeUnit = (typeof TimeUnit)[keyof typeof TimeUnit];

export const PeakShape = { GAUSSIAN: "GAUSSIAN", EMG: "EMG" } as const;
export type PeakShape = (typeof PeakShape)[keyof typeof PeakShape];

export interface FromTo {
  from: number;
  to: number;
}

export interface Target {
  id?: string;
  mz: number;
  rt: number;
  range?: number;
}

export interface Transition {
  id?: string;
  precursor: number;
  product: number;
  rt: number;
  range: number;
}

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

export type ScanQuery = RtQuery | MzQuery;

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

export interface GetPeakOptions {
  range?: number;
  findPeaks?: FindPeaksOptions;
}

export interface FitPeakOptions {
  shape?: PeakShape;
  intensity?: number;
}

export interface DrawPeaksOptions {
  from?: number;
  to?: number;
  points?: number;
}

export interface CalculateBaselineOptions {
  lambda?: number;
  maxIterations?: number;
}

export interface CalculateEicOptions {
  from?: number;
  to?: number;
  ppmTolerance?: number;
  mzTolerance?: number;
  timeUnit?: TimeUnit;
}

export interface GridOptions {
  start?: number;
  end?: number;
  stepSize?: number;
}

export interface GroupingOptions {
  ppmTolerance?: number;
  mzTolerance?: number;
  rtTolerance?: number;
  minFrequency?: number;
}

export interface FindTargetsOptions {
  eic?: CalculateEicOptions;
  findPeaks?: FindPeaksOptions;
  threads?: number;
}

export interface FindTransitionsOptions {
  findPeaks?: FindPeaksOptions;
  threads?: number;
}

export interface FindFeaturesOptions {
  eic?: CalculateEicOptions;
  grid?: GridOptions;
  findPeaks?: FindPeaksOptions;
  threads?: number;
}

export interface MatchFeaturesOptions {
  eic?: CalculateEicOptions;
  grid?: GridOptions;
  grouping?: GroupingOptions;
  findPeaks?: FindPeaksOptions;
  threads?: number;
}

export interface GetScansOptions {
  level?: number;
  timeUnit?: TimeUnit;
}

export interface ParseIonOptions {
  maxCacheSize?: number;
}

export interface WriteIonOptions {
  out?: string | URL;
  overwrite?: boolean;
  level?: number;
  f32Compress?: boolean;
  headers?: Header[];
}

export interface WriteMzmlOptions {
  out?: string;
  overwrite?: boolean;
}

export interface GetIonImageOptions {
  tolerance?: number;
  level?: number;
  onProgress?: (fetched: number, total: number, heldBytes: number) => void;
}

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

export interface DataXY {
  x: Float64Array;
  y: Float64Array;
}

export interface NoiseLevel {
  width: number;
  intensity: number;
}

export interface PeakParameters {
  height: number;
  center: number;
  fwhm: number;
  tail: number;
  r2?: number;
}

export interface PredictedRt {
  rt: number;
  uncertainty: number;
}

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
