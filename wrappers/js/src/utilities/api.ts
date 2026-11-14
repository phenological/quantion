import type {
  AnnotationTables,
  Backend,
  EicSettings,
  FileHandle,
  PeakColumns,
  PositionSettings,
  ScanQuerySettings,
  TargetColumns,
  TransitionColumns,
} from "./backend";
import { SampleFile } from "./sampleFile";
import { Model } from "./model";
import { WarpModel } from "./warpModel";
import { toUint8 } from "./shared";
import { getIonSource, type IonSource } from "./ionSource";
import { QuantionError, invalidArgument } from "./errors";
import { uploadIon } from "./upload";
import {
  checkItems,
  checkKeys,
  FEATURE_NUMBER_KEYS,
  getRequiredNumber,
  getShapeCode,
  getTimeUnitCode,
  packHeaders,
  PEAK_KEYS,
  RANGE_KEYS,
  readBaselineOptions,
  readDeisotopeFeaturesOptions,
  readDrawPeaksOptions,
  readEicOptions,
  readFindFeaturesOptions,
  readFindPeaksOptions,
  readFindTargetsOptions,
  readFindTransitionsOptions,
  readFitPeakOptions,
  readGetPeakOptions,
  getAccessionNumbers,
  hasDefaultScanArrays,
  readGetScanMetadataOptions,
  readAnnotateFeaturesOptions,
  readGetScansOptions,
  readIonImageOptions,
  readMatchFeaturesOptions,
  readParseIonOptions,
  readWriteIonOptions,
  readWriteMzmlOptions,
  TARGET_KEYS,
  TRANSITION_KEYS,
  type Complete,
} from "./options";
import type {
  AnnotateFeaturesOptions,
  CompoundMatch,
  FeatureAnnotation,
  FileAnnotation,
  CalculateBaselineOptions,
  BinaryInput,
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
  GetScanMetadataOptions,
  GetScansOptions,
  IonImage,
  GetIonImageOptions,
  IonInput,
  DeisotopeFeaturesOptions,
  MatchFeaturesOptions,
  NoiseLevel,
  ParseIonOptions,
  Peak,
  PeakParameters,
  Scan,
  ScanParam,
  ScanQuery,
  Target,
  Transition,
  WriteIonOptions,
  WriteMzmlOptions,
} from "../types/types";

let selectedBackend: Backend | null = null;
let initPromise: Promise<void> | null = null;

export function setBackend(backend: Backend): void {
  selectedBackend = backend;
}

export function setInitPromise(promise: Promise<void>): void {
  initPromise = promise;
}

function getBackend(): Backend {
  if (!selectedBackend || !selectedBackend.ready) {
    throw new QuantionError(
      "internal",
      "quantion",
      "the library is not loaded; call init() first in the browser",
    );
  }
  return selectedBackend;
}

async function getBackendAsync(): Promise<Backend> {
  if (initPromise) await initPromise;
  return getBackend();
}

function getHandle(file: SampleFile, functionName: string): FileHandle {
  if (!(file instanceof SampleFile)) {
    throw invalidArgument(functionName, "file", "must be a SampleFile");
  }
  const handle = file._handle;
  if (!handle) {
    throw invalidArgument(
      functionName,
      "file",
      "is not open; it was disposed or never opened",
    );
  }
  return handle;
}

function getNumber(
  value: unknown,
  functionName: string,
  argument: string,
): number {
  if (typeof value !== "number") {
    throw invalidArgument(
      functionName,
      argument,
      `must be a number, got ${typeof value}`,
    );
  }
  return value;
}

function isFloat64Array(value: unknown): value is Float64Array {
  return Object.prototype.toString.call(value) === "[object Float64Array]";
}

function getTrace(
  x: Float64Array,
  y: Float64Array,
  functionName: string,
): [Float64Array, Float64Array] {
  if (!isFloat64Array(x)) {
    throw invalidArgument(functionName, "x", "must be a Float64Array");
  }
  if (!isFloat64Array(y)) {
    throw invalidArgument(functionName, "y", "must be a Float64Array");
  }
  if (x.length !== y.length) {
    throw invalidArgument(
      functionName,
      "y",
      `must have the same length as x (${x.length}), got ${y.length}`,
    );
  }
  return [x, y];
}

function getEicSettings(eic: Complete<CalculateEicOptions>): EicSettings {
  return {
    from: eic.from,
    to: eic.to,
    ppmTolerance: eic.ppmTolerance,
    mzTolerance: eic.mzTolerance,
    timeUnit: getTimeUnitCode(eic.timeUnit),
  };
}

function getTargetColumns(
  targets: Target[],
  functionName: string,
): TargetColumns {
  if (!Array.isArray(targets)) {
    throw invalidArgument(functionName, "targets", "must be an array");
  }
  checkItems(targets, TARGET_KEYS, functionName, "targets");
  return {
    rts: Float64Array.from(targets, (target, index) =>
      getRequiredNumber(target.rt, functionName, `targets[${index}].rt`),
    ),
    mzs: Float64Array.from(targets, (target, index) =>
      getRequiredNumber(target.mz, functionName, `targets[${index}].mz`),
    ),
    ranges: Float64Array.from(targets, (target, index) =>
      target.range === undefined
        ? NaN
        : getNumber(target.range, functionName, `targets[${index}].range`),
    ),
    ids: targets.map((target) => target.id ?? ""),
  };
}

