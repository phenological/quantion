import type { Backend, FileHandle } from "./backend";
import { fileHandleRegistry } from "./handleRegistry";

type Cell = ReturnType<typeof fileHandleRegistry.register>;

/**
 * A loaded sample file. Holds an internal handle to the native
 * file object. Cleanup is automatic, but call {@link dispose} when
 * you're done with large samples or tight processing loops to free memory sooner.
 */
export class SampleFile {
  /** @internal */
  _backend: Backend;

  private _cell: Cell | null;

  constructor(handle: FileHandle, backend: Backend) {
    this._backend = backend;

    if (backend.handlesAreGcFinalized) {
      this._cell = { handle, free: () => {} };
    } else {
      this._cell = fileHandleRegistry.register(this, handle, (h) =>
        backend.freeFile(h),
      );
    }
  }

  get _handle(): FileHandle | null {
    return this._cell?.handle ?? null;
  }

  /**
   * Release the native memory held by this sample.
   */
  dispose(): void {
    const cell = this._cell;
    if (!cell) return;

    if (this._backend.handlesAreGcFinalized) {
      const h = cell.handle;
      if (h != null) {
        this._backend.freeFile(h);
        cell.handle = null;
      }
    } else {
      fileHandleRegistry.release(this, cell);
    }
  }
}
