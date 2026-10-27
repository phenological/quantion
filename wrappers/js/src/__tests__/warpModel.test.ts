import { describe, it, expect, beforeAll, afterAll } from "@jest/globals";
import * as fs from "fs";
import * as http from "http";
import * as path from "path";
import { NodeBackend } from "../utilities/backendNode";
import {
  setBackend,
  loadWarpModel,
  predictRts,
  parseIon,
} from "../utilities/api";

const fixtures = path.join(__dirname, "../../../../core/tests/fixtures");
const modelPath = path.join(fixtures, "shift_model.onnx");
const coveringFile = path.join(fixtures, "api/api.ion");
const emptyFile = path.join(fixtures, "test.ion");
const targets = [
  { id: "first", mz: 250.1, rt: 1.2 },
  { id: "second", mz: 310.2, rt: 3.1 },
];

describe("warp model", () => {
  let server: http.Server;
  let modelUrl: string;
  let fileUrl: string;

  beforeAll((done) => {
    setBackend(new NodeBackend(process));
    const modelBytes = fs.readFileSync(modelPath);
    const fileBytes = fs.readFileSync(coveringFile);
    server = http.createServer((request, response) => {
      if (request.url === "/missing.onnx") {
        response.writeHead(404);
        response.end("not found");
        return;
      }
      if (request.url !== "/api.ion") {
        response.writeHead(200, { "content-length": modelBytes.length });
        response.end(modelBytes);
        return;
      }
      const match = (request.headers.range ?? "").match(/bytes=(\d+)-(\d+)/);
      if (!match) {
        response.writeHead(200, { "content-length": fileBytes.length });
        response.end(fileBytes);
        return;
      }
      const first = Number(match[1]);
      const last = Math.min(Number(match[2]), fileBytes.length - 1);
      const piece = fileBytes.subarray(first, last + 1);
      response.writeHead(206, {
        "content-length": piece.length,
        "content-range": `bytes ${first}-${last}/${fileBytes.length}`,
      });
      response.end(piece);
    });
    server.listen(0, () => {
      const address = server.address() as { port: number };
      modelUrl = `http://127.0.0.1:${address.port}/shift_model.onnx`;
      fileUrl = `http://127.0.0.1:${address.port}/api.ion`;
      done();
    });
  });

  afterAll((done) => {
    server.close(() => done());
  });

  it("gives one prediction per target, in order", async () => {
    const warp = await loadWarpModel(modelPath);
    const file = await parseIon(coveringFile);
    const predicted = await predictRts(warp, file, targets);
    expect(predicted).toHaveLength(targets.length);
    for (const prediction of predicted) {
      expect(Object.keys(prediction).sort()).toEqual(["rt", "uncertainty"]);
      expect(Number.isFinite(prediction.rt)).toBe(true);
    }
    file.dispose();
    warp.dispose();
  });

  it("predicts the same from a path, bytes and a URL", async () => {
    const file = await parseIon(coveringFile);
    const fromPath = await predictRts(await loadWarpModel(modelPath), file, targets);
    const fromBytes = await predictRts(
      await loadWarpModel(new Uint8Array(fs.readFileSync(modelPath))),
      file,
      targets,
    );
    const fromUrl = await predictRts(await loadWarpModel(modelUrl), file, targets);
    expect(fromBytes).toEqual(fromPath);
    expect(fromUrl).toEqual(fromPath);
    file.dispose();
  });

  it("predicts the same on a remote file as on a local one", async () => {
    const warp = await loadWarpModel(modelPath);
    const local = await parseIon(coveringFile);
    const remote = await parseIon(fileUrl);
    expect(await predictRts(warp, remote, targets)).toEqual(
      await predictRts(warp, local, targets),
    );
    local.dispose();
    remote.dispose();
  });

  it("names a target without mz by its index", async () => {
    const warp = await loadWarpModel(modelPath);
    const file = await parseIon(coveringFile);
    await expect(
      predictRts(warp, file, [targets[0], { id: "broken", mz: NaN, rt: 2 }]),
    ).rejects.toThrow("predict_rts: target 2 has no valid m/z or rt");
    file.dispose();
  });

  it("refuses a file whose scans do not cover the model range", async () => {
    const warp = await loadWarpModel(modelPath);
    const file = await parseIon(emptyFile);
    await expect(predictRts(warp, file, targets)).rejects.toThrow(
      "predict_rts: the file has no scans, the warp model needs",
    );
    file.dispose();
  });

  it("refuses a model URL that answers with an HTTP error", async () => {
    await expect(
      loadWarpModel(modelUrl.replace("shift_model.onnx", "missing.onnx")),
    ).rejects.toThrow("the model download failed with HTTP 404");
  });

  it("refuses a missing model file", async () => {
    await expect(
      loadWarpModel(path.join(fixtures, "missing.onnx")),
    ).rejects.toThrow("load_warp_model: the warp model was not read");
  });
});