function getTransitionColumns(
  targets: Transition[],
  functionName: string,
): TransitionColumns {
  if (!Array.isArray(targets)) {
    throw invalidArgument(functionName, "targets", "must be an array");
  }
  checkItems(targets, TRANSITION_KEYS, functionName, "targets");
  const read = (key: keyof Transition) =>
    Float64Array.from(targets, (target, index) =>
      getRequiredNumber(target[key], functionName, `targets[${index}].${key}`),
    );
  return {
    precursors: read("precursor"),
    products: read("product"),
    rts: read("rt"),
    ranges: read("range"),
    ids: targets.map((target) => target.id ?? ""),
  };
}

function openIon(
  backend: Backend,
  source: IonSource,
  cacheSize: number,
): FileHandle | Promise<FileHandle> {
  if (source.kind === "path")
    return backend.parseIonPath(source.path, cacheSize);
  if (source.kind === "url")
    return backend.parseIonRemote(source.url, cacheSize);
  return backend.parseIonBuffer(source.bytes, cacheSize);
}

/**
 * parseIon · Files
 *
 * Open an Ionic file (.ion) from a path, a URL or bytes. Paths and URLs are read lazily. Only the parts a query needs are read, so large or remote files open fast.
 *
 * **Warning:** The sample is freed by the library once it is no longer used, so dispose is not needed in normal use. When processing many samples, dispose each one after using it, so its memory is freed right away instead of piling up.
 *
 * @param source - Where the file is. It can be a local path, an http(s) URL, or the file's bytes.
 * @param options - Optional settings: `maxCacheSize` = 0. See {@link https://phenological.github.io/quantion/?language=js#ParseIonOptions | ParseIonOptions}.
 * @returns The opened sample (one file, i.e. one injection). Pass it to calculate_eic, get_scans and the other functions.
 * @see {@link https://phenological.github.io/quantion/?language=js#parse_ion | Documentation}
 */
export async function parseIon(
  source: IonInput,
  options?: ParseIonOptions,
): Promise<SampleFile> {
  const settings = readParseIonOptions(options, "parseIon");
  const ionSource = getIonSource(source, "parseIon");
  const backend = await getBackendAsync();
  const handle = await openIon(backend, ionSource, settings.maxCacheSize);
  return new SampleFile(handle, backend);
}

/**
 * parseMzml · Files
 *
 * Open an mzML file from a path or bytes. The whole file is loaded into memory. URLs are not allowed, so download the file first, or convert it to .ion with write_ion.
 *
 * **Warning:** The sample is freed by the library once it is no longer used, so dispose is not needed in normal use. When processing many samples, dispose each one after using it, so its memory is freed right away instead of piling up.
 *
 * @param source - Where the file is. It can be a local path or the file's bytes.
 * @returns The opened sample (one file, i.e. one injection).
 * @see {@link https://phenological.github.io/quantion/?language=js#parse_mzml | Documentation}
 */
export async function parseMzml(
  source: string | BinaryInput,
): Promise<SampleFile> {
  const backend = await getBackendAsync();
  if (typeof source === "string") {
    if (/^https?:/i.test(source)) {
      throw new QuantionError(
        "remote_file",
        "parseMzml",
        "this needs the whole file: open it from a path or bytes, not a URL",
      );
    }
    if (!backend.canReadPaths) {
      throw invalidArgument(
        "parseMzml",
        "source",
        "must be bytes in the browser; paths need Node",
      );
    }
    return new SampleFile(backend.parseMzmlPath(source), backend);
  }
  if (!(source instanceof Uint8Array) && !(source instanceof ArrayBuffer)) {
    throw invalidArgument("parseMzml", "source", "must be a path or bytes");
  }
  return new SampleFile(backend.parseMzmlBytes(toUint8(source)), backend);
}

/**
 * writeIon · Files
 *
 * Save a sample as an Ionic file (.ion), for example to convert mzML to .ion. With out, it writes a file to a path or uploads it to a URL with one PUT request. Without out, it returns the bytes.
 *
 * @param file - The sample to save, from parse_ion or parse_mzml.
 * @param options - Optional settings: `out` = undefined, `overwrite` = false, `level` = 12, `f32Compress` = false, `blockSize` = 8, `mzWindow` = 20, `headers` = []. See {@link https://phenological.github.io/quantion/?language=js#WriteIonOptions | WriteIonOptions}.
 * @returns The file's bytes when out is not given, or nothing when the file is written or uploaded.
 * @see {@link https://phenological.github.io/quantion/?language=js#write_ion | Documentation}
 */
export async function writeIon(
  file: SampleFile,
  options?: WriteIonOptions,
): Promise<Uint8Array | undefined> {
  const handle = getHandle(file, "writeIon");
  const settings = readWriteIonOptions(options, "writeIon");
  const isUrl = settings.out !== null && /^https?:/i.test(settings.out);
  if (settings.out !== null && !isUrl && !file._backend.canReadPaths) {
    throw invalidArgument(
      "writeIon",
      "out",
      "cannot be a path in the browser; use a URL or leave it out to get the bytes",
    );
  }
  const bytes = await file._backend.writeIon(
    handle,
    settings.out,
    settings.overwrite,
    settings.level,
    settings.f32Compress,
    settings.blockSize,
    settings.mzWindow,
    packHeaders(settings.headers),
  );
  if (!isUrl) return bytes;
  await uploadIon(
    settings.out as string,
    settings.headers,
    bytes as Uint8Array,
  );
  return undefined;
}

