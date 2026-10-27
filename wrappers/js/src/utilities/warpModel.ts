import type { Backend, WarpModelHandle } from "./backend";
import { fileHandleRegistry } from "./handleRegistry";
import { Metadata } from "./metadata";
import { SampleFile } from "./sampleFile";
import type { ModelEntry, PredictedRt, WarpTarget } from "../types/types";

type Cell = ReturnType<typeof fileHandleRegistry.register>;

/**
 * A warp model. Make it once with {@link loadWarp} and use it
 * with {@link predictRts} for every file.
 */
export class WarpModel extends Metadata {
  /** @internal */
  _backend: Backend;

  private _cell: Cell | null;

  private metadata: ModelEntry[];

  constructor(
    handle: WarpModelHandle,
    backend: Backend,
    metadata: ModelEntry[],
  ) {
    super();
    this._backend = backend;
    this.metadata = metadata;
    this._cell = fileHandleRegistry.register(this, handle, (h) =>
      backend.freeWarpModel(h),
    );
  }

  get _handle(): WarpModelHandle | null {
    return this._cell?.handle ?? null;
  }

  /**
   * Predict each target's RT in a file.
   *
   * @param file - The sample file.
   * @param targets - Targets, each with `mz` and `rt`.
   * @returns One `{ rt, uncertainty }` per target, in the same order.
   */
  async predictRts(
    file: SampleFile,
    targets: WarpTarget[],
  ): Promise<PredictedRt[]> {
    const warp = this.getHandle("predictRts");
    const handle = getFileHandle(file, "predictRts");
    if (targets.length === 0) return [];

    const mzs = Float64Array.from(targets, (target) => target.mz);
    const rts = Float64Array.from(targets, (target) => target.rt);
    await this._backend.prefetchRts(warp, handle, mzs, rts);
    const values = this._backend.predictRts(warp, handle, mzs, rts);
    return targets.map((_, index) => ({
      rt: values[index * 2],
      uncertainty: values[index * 2 + 1],
    }));
  }

  /**
   * Turn the RTs seen in anchor samples into targets for {@link predictRts}.
   *
   * @param anchorSamples - The samples where the targets were seen, like LTRs or QCs.
   * @param targetLists - One list per sample, with `id`, `mz` and the `rt` seen in it.
   * @returns One `{ id, mz, rt }` per id, with the median calibrated rt over the samples.
   */
  async calibrate(
    anchorSamples: SampleFile[],
    targetLists: WarpTarget[][],
  ): Promise<WarpTarget[]> {
    const warp = this.getHandle("calibrate");
    if (anchorSamples.length !== targetLists.length) {
      throw new Error(
        `calibrate needs one target list per sample, got ${anchorSamples.length} samples and ${targetLists.length} target lists`,
      );
    }
    const handles = anchorSamples.map((sample) =>
      getFileHandle(sample, "calibrate"),
    );
    for (const [index, targets] of targetLists.entries()) {
      await this._backend.prefetchRts(
        warp,
        handles[index],
        Float64Array.from(targets, (target) => target.mz),
        Float64Array.from(targets, (target) => target.rt),
      );
    }
    const targets = targetLists.flat();
    return this._backend.calibrateRts(
      warp,
      handles,
      targetLists.map((list) => list.length),
      targets.map((target) => target.id ?? ""),
      Float64Array.from(targets, (target) => target.mz),
      Float64Array.from(targets, (target) => target.rt),
    );
  }

  /**
   * Release the native memory held by this model.
   */
  dispose(): void {
    const cell = this._cell;
    if (!cell) return;
    fileHandleRegistry.release(this, cell);
  }

  protected getMetadata(): ModelEntry[] {
    return this.metadata;
  }

  private getHandle(caller: string): WarpModelHandle {
    const handle = this._handle;
    if (handle === null) {
      throw new Error(`${caller}: the warp model has been disposed`);
    }
    return handle;
  }
}

function getFileHandle(file: SampleFile, caller: string) {
  if (!(file instanceof SampleFile) || !file._handle) {
    throw new Error(`${caller}: expects a valid SampleFile object`);
  }
  return file._handle;
}
