import type {
  Backend,
  EicSettings,
  FileHandle,
  PeakColumns,
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
  getShapeCode,
  getTimeUnitCode,
  packHeaders,
  readBaselineOptions,
  readDrawPeaksOptions,
  readEicOptions,
  readFindFeaturesOptions,
  readFindPeaksOptions,
  readFindTargetsOptions,
  readFindTransitionsOptions,
  readFitPeakOptions,
  readGetPeakOptions,
  readGetScansOptions,
  readIonImageOptions,
  readMatchFeaturesOptions,
  readParseIonOptions,
  readWriteIonOptions,
  readWriteMzmlOptions,
  type Complete,
} from "./options";
import type {
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
  GetScansOptions,
  IonImage,
  GetIonImageOptions,
  IonInput,
  MatchFeaturesOptions,
  NoiseLevel,
  ParseIonOptions,
  Peak,
  PeakParameters,
  Scan,
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
  return {
    rts: Float64Array.from(targets, (target, index) =>
      getNumber(target.rt, functionName, `targets[${index}].rt`),
    ),
    mzs: Float64Array.from(targets, (target, index) =>
      getNumber(target.mz, functionName, `targets[${index}].mz`),
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
  const read = (key: keyof Transition) =>
    Float64Array.from(targets, (target, index) =>
      getNumber(target[key], functionName, `targets[${index}].${key}`),
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

export async function parseIon(
  source: IonInput,
  options?: ParseIonOptions,
): Promise<SampleFile> {
  const settings = readParseIonOptions(options, "parse_ion");
  const ionSource = getIonSource(source, "parse_ion");
  const backend = await getBackendAsync();
  const handle = await openIon(backend, ionSource, settings.maxCacheSize);
  return new SampleFile(handle, backend);
}

export async function parseMzml(
  source: string | BinaryInput,
): Promise<SampleFile> {
  const backend = await getBackendAsync();
  if (typeof source === "string") {
    if (/^https?:/i.test(source)) {
      throw new QuantionError(
        "remote_file",
        "parse_mzml",
        "this needs the whole file: open it from a path or bytes, not a URL",
      );
    }
    if (!backend.canReadPaths) {
      throw invalidArgument(
        "parse_mzml",
        "source",
        "must be bytes in the browser; paths need Node",
      );
    }
    return new SampleFile(backend.parseMzmlPath(source), backend);
  }
  if (!(source instanceof Uint8Array) && !(source instanceof ArrayBuffer)) {
    throw invalidArgument("parse_mzml", "source", "must be a path or bytes");
  }
  return new SampleFile(backend.parseMzmlBytes(toUint8(source)), backend);
}

export async function writeIon(
  file: SampleFile,
  options?: WriteIonOptions,
): Promise<Uint8Array | undefined> {
  const handle = getHandle(file, "write_ion");
  const settings = readWriteIonOptions(options, "write_ion");
  const isUrl = settings.out !== null && /^https?:/i.test(settings.out);
  if (settings.out !== null && !isUrl && !file._backend.canReadPaths) {
    throw invalidArgument(
      "write_ion",
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

export async function writeMzml(
  file: SampleFile,
  options?: WriteMzmlOptions,
): Promise<Uint8Array | undefined> {
  const handle = getHandle(file, "write_mzml");
  const settings = readWriteMzmlOptions(options, "write_mzml");
  if (settings.out !== null && !file._backend.canReadPaths) {
    throw invalidArgument(
      "write_mzml",
      "out",
      "cannot be a path in the browser; leave it out to get the bytes",
    );
  }
  return file._backend.writeMzml(handle, settings.out, settings.overwrite);
}

export async function loadModel(source: IonInput): Promise<Model> {
  const modelSource = getIonSource(source, "load_model");
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
      "load_model",
      `the file could not be read: GET ${url} failed: ${error.message}`,
    );
  });
  if (!response.ok) {
    throw new QuantionError(
      "parse_failed",
      "load_model",
      `the file could not be read: GET ${url} answered ${response.status}`,
    );
  }
  return new Uint8Array(await response.arrayBuffer());
}

export function loadWarp(model: Model): WarpModel {
  if (!(model instanceof Model) || !model._handle) {
    throw invalidArgument(
      "load_warp",
      "model",
      "must be an open Model from loadModel",
    );
  }
  return new WarpModel(
    model._backend.loadWarp(model._handle),
    model._backend,
    model._getMetadata("load_warp"),
  );
}

const QUERY_RT_RANGE = 0;
const QUERY_CLOSEST_RT = 1;
const QUERY_MZ_RANGE = 2;
const QUERY_CLOSEST_MZ = 3;

function getScanQuery(query: ScanQuery): [number, number, number] {
  const given = query as unknown as Record<string, unknown>;
  if (given === null || typeof given !== "object") {
    throw invalidArgument("get_scans", "query", "must be { rt } or { mz }");
  }
  const keys = Object.keys(given);
  if (keys.length !== 1 || (keys[0] !== "rt" && keys[0] !== "mz")) {
    throw invalidArgument(
      "get_scans",
      "query",
      "must have exactly one of rt or mz",
    );
  }
  const key = keys[0];
  const value = given[key];
  const closest = key === "rt" ? QUERY_CLOSEST_RT : QUERY_CLOSEST_MZ;
  const range = key === "rt" ? QUERY_RT_RANGE : QUERY_MZ_RANGE;
  if (typeof value === "number") return [closest, value, NaN];
  const fromTo = value as FromTo;
  if (fromTo === null || typeof fromTo !== "object") {
    throw invalidArgument("get_scans", key, "must be a number or { from, to }");
  }
  return [
    range,
    getNumber(fromTo.from, "get_scans", "from"),
    getNumber(fromTo.to, "get_scans", "to"),
  ];
}

export async function getScans(
  file: SampleFile,
  query: ScanQuery,
  options?: GetScansOptions,
): Promise<Scan[]> {
  const handle = getHandle(file, "get_scans");
  const settings = readGetScansOptions(options, "get_scans");
  const [queryType, from, to] = getScanQuery(query);
  return file._backend.getScans(
    handle,
    queryType,
    from,
    to,
    settings.level,
    getTimeUnitCode(settings.timeUnit),
  );
}

export async function calculateEic(
  file: SampleFile,
  mz: number,
  options?: CalculateEicOptions,
): Promise<DataXY> {
  const handle = getHandle(file, "calculate_eic");
  const eic = readEicOptions(options, "calculate_eic", "");
  return file._backend.calculateEic(
    handle,
    getNumber(mz, "calculate_eic", "mz"),
    getEicSettings(eic),
  );
}

export async function getIonImage(
  file: SampleFile,
  mz: number,
  options?: GetIonImageOptions,
): Promise<IonImage> {
  const handle = getHandle(file, "get_ion_image");
  const settings = readIonImageOptions(options, "get_ion_image");
  return file._backend.getIonImage(
    handle,
    getNumber(mz, "get_ion_image", "mz"),
    settings.tolerance,
    settings.level,
    settings.onProgress,
  );
}

export function findPeaks(
  x: Float64Array,
  y: Float64Array,
  options?: FindPeaksOptions,
): Peak[] {
  getTrace(x, y, "find_peaks");
  const settings = readFindPeaksOptions(options, "find_peaks", "");
  return getBackend().findPeaks(x, y, settings);
}

export function getPeak(
  x: Float64Array,
  y: Float64Array,
  rt: number,
  options?: GetPeakOptions,
): Peak {
  getTrace(x, y, "get_peak");
  const settings = readGetPeakOptions(options, "get_peak");
  return getBackend().getPeak(
    x,
    y,
    getNumber(rt, "get_peak", "rt"),
    settings.range,
    settings.findPeaks,
  );
}

export function fitPeak(
  x: Float64Array,
  y: Float64Array,
  rt: number,
  options?: FitPeakOptions,
): PeakParameters | null {
  getTrace(x, y, "fit_peak");
  const settings = readFitPeakOptions(options, "fit_peak");
  return getBackend().fitPeak(
    x,
    y,
    getNumber(rt, "fit_peak", "rt"),
    getShapeCode(settings.shape),
    settings.intensity,
  );
}

export function drawPeaks(
  params: PeakParameters[],
  options?: DrawPeaksOptions,
): DataXY {
  if (!Array.isArray(params)) {
    throw invalidArgument(
      "draw_peaks",
      "params",
      "must be an array of PeakParameters",
    );
  }
  const settings = readDrawPeaksOptions(options, "draw_peaks");
  const read = (key: keyof PeakParameters) =>
    Float64Array.from(params, (peak, index) =>
      getNumber(peak[key], "draw_peaks", `params[${index}].${key}`),
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

export function findNoiseLevel(y: Float64Array | Float32Array): NoiseLevel {
  const isFloat32Array =
    Object.prototype.toString.call(y) === "[object Float32Array]";
  if (!isFloat64Array(y) && !isFloat32Array) {
    throw invalidArgument(
      "find_noise_level",
      "y",
      "must be a Float64Array or Float32Array",
    );
  }
  const values = isFloat32Array ? (y as Float32Array) : new Float32Array(y);
  return getBackend().findNoiseLevel(values);
}

export function calculateBaseline(
  y: Float64Array,
  options?: CalculateBaselineOptions,
): Float64Array {
  if (!isFloat64Array(y)) {
    throw invalidArgument("calculate_baseline", "y", "must be a Float64Array");
  }
  const settings = readBaselineOptions(options, "calculate_baseline");
  return getBackend().calculateBaseline(
    y,
    settings.lambda,
    settings.maxIterations,
  );
}

export async function findTargets(
  file: SampleFile,
  targets: Target[],
  options?: FindTargetsOptions,
): Promise<Peak[]> {
  const handle = getHandle(file, "find_targets");
  const settings = readFindTargetsOptions(options, "find_targets");
  return file._backend.findTargets(
    handle,
    getTargetColumns(targets, "find_targets"),
    getEicSettings(settings.eic),
    settings.findPeaks,
    settings.threads,
  );
}

export function findTransitions(
  file: SampleFile,
  targets: Transition[],
  options?: FindTransitionsOptions,
): Peak[] {
  const handle = getHandle(file, "find_transitions");
  const settings = readFindTransitionsOptions(options, "find_transitions");
  return file._backend.findTransitions(
    handle,
    getTransitionColumns(targets, "find_transitions"),
    settings.findPeaks,
    settings.threads,
  );
}

export async function findFeatures(
  file: SampleFile,
  options?: FindFeaturesOptions,
): Promise<Feature[]> {
  const handle = getHandle(file, "find_features");
  const settings = readFindFeaturesOptions(options, "find_features");
  if (!file._backend.findFeatures) {
    throw new QuantionError(
      "invalid_argument",
      "find_features",
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

export function matchFeatures(
  directory: string,
  options?: MatchFeaturesOptions,
): Feature[] {
  const backend = getBackend();
  if (typeof directory !== "string") {
    throw invalidArgument("match_features", "directory", "must be a path");
  }
  const settings = readMatchFeaturesOptions(options, "match_features");
  if (!backend.matchFeatures) {
    throw new QuantionError(
      "invalid_argument",
      "match_features",
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