/**
 * writeMzml · Files
 *
 * Save a sample as mzML. With out, it writes a file to a path (URLs are not allowed). Without out, it returns the bytes.
 *
 * @param file - The sample to save, from parse_ion or parse_mzml.
 * @param options - Optional settings: `out` = undefined, `overwrite` = false. See {@link https://phenological.github.io/quantion/?language=js#WriteMzmlOptions | WriteMzmlOptions}.
 * @returns The file's bytes when out is not given, or nothing when the file is written.
 * @see {@link https://phenological.github.io/quantion/?language=js#write_mzml | Documentation}
 */
export async function writeMzml(
  file: SampleFile,
  options?: WriteMzmlOptions,
): Promise<Uint8Array | undefined> {
  const handle = getHandle(file, "writeMzml");
  const settings = readWriteMzmlOptions(options, "writeMzml");
  if (settings.out !== null && !file._backend.canReadPaths) {
    throw invalidArgument(
      "writeMzml",
      "out",
      "cannot be a path in the browser; leave it out to get the bytes",
    );
  }
  return file._backend.writeMzml(handle, settings.out, settings.overwrite);
}

/**
 * loadModel · Models
 *
 * Load an ONNX model from a path, a URL or bytes, and read the keys and values stored in it.
 *
 * @param source - Where the model is. It can be a local path, an http(s) URL, or the file's bytes.
 * @returns The model, with get_keys(), get_values() and get_value(key) to read what is stored in it.
 * @see {@link https://phenological.github.io/quantion/?language=js#load_model | Documentation}
 */
export async function loadModel(source: IonInput): Promise<Model> {
  const modelSource = getIonSource(source, "loadModel");
  const backend = await getBackendAsync();
  if (modelSource.kind === "path") {
    return new Model(backend.loadModelPath(modelSource.path), backend);
  }
  const bytes =
    modelSource.kind === "url"
      ? await fetchModel(modelSource.url)
      : modelSource.bytes;
  return new Model(backend.loadModelBytes(bytes), backend);
}

async function fetchModel(url: URL): Promise<Uint8Array> {
  const response = await fetch(url).catch((error: Error) => {
    throw new QuantionError(
      "parse_failed",
      "loadModel",
      `the file could not be read: GET ${url} failed: ${error.message}`,
    );
  });
  if (!response.ok) {
    throw new QuantionError(
      "parse_failed",
      "loadModel",
      `the file could not be read: GET ${url} answered ${response.status}`,
    );
  }
  return new Uint8Array(await response.arrayBuffer());
}

/**
 * loadWarp · Models
 *
 * When retention times shift between batches or within a batch, some peaks are easy to miss, e.g. low-intensity peaks. A deepwarp model predicts how each LC-MS run shifts, so peak detection finds the right peaks. Make a retention-time warp model from a loaded model. Use it with calibrate and predict_rts to correct rt shifts between runs.
 *
 * **Warning:** Works with any ONNX model that has the right inputs, outputs and keys. See them in deepwarp: https://github.com/josoriom/deepwarp#inputs-outputs-and-keys
 *
 * @param model - A model from load_model that holds a warp.
 * @returns The warp model.
 * @see {@link https://phenological.github.io/quantion/?language=js#load_warp | Documentation}
 */
export function loadWarp(model: Model): WarpModel {
  if (!(model instanceof Model) || !model._handle) {
    throw invalidArgument(
      "loadWarp",
      "model",
      "must be an open Model from loadModel",
    );
  }
  return new WarpModel(
    model._backend.loadWarp(model._handle),
    model._backend,
    model._getMetadata("loadWarp"),
  );
}

const POSITION_ANY = 0;
const POSITION_RANGE = 1;
const POSITION_CLOSEST = 2;
const SCAN_QUERY_KEYS = ["rt", "mz"];

function getPosition(value: unknown, key: string): PositionSettings {
  if (value === undefined) return { kind: POSITION_ANY, from: NaN, to: NaN };
  if (typeof value === "number") {
    return { kind: POSITION_CLOSEST, from: value, to: NaN };
  }
  const fromTo = value as FromTo;
  if (fromTo === null || typeof fromTo !== "object") {
    throw invalidArgument("getScans", key, "must be a number or { from, to }");
  }
  checkKeys(
    fromTo as unknown as Record<string, unknown>,
    RANGE_KEYS,
    "getScans",
    `${key}.`,
  );
  return {
    kind: POSITION_RANGE,
    from: getRequiredNumber(fromTo.from, "getScans", `${key}.from`),
    to: getRequiredNumber(fromTo.to, "getScans", `${key}.to`),
  };
}

function getScanQuery(query: ScanQuery): ScanQuerySettings {
  const given = query as unknown as Record<string, unknown>;
  if (given === null || typeof given !== "object") {
    throw invalidArgument("getScans", "query", "must be { rt }, { mz } or { rt, mz }");
  }
  checkKeys(given, SCAN_QUERY_KEYS, "getScans", "");
  if (given.rt === undefined && given.mz === undefined) {
    throw invalidArgument("getScans", "rt", "or mz must be given");
  }
  return { rt: getPosition(given.rt, "rt"), mz: getPosition(given.mz, "mz") };
}

