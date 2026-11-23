import { describe, it, expect, beforeAll, afterAll } from "@jest/globals";
import * as fs from "fs";
import * as path from "path";
import { type LibraryServer, startLibraryServer } from "./syntheticLibrary";

const bundlePath = path.join(__dirname, "../../dist/quantion.js");
const srmFile = path.join(__dirname, "../../../../core/tests/fixtures/srm.ion");
const describeWithBundle = fs.existsSync(bundlePath) ? describe : describe.skip;
const alanine = 89.04768;
const serine = 90.0555;

describeWithBundle("getTransition on wasm", () => {
  let quantion: any;
  let bytes: Uint8Array;
  let server: LibraryServer;

  beforeAll(async () => {
    quantion = require(bundlePath);
    await quantion.init();
    bytes = new Uint8Array(fs.readFileSync(srmFile));
    server = await startLibraryServer(bytes);
  });

  afterAll(() => server.stop());

  it("reads the same chromatogram by index and by transition", async () => {
    const file = await quantion.parseIon(bytes);
    const trace = quantion.getTransition(file, {
      precursor: alanine,
      product: alanine,
    });
    const byIndex = [0, 1].map((index) =>
      Array.from(quantion.getTransition(file, index).y),
    );
    expect(trace.x.length).toBeGreaterThan(0);
    expect(byIndex).toContainEqual(Array.from(trace.y));
    expect(quantion.getTransition(file, 2).x.length).toBe(0);
    expect(
      quantion.getTransition(file, { precursor: alanine, product: serine }).x
        .length,
    ).toBe(0);
    file.dispose();
  });

  it("reads the metadata of a chromatogram by index and by transition", async () => {
    const file = await quantion.parseIon(bytes);
    const rows = quantion.getTransitionMetadata(file, 0);
    expect(rows).toHaveLength(11);
    expect(rows[1].value).toBe("SRM SIC Q1=89.04768 Q3=89.04768");
    expect(
      quantion.getTransitionMetadata(file, {
        precursor: serine,
        product: serine,
      }),
    ).toEqual(quantion.getTransitionMetadata(file, 1));
    expect(
      quantion.getTransitionMetadata(file, 0, { tags: ["isolationWindow"] }),
    ).toHaveLength(2);
    expect(quantion.getTransitionMetadata(file, 2)).toEqual([]);
    file.dispose();
  });

  it("gives a remote_file error for a sample opened from a URL", async () => {
    const file = await quantion.parseIon(server.url);
    let error: any = null;
    try {
      quantion.getTransition(file, 0);
    } catch (caught) {
      error = caught;
    }
    expect(error).toBeInstanceOf(quantion.QuantionError);
    expect(error.kind).toBe("remote_file");
    expect(error.function).toBe("getTransition");
    file.dispose();
  });
});
