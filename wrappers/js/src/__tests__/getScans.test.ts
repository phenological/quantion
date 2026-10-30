import { describe, it, expect, beforeAll } from "@jest/globals";
import * as path from "path";
import { NodeBackend } from "../utilities/backendNode";
import { setBackend, parseIon, getScans } from "../utilities/api";
import { QuantionError } from "../utilities/errors";

const sampleFile = path.join(
  __dirname,
  "../../../../core/tests/fixtures/api/api.ion",
);

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
    const scans = await getScans(await parseIon(sampleFile), {
      rt: { from: 2.3, to: 2.5 },
    });
    expect(scans.length).toBeGreaterThan(0);
    expect(scans.every((scan) => scan.rt >= 2.3 && scan.rt <= 2.5)).toBe(true);
  });

  it("gives the closest scan for one rt", async () => {
    const scans = await getScans(await parseIon(sampleFile), { rt: 2.385 });
    expect(scans).toHaveLength(1);
    expect(Math.abs(scans[0].rt - 2.385)).toBeLessThan(0.05);
    expect(scans[0].level).toBe(1);
  });

  it("refuses an mz that is not positive", async () => {
    const error = await getScans(await parseIon(sampleFile), { mz: 0 }).catch(
      (caught) => caught,
    );
    expect(error).toBeInstanceOf(QuantionError);
    expect(error.message).toBe("getScans: mz must be greater than 0, got 0");
    expect(error.argument).toBe("mz");
  });
});