/**
 * getScans · Files
 *
 * Read scans (spectra) by retention time, by precursor m/z, or by both. A range returns every scan inside it. One value returns the closest scan, as a list of one.
 *
 * @param file - The sample to read, from parse_ion or parse_mzml.
 * @param query.rt - Retention time of the scans, as a range {from, to} or as one value for the closest scan. Optional when mz is given. Each FromTo has `from`, `to`. See {@link https://phenological.github.io/quantion/?language=js#FromTo | FromTo}.
 *   Example: `const window: FromTo = { from: 1.0, to: 2.5 };`
 * @param query.mz - Precursor m/z of the scans, as a range {from, to} or as one value for the closest scan. Give rt, mz or both. Only MS2 scans have a precursor m/z, so set level to 2 (or 0 for every level) when you give mz. Each FromTo has `from`, `to`. See {@link https://phenological.github.io/quantion/?language=js#FromTo | FromTo}.
 *   Example: `const window: FromTo = { from: 1.0, to: 2.5 };`
 * @param options - Optional settings: `level` = 1, `timeUnit` = "MINUTES", `arrays` = { key: "mz", accession: "MS:1000514" }, { key: "intensity", accession: "MS:1000515" }. See {@link https://phenological.github.io/quantion/?language=js#GetScansOptions | GetScansOptions}.
 * @returns The scans found, in rt order. An empty list means no scan matched, so check rt, mz and level.
 * @see {@link https://phenological.github.io/quantion/?language=js#get_scans | Documentation}
 */
export async function getScans(
  file: SampleFile,
  query: ScanQuery,
  options?: GetScansOptions,
): Promise<Scan[]> {
  const handle = getHandle(file, "getScans");
  const settings = readGetScansOptions(options, "getScans");
  const scanQuery = getScanQuery(query);
  if (!hasDefaultScanArrays(settings.arrays)) {
    return file._backend.getScansWithArrays(
      handle,
      scanQuery,
      settings.level,
      getTimeUnitCode(settings.timeUnit),
      getAccessionNumbers(settings.arrays),
      settings.arrays.map((array) => array.key),
    );
  }
  return file._backend.getScans(
    handle,
    scanQuery,
    settings.level,
    getTimeUnitCode(settings.timeUnit),
  );
}

function getScanIndex(index: unknown): number {
  if (typeof index !== "number" || !Number.isSafeInteger(index) || index < 0) {
    throw invalidArgument(
      "getScanMetadata",
      "index",
      `must be a whole number 0 or more, got ${String(index)}`,
    );
  }
  return index;
}

/**
 * getScanMetadata · Metadata
 *
 * Read the details stored for one scan (attributes, cvParams and userParams), one row each, parents before children. Find the scan with get_scans, then pass its index.
 *
 * @param file - The sample to read, from parse_ion or parse_mzml.
 * @param index - Position of the scan in the file, from 0. Use the index of a scan from get_scans.
 * @param options - Optional settings: `tags` = []. See {@link https://phenological.github.io/quantion/?language=js#GetScanMetadataOptions | GetScanMetadataOptions}.
 * @returns One row per attribute or param, in the order of the mzML tree. Values are text. A field the file does not give is missing (None in Rust and Python, null in JavaScript, NA in R).
 * @see {@link https://phenological.github.io/quantion/?language=js#get_scan_metadata | Documentation}
 */
export async function getScanMetadata(
  file: SampleFile,
  index: number,
  options?: GetScanMetadataOptions,
): Promise<ScanParam[]> {
  const handle = getHandle(file, "getScanMetadata");
  const position = getScanIndex(index);
  const settings = readGetScanMetadataOptions(options, "getScanMetadata");
  return file._backend.getScanMetadata(handle, position, settings.tags);
}

/**
 * calculateEic · Files
 *
 * Calculate the extracted ion chromatogram (EIC, also called XIC) of one m/z. For each MS1 scan, the intensities within the m/z window are summed. Use it to get the chromatogram before find_peaks or get_peak.
 *
 * @param file - The sample to read, from parse_ion or parse_mzml.
 * @param mz - The m/z of the ion to extract (Da), not the neutral mass of the compound. For example, for the protonated molecule [M+H]+ in positive mode, use M + 1.00728, and for the deprotonated molecule [M−H]− in negative mode, use M − 1.00728, where M is the monoisotopic mass.
 * @param options - Optional settings: `from` = 0, `to` = Infinity, `ppmTolerance` = 20, `mzTolerance` = 0.005, `timeUnit` = "MINUTES". See {@link https://phenological.github.io/quantion/?language=js#CalculateEicOptions | CalculateEicOptions}.
 * @returns The chromatogram, with x as the retention times in minutes and y as the summed intensities.
 * @see {@link https://phenological.github.io/quantion/?language=js#calculate_eic | Documentation}
 */
export async function calculateEic(
  file: SampleFile,
  mz: number,
  options?: CalculateEicOptions,
): Promise<DataXY> {
  const handle = getHandle(file, "calculateEic");
  const eic = readEicOptions(options, "calculateEic", "");
  return file._backend.calculateEic(
    handle,
    getNumber(mz, "calculateEic", "mz"),
    getEicSettings(eic),
  );
}

