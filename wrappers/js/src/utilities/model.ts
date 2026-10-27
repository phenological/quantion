import type { Backend, ModelHandle } from "./backend";
import { fileHandleRegistry } from "./handleRegistry";
import { Metadata } from "./metadata";
import type { ModelEntry } from "../types/types";

type Cell = ReturnType<typeof fileHandleRegistry.register>;

/**
 * A loaded ONNX model. Load it with {@link loadModel} and read its stored keys and values.
 */
export class Model extends Metadata {
  /** @internal */
  _backend: Backend;

  private _cell: Cell | null;

  private metadata: ModelEntry[] | null = null;

  constructor(handle: ModelHandle, backend: Backend) {
    super();
    this._backend = backend;
    this._cell = fileHandleRegistry.register(this, handle, (h) =>
      backend.freeModel(h),
    );
  }

  get _handle(): ModelHandle | null {
    return this._cell?.handle ?? null;
  }

  /**
   * Release the native memory held by this model. Warp models made from it keep working.
   */
  dispose(): void {
    const cell = this._cell;
    if (!cell) return;
    fileHandleRegistry.release(this, cell);
  }

  /** @internal */
  getMetadata(caller: string): ModelEntry[] {
    if (this.metadata === null) {
      const handle = this._handle;
      if (handle === null) {
        throw new Error(`${caller}: the model has been disposed`);
      }
      this.metadata = this._backend.modelMetadata(handle);
    }
    return this.metadata;
  }
}
