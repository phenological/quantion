import { describe, it, expect, beforeAll } from "@jest/globals";
import * as path from "path";
import { NodeBackend } from "../utilities/backendNode";
import {
  setBackend,
  parseIon,
  getTransition,
  getTransitionMetadata,
} from "../utilities/api";
import { QuantionError } from "../utilities/errors";
import type { SampleFile } from "../utilities/sampleFile";

const srmFile = path.join(__dirname, "../../../../core/tests/fixtures/srm.ion");
const alanine = 89.04768;
const serine = 90.0555;

function getNodeBackend(): NodeBackend | null {
  try {
    return new NodeBackend(process);
  } catch {
    return null;
  }
}

const nodeBackend = getNodeBackend();
const describeWithNode = nodeBackend ? describe : describe.skip;

function getError(read: () => unknown): QuantionError {
  try {
    read();
  } catch (error) {
    return error as QuantionError;
  }
  throw new Error("no error");
}

describeWithNode("getTransition", () => {
  let file: SampleFile;

  beforeAll(async () => {
    setBackend(nodeBackend!);
    file = await parseIon(srmFile);
  });

  it("reads each stored chromatogram by index", () => {
    for (const index of [0, 1]) {
      const trace = getTransition(file, index);
      expect(trace.x.length).toBeGreaterThan(0);
      expect(trace.y.length).toBe(trace.x.length);
    }
  });

  it("gives an empty chromatogram for an index past the end", () => {
    const trace = getTransition(file, 2);
    expect(trace.x.length).toBe(0);
    expect(trace.y.length).toBe(0);
  });

  it("reads the same chromatogram by transition as by index", () => {
    const trace = getTransition(file, {
      precursor: alanine,
      product: alanine,
    });
    const byIndex = [0, 1].map((index) =>
      Array.from(getTransition(file, index).y),
    );
    expect(trace.y.length).toBeGreaterThan(0);
    expect(byIndex).toContainEqual(Array.from(trace.y));
  });

  it("matches a transition within the tolerance", () => {
    const near = getTransition(file, {
      precursor: 89.0526,
      product: 89.0436,
    });
    const exact = getTransition(file, {
      precursor: alanine,
      product: alanine,
    });
    expect(Array.from(near.y)).toEqual(Array.from(exact.y));
  });

  it("gives an empty chromatogram for a wrong product", () => {
    const trace = getTransition(file, {
      precursor: alanine,
      product: serine,
    });
    expect(trace.x.length).toBe(0);
  });

  it("refuses a negative index", () => {
    const error = getError(() => getTransition(file, -1));
    expect(error).toBeInstanceOf(QuantionError);
    expect(error.kind).toBe("invalid_argument");
    expect(error.argument).toBe("query");
  });

  it("refuses a query without a product", () => {
    const error = getError(() =>
      getTransition(file, { precursor: alanine } as never),
    );
    expect(error.kind).toBe("invalid_argument");
    expect(error.argument).toBe("query");
  });

  it("refuses a negative precursor", () => {
    const error = getError(() =>
      getTransition(file, { precursor: -1, product: alanine }),
    );
    expect(error.kind).toBe("invalid_argument");
    expect(error.argument).toBe("precursor");
    expect(error.message).toBe(
      "getTransition: precursor must be greater than 0, got -1",
    );
  });
});

describeWithNode("getTransitionMetadata", () => {
  let file: SampleFile;

  beforeAll(async () => {
    setBackend(nodeBackend!);
    file = await parseIon(srmFile);
  });

  it("reads every row of the chromatogram at an index", () => {
    const rows = getTransitionMetadata(file, 0);
    expect(rows).toHaveLength(11);
    expect(
      rows.slice(0, 5).map((row) => [row.path, row.name, row.value]),
    ).toEqual([
      ["chromatogram", "index", "0"],
      ["chromatogram", "id", "SRM SIC Q1=89.04768 Q3=89.04768"],
      ["chromatogram", "defaultArrayLength", "1035"],
      [
        "chromatogram/precursor/isolationWindow",
        "isolation window target m/z",
        "89.04768",
      ],
      [
        "chromatogram/product/isolationWindow",
        "isolation window target m/z",
        "89.04768",
      ],
    ]);
    expect(rows[3].kind).toBe("CV");
    expect(rows[3].accession).toBe("MS:1000827");
    expect(rows[0].accession).toBeNull();
  });

  it("reads the same rows by transition as by index", () => {
    expect(
      getTransitionMetadata(file, { precursor: serine, product: serine }),
    ).toEqual(getTransitionMetadata(file, 1));
    expect(
      getTransitionMetadata(file, { precursor: 89.0526, product: 89.0436 }),
    ).toEqual(getTransitionMetadata(file, 0));
  });

  it("keeps only the rows of the given tags", () => {
    expect(
      getTransitionMetadata(file, 0, { tags: ["chromatogram"] }),
    ).toHaveLength(3);
    expect(
      getTransitionMetadata(file, 0, { tags: ["isolationWindow"] }).map(
        (row) => row.path,
      ),
    ).toEqual([
      "chromatogram/precursor/isolationWindow",
      "chromatogram/product/isolationWindow",
    ]);
  });

  it("gives no rows when no chromatogram matches", () => {
    expect(getTransitionMetadata(file, 2)).toEqual([]);
    expect(
      getTransitionMetadata(file, { precursor: alanine, product: serine }),
    ).toEqual([]);
  });

  it("refuses bad arguments", () => {
    const tagError = getError(() =>
      getTransitionMetadata(file, 0, { tags: ["scan"] }),
    );
    expect(tagError.argument).toBe("tags");
    expect(tagError.function).toBe("getTransitionMetadata");
    expect(getError(() => getTransitionMetadata(file, -1)).argument).toBe(
      "query",
    );
    const productError = getError(() =>
      getTransitionMetadata(file, { precursor: alanine, product: -1 }),
    );
    expect(productError.kind).toBe("invalid_argument");
    expect(productError.argument).toBe("product");
  });
});
