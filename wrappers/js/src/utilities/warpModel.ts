import type { Backend, WarpModelHandle } from "./backend";
import { fileHandleRegistry } from "./handleRegistry";

type Cell = ReturnType<typeof fileHandleRegistry.register>;

/**
 * A loaded warp model. Load it once with {@link loadWarpModel} and use it
 * with {@link predictRts} for every file.
 */
export class WarpModel {
  /** @internal */
  _backend: Backend;

  private _cell: Cell | null;

  constructor(handle: WarpModelHandle, backend: Backend) {
    this._backend = backend;
    this._cell = fileHandleRegistry.register(this, handle, (h) =>
      backend.freeWarpModel(h),
    );
  }

  get _handle(): WarpModelHandle | null {
    return this._cell?.handle ?? null;
  }

  /**
   * Release the native memory held by this model.
   */
  dispose(): void {
    const cell = this._cell;
    if (!cell) return;
    fileHandleRegistry.release(this, cell);
  }
}