/**
 * getIonImage · Files
 *
 * Build an imaging map of one m/z for mass spectrometry imaging, with the summed intensity at each x, y position.
 *
 * **Warning:** Only Ionic files (.ion). The sample must come from parse_ion; a sample from parse_mzml gives an invalid_argument error.
 *
 * @param file - The sample to read. It must come from parse_ion.
 * @param mz - The m/z of the ion to map (Da), not the neutral mass of the compound. For example, for the protonated molecule [M+H]+ in positive mode, use M + 1.00728, and for the deprotonated molecule [M−H]− in negative mode, use M − 1.00728, where M is the monoisotopic mass.
 * @param options - Optional settings: `tolerance` = 0.125, `level` = 1, `onProgress` = undefined. See {@link https://phenological.github.io/quantion/?language=js#GetIonImageOptions | GetIonImageOptions}.
 * @returns The image, with one value per pixel, plus its size and position.
 * @see {@link https://phenological.github.io/quantion/?language=js#get_ion_image | Documentation}
 */
export async function getIonImage(
  file: SampleFile,
  mz: number,
  options?: GetIonImageOptions,
): Promise<IonImage> {
  const handle = getHandle(file, "getIonImage");
  const settings = readIonImageOptions(options, "getIonImage");
  return file._backend.getIonImage(
    handle,
    getNumber(mz, "getIonImage", "mz"),
    settings.tolerance,
    settings.level,
    settings.onProgress,
  );
}

/**
 * findPeaks · Traces
 *
 * Find every peak (peak picking) in one trace or signal, such as a chromatogram (TIC, BPC or EIC). For one known peak at an rt, use get_peak. To calculate EIC use calculate_eic.
 *
 * @param x - Retention times in minutes, ascending, at least 3 points. For a chromatogram, its x.
 * @param y - Intensities, same length as x. For a chromatogram, its y.
 * @param options - Optional settings: `minIntegral` = off, `minIntensity` = 500, `minPeakWidthPoints` = 3, `noise` = off, `autoNoise` = true, `autoBaseline` = true, `lambda` = 20, `maxIterations` = 4, `allowOverlap` = false, `minSnr` = 2, `minR2` = 0, `shape` = "EMG", `kernelSize` = 0. See {@link https://phenological.github.io/quantion/?language=js#FindPeaksOptions | FindPeaksOptions}.
 * @returns Every peak that passes the filters specified in the function options: the peak table, one row per peak.
 * @see {@link https://phenological.github.io/quantion/?language=js#find_peaks | Documentation}
 */
export function findPeaks(
  x: Float64Array,
  y: Float64Array,
  options?: FindPeaksOptions,
): Peak[] {
  getTrace(x, y, "findPeaks");
  const settings = readFindPeaksOptions(options, "findPeaks", "");
  return getBackend().findPeaks(x, y, settings);
}

/**
 * getPeak · Traces
 *
 * Find the peak closest to rt. If range is given, only peaks within rt ± range are searched. If it is not, the whole trace is searched, and the peak is chosen by its size and its distance from rt.
 *
 * @param x - Retention times, ascending. At least 3 points.
 * @param y - Intensities, same length as x.
 * @param rt - Expected retention time of the target peak, in minutes.
 * @param options - Optional settings: `range` = 0, `findPeaks`. See {@link https://phenological.github.io/quantion/?language=js#GetPeakOptions | GetPeakOptions}.
 * @returns The chosen peak, or an empty peak (every field is 0) when no peak is found.
 * @see {@link https://phenological.github.io/quantion/?language=js#get_peak | Documentation}
 */
export function getPeak(
  x: Float64Array,
  y: Float64Array,
  rt: number,
  options?: GetPeakOptions,
): Peak {
  getTrace(x, y, "getPeak");
  const settings = readGetPeakOptions(options, "getPeak");
  return getBackend().getPeak(
    x,
    y,
    getNumber(rt, "getPeak", "rt"),
    settings.range,
    settings.findPeaks,
  );
}

/**
 * fitPeak · Traces
 *
 * Fit a Gaussian or EMG curve to the target peak at rt. The function finds the peak within rt ± 0.05 min, then fits the points between its start and end to get its width (fwhm) and tailing (tail). The peak needs at least 5 points.
 *
 * @param x - Retention times, ascending. At least 3 points.
 * @param y - Intensities, same length as x.
 * @param rt - Retention time of the peak to fit, in minutes. The peak must be within ± 0.05 min of it.
 * @param options - Optional settings: `shape` = "EMG", `intensity` = 0. See {@link https://phenological.github.io/quantion/?language=js#FitPeakOptions | FitPeakOptions}.
 * @returns The fitted parameters (height, center, fwhm, tail, r2) for the target peak, or none if no peak is found near rt or the fit fails.
 * @see {@link https://phenological.github.io/quantion/?language=js#fit_peak | Documentation}
 */
export function fitPeak(
  x: Float64Array,
  y: Float64Array,
  rt: number,
  options?: FitPeakOptions,
): PeakParameters | null {
  getTrace(x, y, "fitPeak");
  const settings = readFitPeakOptions(options, "fitPeak");
  return getBackend().fitPeak(
    x,
    y,
    getNumber(rt, "fitPeak", "rt"),
    getShapeCode(settings.shape),
    settings.intensity,
  );
}

