import { describe, it, expect, beforeAll } from "@jest/globals";
import * as path from "path";
import { NodeBackend } from "../utilities/backendNode";
import { setBackend, parseIon, getScans } from "../utilities/api";
import type { CentroidScan } from "../types/types";

const sampleFile = path.join(__dirname, "../../../../core/tests/fixtures/api/api.ion");

function getNodeBackend(): NodeBackend | null {
  try {
    return new NodeBackend(process);
  } catch {
    return null;
  }
}

const nodeBackend = getNodeBackend();
const describeWithNode = nodeBackend ? describe : describe.skip;

describeWithNode("getScans", () => {
  beforeAll(() => {
    setBackend(nodeBackend!);
  });

  it("gives every scan inside an rt range", async () => {
    const scans = (await getScans(await parseIon(sampleFile), { rt: { from: 2.3, to: 2.5 } })) as CentroidScan[];
    expect(scans.length).toBeGreaterThan(0);
    expect(scans.every((scan) => scan.rt >= 2.3 && scan.rt <= 2.5)).toBe(true);
  });

  it("gives the closest scan for one rt", async () => {
    const scan = (await getScans(await parseIon(sampleFile), { rt: 2.385 })) as CentroidScan;
    expect(Math.abs(scan.rt - 2.385)).toBeLessThan(0.05);
  });

  it("refuses an mz that is not positive", async () => {
    await expect(getScans(await parseIon(sampleFile), { mz: 0 })).rejects.toThrow("mz must be positive");
  });
});
