import type {
  CentroidScan,
  IonImage,
  NoiseLevel,
  FindPeaksOptions,
  WarpTarget,
  ModelEntry,
} from "../types/types";

export type FileHandle = unknown;
export type ModelHandle = unknown;
export type WarpModelHandle = unknown;
export type MaybePromise<T> = T | Promise<T>;

export interface ByteRangeResult {
  offset: bigint;
  length: bigint;
}

export interface Backend {
  readonly ready: boolean;
  readonly handlesAreGcFinalized: boolean;

  init(): Promise<void>;

  parseMzML(data: Uint8Array): FileHandle;
  parseBin(data: Uint8Array, maxCacheSize?: number): FileHandle;
  parseIonPath(
    path: string,
    cacheSize: number,
  ): FileHandle | Promise<FileHandle>;

  parseIonRemote(url: URL, cacheSize: number): Promise<FileHandle>;
  parseIonBuffer(
    bytes: Uint8Array,
    cacheSize: number,
  ): FileHandle | Promise<FileHandle>;
  freeFile(handle: FileHandle): void;

  loadModelPath(path: string): ModelHandle;
  loadModelBytes(bytes: Uint8Array): ModelHandle;
  freeModel(model: ModelHandle): void;
  modelMetadata(model: ModelHandle): ModelEntry[];
  loadWarp(model: ModelHandle): WarpModelHandle;
  freeWarpModel(warp: WarpModelHandle): void;
  prefetchRts(
    warp: WarpModelHandle,
    handle: FileHandle,
    mzs: Float64Array,
    rts: Float64Array,
  ): Promise<void>;
  prefetchTargets(
    handle: FileHandle,
    mzs: Float64Array,
    from: number,
    to: number,
  ): Promise<void>;
  prefetchFeatures?(
    handle: FileHandle,
    from: number,
    to: number,
    eicPpm: number,
    eicMz: number,
    gridStart: number,
    gridEnd: number,
    gridStep: number,
  ): Promise<void>;
  predictRts(
    warp: WarpModelHandle,
    handle: FileHandle,
    mzs: Float64Array,
    rts: Float64Array,
  ): Float64Array;
  calibrateRts(
    warp: WarpModelHandle,
    handles: FileHandle[],
    listLengths: number[],
    ids: string[],
    mzs: Float64Array,
    rts: Float64Array,
  ): WarpTarget[];
  fileToMzml(handle: FileHandle): string;
  fileToBin(
    handle: FileHandle,
    level: number,
    f32Compress: boolean,
  ): Uint8Array;

  calculateEic(
    handle: FileHandle,
    targetMz: number,
    from: number,
    to: number,
    ppmTolerance: number,
    mzTolerance: number,
  ): MaybePromise<{ x: Float64Array; y: Float64Array }>;

  getScans(
    handle: FileHandle,
    queryType: number,
    a: number,
    b: number,
    level: number,
  ): MaybePromise<CentroidScan[]>;

  getIonImage(
    handle: FileHandle,
    mz: number,
    tolerance: number,
    level: number,
    onProgress?: (fetched: number, total: number, heldBytes: number) => void,
  ): MaybePromise<IonImage>;

  findPeaks(
    x: Float64Array,
    y: Float64Array,
    options: FindPeaksOptions | undefined,
  ): any;

  getPeak(
    x: Float64Array,
    y: Float64Array,
    rt: number,
    range: number,
    options: FindPeaksOptions | undefined,
  ): any;

  findNoiseLevel(y: Float32Array): NoiseLevel;

  calculateBaseline(
    y: Float64Array,
    lambda: number,
    maxIterations: number,
  ): Float64Array;

  findTargets(
    handle: FileHandle,
    rts: Float64Array,
    mzs: Float64Array,
    ranges: Float64Array,
    offsets: Uint32Array | null,
    lengths: Uint32Array | null,
    idBytes: Uint8Array | null,
    idBytesLen: number,
    count: number,
    from: number,
    to: number,
    options: FindPeaksOptions | undefined,
    threads: number,
  ): any;

  findTransitions(
    handle: FileHandle,
    precursors: Float64Array,
    products: Float64Array,
    rts: Float64Array,
    ranges: Float64Array,
    offsets: Uint32Array | null,
    lengths: Uint32Array | null,
    idBytes: Uint8Array | null,
    idBytesLen: number,
    count: number,
    options: FindPeaksOptions | undefined,
    threads: number,
  ): any;

  findFeatures?(
    handle: FileHandle,
    from: number,
    to: number,
    eicPpm: number,
    eicMz: number,
    gridStart: number,
    gridEnd: number,
    gridStep: number,
    options: FindPeaksOptions | undefined,
    threads: number,
  ): any;


  matchFeatures?(
    dirPath: string,
    from: number,
    to: number,
    eicPpm: number,
    eicMz: number,
    gStart: number,
    gEnd: number,
    gStep: number,
    groupPpm: number,
    groupMz: number,
    groupRt: number,
    prevalence: number,
    options: FindPeaksOptions | undefined,
    threads: number,
  ): any;
}