/**
 * drawPeaks · Traces
 *
 * Simulate a chromatogram (a synthetic trace) from peak curves (height, center, fwhm, tail). Overlapping peaks add up. Useful for tests, no data needed.
 *
 * @param params - The peaks to draw, one PeakParameters per peak. Each PeakParameters has `height`, `center`, `fwhm`, `tail`, `r2`. See {@link https://phenological.github.io/quantion/?language=js#PeakParameters | PeakParameters}.
 *   Example: `const peak: PeakParameters = { height: 9800, center: 2.38, fwhm: 0.021, tail: 0.006 };`
 * @param options - Optional settings: `from` = 0, `to` = 0, `points` = 1000. See {@link https://phenological.github.io/quantion/?language=js#DrawPeaksOptions | DrawPeaksOptions}.
 * @returns The simulated chromatogram: x = retention times in minutes, y = summed intensities.
 * @see {@link https://phenological.github.io/quantion/?language=js#draw_peaks | Documentation}
 */
export function drawPeaks(
  params: PeakParameters[],
  options?: DrawPeaksOptions,
): DataXY {
  if (!Array.isArray(params)) {
    throw invalidArgument(
      "drawPeaks",
      "params",
      "must be an array of PeakParameters",
    );
  }
  checkItems(params, PEAK_KEYS, "drawPeaks", "params");
  const settings = readDrawPeaksOptions(options, "drawPeaks");
  const read = (key: keyof PeakParameters) =>
    Float64Array.from(params, (peak, index) =>
      getRequiredNumber(peak[key], "drawPeaks", `params[${index}].${key}`),
    );
  const peaks: PeakColumns = {
    heights: read("height"),
    centers: read("center"),
    fwhms: read("fwhm"),
    tails: read("tail"),
  };
  return getBackend().drawPeaks(
    peaks,
    settings.from,
    settings.to,
    settings.points,
  );
}

/**
 * findNoiseLevel · Traces
 *
 * Estimate the noise level of a chromatogram or any signal: how noisy it is. Small, narrow peaks are grouped together, and their maximum intensity and width (in number of points) are reported as the noise.
 *
 * @param y - Intensities, for example the y of a chromatogram.
 * @returns The noise intensity and the width (in points) of the noise regions it was estimated from.
 * @see {@link https://phenological.github.io/quantion/?language=js#find_noise_level | Documentation}
 */
export function findNoiseLevel(y: Float64Array | Float32Array): NoiseLevel {
  const isFloat32Array =
    Object.prototype.toString.call(y) === "[object Float32Array]";
  if (!isFloat64Array(y) && !isFloat32Array) {
    throw invalidArgument(
      "findNoiseLevel",
      "y",
      "must be a Float64Array or Float32Array",
    );
  }
  const values = isFloat32Array ? (y as Float32Array) : new Float32Array(y);
  return getBackend().findNoiseLevel(values);
}

/**
 * calculateBaseline · Traces
 *
 * Estimate the baseline (slow background drift) of the input chromatogram or signal using airPLS (Zhang et al., 2010).
 *
 * @param y - Intensities, for example the y of a chromatogram.
 * @param options - Optional settings: `lambda` = 20, `maxIterations` = 4. See {@link https://phenological.github.io/quantion/?language=js#CalculateBaselineOptions | CalculateBaselineOptions}.
 * @returns The baseline: one value per point of y.
 * @see {@link https://phenological.github.io/quantion/?language=js#calculate_baseline | Documentation}
 */
export function calculateBaseline(
  y: Float64Array,
  options?: CalculateBaselineOptions,
): Float64Array {
  if (!isFloat64Array(y)) {
    throw invalidArgument("calculateBaseline", "y", "must be a Float64Array");
  }
  const settings = readBaselineOptions(options, "calculateBaseline");
  return getBackend().calculateBaseline(
    y,
    settings.lambda,
    settings.maxIterations,
  );
}

/**
 * findTargets · Targeted
 *
 * Quantify a list of known compounds (targets) in one file. For each target, it builds the EIC at its mz and finds its peak near rt ± range. Use it for a targeted panel.
 *
 * @param file - The sample to read, from parse_ion or parse_mzml.
 * @param targets - The compounds to look for, each with id, mz, rt and an optional range. A target without range searches its whole EIC. Each Target has `id`, `mz`, `rt`, `range`. See {@link https://phenological.github.io/quantion/?language=js#Target | Target}.
 *   Example: `const target: Target = { id: "compound_1", mz: 260.103, rt: 2.383, range: 0.05 };`
 * @param options - Optional settings: `eic`, `findPeaks`, `threads` = 1. See {@link https://phenological.github.io/quantion/?language=js#FindTargetsOptions | FindTargetsOptions}.
 * @returns One peak per target, in the same order as targets. A target with no peak gets an empty peak (every field is 0).
 * @see {@link https://phenological.github.io/quantion/?language=js#find_targets | Documentation}
 */
export async function findTargets(
  file: SampleFile,
  targets: Target[],
  options?: FindTargetsOptions,
): Promise<Peak[]> {
  const handle = getHandle(file, "findTargets");
  const settings = readFindTargetsOptions(options, "findTargets");
  return file._backend.findTargets(
    handle,
    getTargetColumns(targets, "findTargets"),
    getEicSettings(settings.eic),
    settings.findPeaks,
    settings.threads,
  );
}

