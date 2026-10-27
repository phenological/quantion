import type { ModelEntry } from "../types/types";

export abstract class Metadata {
  protected abstract getMetadata(caller: string): ModelEntry[];

  /**
   * The keys stored in the model file, in file order.
   */
  getKeys(): string[] {
    return this.getMetadata("getKeys").map((row) => row.key);
  }

  /**
   * The values stored in the model file, in the same order as {@link getKeys}.
   */
  getValues(): string[] {
    return this.getMetadata("getValues").map((row) => row.value);
  }

  /**
   * The text stored under `key` in the model file, or `undefined` if it is missing.
   */
  getValue(key: string): string | undefined {
    return this.getMetadata("getValue").find((row) => row.key === key)?.value;
  }
}
