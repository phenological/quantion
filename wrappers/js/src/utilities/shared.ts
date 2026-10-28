import type { BinaryInput } from "../types/types";

export function toThreads(threads: unknown): number {
  const v = typeof threads === "number" && Number.isFinite(threads) ? threads | 0 : 1;
  return v > 0 ? v : 1;
}

export function toUint8(input: BinaryInput): Uint8Array {
  if (input instanceof Uint8Array) return input;
  return new Uint8Array(input);
}