/**
 * findTransitions · Targeted
 *
 * Quantify SRM or MRM transitions in one file. For each transition, it takes the stored chromatogram whose precursor (Q1) and product (Q3) are both within 0.005 m/z, and finds its peak near rt ± range.
 *
 * **Warning:** Needs the whole file. A sample opened from a URL gives a remote_file error; open it from a path or bytes.
 *
 * @param file - The sample to read, from parse_ion or parse_mzml.
 * @param targets - The transitions to look for, each with id, precursor, product, rt and range. Each Transition has `id`, `precursor`, `product`, `rt`, `range`. See {@link https://phenological.github.io/quantion/?language=js#Transition | Transition}.
 *   Example: `const transition: Transition = { id: "compound_1", precursor: 89.04768, product: 89.04768, rt: 2.385, range: 0.1 };`
 * @param options - Optional settings: `findPeaks`, `threads` = 1. See {@link https://phenological.github.io/quantion/?language=js#FindTransitionsOptions | FindTransitionsOptions}.
 * @returns One peak per transition, in the same order. A transition with no matching chromatogram gets an empty peak (every field is 0).
 * @see {@link https://phenological.github.io/quantion/?language=js#find_transitions | Documentation}
 */
export function findTransitions(
  file: SampleFile,
  targets: Transition[],
  options?: FindTransitionsOptions,
): Peak[] {
  const handle = getHandle(file, "findTransitions");
  const settings = readFindTransitionsOptions(options, "findTransitions");
  return file._backend.findTransitions(
    handle,
    getTransitionColumns(targets, "findTransitions"),
    settings.findPeaks,
    settings.threads,
  );
}

/**
 * findFeatures · Untargeted
 *
 * Find every feature in one file without a target list (untargeted). It scans an m/z grid, builds the EIC of each grid m/z and finds its peaks.
 *
 * @param file - The sample to read, from parse_ion or parse_mzml.
 * @param options - Optional settings: `eic`, `grid`, `findPeaks`, `threads` = 1. See {@link https://phenological.github.io/quantion/?language=js#FindFeaturesOptions | FindFeaturesOptions}.
 * @returns Every feature found (the feature table), without frequency.
 * @see {@link https://phenological.github.io/quantion/?language=js#find_features | Documentation}
 */
export async function findFeatures(
  file: SampleFile,
  options?: FindFeaturesOptions,
): Promise<Feature[]> {
  const handle = getHandle(file, "findFeatures");
  const settings = readFindFeaturesOptions(options, "findFeatures");
  if (!file._backend.findFeatures) {
    throw new QuantionError(
      "invalid_argument",
      "findFeatures",
      "is not available in the browser build",
    );
  }
  return file._backend.findFeatures(
    handle,
    getEicSettings(settings.eic),
    settings.grid,
    settings.findPeaks,
    settings.threads,
  );
}

/**
 * matchFeatures · Untargeted
 *
 * Find the features in every file of a folder and match them across samples (untargeted). The values of each feature are medians over the samples where it was found. Instead of aligning retention times, it groups features with close m/z and rt. Small rt differences between runs are covered by rt_tolerance.
 *
 * @param directory - Folder with the Ionic (.ion) or mzML files, one file per sample.
 * @param options - Optional settings: `eic`, `grid`, `grouping`, `findPeaks`, `threads` = 1. See {@link https://phenological.github.io/quantion/?language=js#MatchFeaturesOptions | MatchFeaturesOptions}.
 * @returns One feature per matched feature (the feature table). frequency is the fraction of samples where it was found, from 0 to 1.
 * @see {@link https://phenological.github.io/quantion/?language=js#match_features | Documentation}
 */
export function matchFeatures(
  directory: string,
  options?: MatchFeaturesOptions,
): Feature[] {
  const backend = getBackend();
  if (typeof directory !== "string") {
    throw invalidArgument("matchFeatures", "directory", "must be a path");
  }
  const settings = readMatchFeaturesOptions(options, "matchFeatures");
  if (!backend.matchFeatures) {
    throw new QuantionError(
      "invalid_argument",
      "matchFeatures",
      "is not available in the browser build",
    );
  }
  return backend.matchFeatures(
    directory,
    getEicSettings(settings.eic),
    settings.grid,
    settings.grouping,
    settings.findPeaks,
    settings.threads,
  );
}

function getFeatureValues(
  features: Feature[],
  functionName: string,
): Float64Array {
  if (!Array.isArray(features)) {
    throw invalidArgument(functionName, "features", "must be an array");
  }
  checkItems(
    features,
    [...FEATURE_NUMBER_KEYS, "frequency"],
    functionName,
    "features",
  );
  const size = FEATURE_NUMBER_KEYS.length + 1;
  const values = new Float64Array(features.length * size);
  features.forEach((feature, index) => {
    const start = index * size;
    FEATURE_NUMBER_KEYS.forEach((key, position) => {
      values[start + position] = getRequiredNumber(
        feature[key],
        functionName,
        `features[${index}].${key}`,
      );
    });
    values[start + size - 1] =
      feature.frequency === undefined
        ? NaN
        : getRequiredNumber(
            feature.frequency,
            functionName,
            `features[${index}].frequency`,
          );
  });
  return values;
}

/**
 * deisotopeFeatures · Untargeted
 *
 * Remove the 13C isotope peaks from a feature list and keep the monoisotopic peak (M) of each compound. A feature is an isotope when it is 1.00335/charge above the previous peak (charge 1 or 2), has the same rt as M, and is not too intense (M+1 at most M × mz × charge / 12 × 0.0108157). Run it after find_features or match_features, and before annotate_features.
 *
 * @param features - The features to clean, from find_features or match_features.
 * @param options - Optional settings: `rtTolerance` = 0.02, `mzTolerance` = 0.005. See {@link https://phenological.github.io/quantion/?language=js#DeisotopeFeaturesOptions | DeisotopeFeaturesOptions}.
 * @returns The features that are not isotopes, sorted by m/z.
 * @see {@link https://phenological.github.io/quantion/?language=js#deisotope_features | Documentation}
 */
export function deisotopeFeatures(
  features: Feature[],
  options?: DeisotopeFeaturesOptions,
): Feature[] {
  const values = getFeatureValues(features, "deisotopeFeatures");
  const settings = readDeisotopeFeaturesOptions(options, "deisotopeFeatures");
  return getBackend().deisotopeFeatures(
    values,
    features.length,
    settings.rtTolerance,
    settings.mzTolerance,
  );
}

const ANNOTATION_FEATURE_KEYS = [
  "mz",
  "rt",
  "from",
  "to",
  "intensity",
  "integral",
  "nPoints",
  "frequency",
];

function getNumberOrNull(value: unknown): number | null {
  return typeof value === "number" && !Number.isNaN(value) ? value : null;
}

function getCompoundMatch(row: Record<string, unknown>): CompoundMatch {
  const names = row.metadataNames ? String(row.metadataNames).split("\x1f") : [];
  const values = row.metadataNames ? String(row.metadataValues).split("\x1f") : [];
  return {
    label: row.label as string,
    id: row.id as string,
    database: row.database as string,
    score: row.score as number,
    cosine: row.cosine as number,
    mz: row.mz as Float64Array,
    intensity: row.intensity as Float64Array,
    metadata: Object.fromEntries(names.map((name, index) => [name, values[index]])),
  };
}

function getAnnotations(tables: AnnotationTables): FeatureAnnotation[] {
  const matches = new Map<string, CompoundMatch[]>();
  for (const row of tables.matches) {
    const key = `${row.feature}:${row.file}`;
    const list = matches.get(key) ?? [];
    list.push(getCompoundMatch(row));
    matches.set(key, list);
  }
  const files = new Map<number, FileAnnotation[]>();
  for (const row of tables.files) {
    const feature = row.feature as number;
    const list = files.get(feature) ?? [];
    list.push({
      file: row.file as number,
      scanIndex: getNumberOrNull(row.scanIndex),
      matches: matches.get(`${feature}:${row.file}`) ?? [],
    });
    files.set(feature, list);
  }
  return tables.features.map((row, position) => ({
    mz: row.mz as number,
    rt: row.rt as number,
    label: row.label as string | null,
    id: row.id as string | null,
    database: row.database as string | null,
    frequency: row.frequency as number,
    score: getNumberOrNull(row.score),
    cosine: getNumberOrNull(row.cosine),
    files: files.get(position) ?? [],
  }));
}

/**
 * annotateFeatures · Untargeted
 *
 * Name features (annotation) by matching their MS2 spectra against a spectral library. For each feature and sample, it takes the positive MS2 scan closest to the peak apex, whose precursor matches the feature m/z and whose rt is within rt_range. A model turns the scan into a vector, which is scored against the library spectra with the same precursor m/z. By default, the library and the model come from the josoriom/cipher releases.
 *
 * **Warning:** This is an experimental feature.
 *
 * @param files - The samples, from parse_ion or parse_mzml. It must not be empty.
 * @param features - The features to name, each with mz and rt. The output of find_features and match_features works as is. An empty list gives an empty result.
 * @param options - Optional settings: `library` = "https://github.com/josoriom/cipher/releases/latest/download/library_positive.ion", `model` = "https://github.com/josoriom/cipher/releases/latest/download/model.onnx", `matches` = 3, `ppmTolerance` = 20, `mzTolerance` = 0.005, `rtRange` = 0.3, `minRelativeIntensity` = 0.02. See {@link https://phenological.github.io/quantion/?language=js#AnnotateFeaturesOptions | AnnotateFeaturesOptions}.
 * @returns One annotation per feature, in the same order as features.
 * @see {@link https://phenological.github.io/quantion/?language=js#annotate_features | Documentation}
 */
export async function annotateFeatures(
  files: SampleFile[],
  features: Array<{ mz: number; rt: number }>,
  options?: AnnotateFeaturesOptions,
): Promise<FeatureAnnotation[]> {
  const functionName = "annotateFeatures";
  const settings = readAnnotateFeaturesOptions(options, functionName);
  if (!Array.isArray(files)) {
    throw invalidArgument(functionName, "files", "must be an array of samples");
  }
  if (!Array.isArray(features)) {
    throw invalidArgument(functionName, "features", "must be an array of features");
  }
  checkItems(features, ANNOTATION_FEATURE_KEYS, functionName, "features");
  const handles = files.map((file) => getHandle(file, functionName));
  const mzs = Float64Array.from(features, (feature, index) =>
    getRequiredNumber(feature.mz, functionName, `features[${index}].mz`),
  );
  const rts = Float64Array.from(features, (feature, index) =>
    getRequiredNumber(feature.rt, functionName, `features[${index}].rt`),
  );
  const backend = files[0]?._backend ?? (await getBackendAsync());
  if (features.length === 0) {
    await backend.annotateFeatures(null, null, handles, mzs, rts, settings);
    return [];
  }
  const library = await parseIon(settings.library);
  const model = await loadModel(settings.model);
  try {
    const tables = await backend.annotateFeatures(
      library._handle,
      model._handle,
      handles,
      mzs,
      rts,
      settings,
    );
    return getAnnotations(tables);
  } finally {
    library.dispose();
    model.dispose();
  }
}

