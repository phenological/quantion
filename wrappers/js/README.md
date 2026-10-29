# quantion-js

JavaScript wrappers for the quantion library for LC-MS data processing.

## Install

```bash
npm install quantion
```

## Concepts

**Sample**: an opened `.ion` or mzML file. Open it with `parseIon()` or `parseMzml()`. It is freed when no longer referenced, or call `dispose()` to free it sooner.

**DataXY**: a pair of `Float64Array`s, `x` = retention times, `y` = intensities.

**Options**: every function takes one optional `options` object. Groups are nested and named the same in every quantion language: `eic`, `grid`, `grouping`, `findPeaks`, `threads`. An unknown key throws a `QuantionError` with kind `unknown_option`.

**Enums**: `TimeUnit` (`MINUTES`, `SECONDS`) and `PeakShape` (`GAUSSIAN`, `EMG`). Use the constants or the uppercase strings.

## Quick start

```js
import * as quantion from "quantion";

const sample = await quantion.parseIon("sample.ion");
const eic = await quantion.calculateEic(sample, 260.103, { from: 0.5, to: 5 });
const peaks = quantion.findPeaks(eic.x, eic.y, { minSnr: 3 });

for (const peak of peaks) {
  console.log(`rt=${peak.rt.toFixed(3)}  intensity=${peak.intensity.toFixed(0)}`);
}
```

## Open and write files

```js
const sample = await quantion.parseIon("sample.ion");
const remote = await quantion.parseIon("https://example.org/sample.ion", {
  maxCacheSize: 64_000_000,
});
const fromBytes = await quantion.parseIon(new Uint8Array(bytes));

const mzml = await quantion.parseMzml("sample.mzML");
```

A path or URL is read lazily: only the bytes a call needs are fetched. mzML is always loaded fully, from a path (Node) or bytes; a URL is refused.

`writeIon` and `writeMzml` return the bytes when `out` is left out, write a file when `out` is a path (Node only), and `writeIon` uploads with one `PUT` when `out` is a URL.

```js
const bytes = await quantion.writeIon(sample);
await quantion.writeIon(sample, { out: "sample.ion", level: 12 });
await quantion.writeIon(sample, {
  out: "https://example.org/upload/sample.ion",
  headers: [{ name: "Authorization", value: "Bearer <token>" }],
});
await quantion.writeMzml(sample, { out: "sample.mzML", overwrite: true });
```

## Read data

```js
const eic = await quantion.calculateEic(sample, 260.103, {
  from: 0.5,
  to: 5,
  ppmTolerance: 20,
  mzTolerance: 0.005,
});

const scans = await quantion.getScans(sample, { rt: { from: 1.0, to: 2.5 } });
const closest = await quantion.getScans(
  sample,
  { rt: 108 },
  { level: 2, timeUnit: quantion.TimeUnit.SECONDS },
);

const image = await quantion.getIonImage(sample, 760.585, { tolerance: 0.05 });
```

`getScans` always returns an array; one `rt` or `mz` gives at most one scan. `getIonImage` needs an `.ion` sample.

## Process one trace

```js
const peaks = quantion.findPeaks(eic.x, eic.y, { minIntensity: 1000 });
const peak = quantion.getPeak(eic.x, eic.y, 2.383, { range: 0.05 });
const anywhere = quantion.getPeak(eic.x, eic.y, 2.383);

const fitted = quantion.fitPeak(eic.x, eic.y, 2.383, { shape: "GAUSSIAN" });
const simulated = quantion.drawPeaks(
  [
    { height: 9800, center: 2.38, fwhm: 0.021, tail: 0.006 },
    { height: 4200, center: 2.45, fwhm: 0.018, tail: 0 },
  ],
  { from: 2, to: 3, points: 2000 },
);

const noise = quantion.findNoiseLevel(eic.y).intensity;
const baseline = quantion.calculateBaseline(eic.y, { lambda: 50 });
```

`getPeak` without `range` searches the whole trace. No peak gives a `Peak` with every field 0.

## Find targets

```js
const targets = [
  { id: "compound_1", mz: 260.103, rt: 2.383, range: 0.05 },
  { id: "compound_2", mz: 260.103, rt: 2.189 },
];
const peaks = await quantion.findTargets(sample, targets, {
  eic: { from: 0.5, to: 5 },
  findPeaks: { minIntensity: 1000 },
  threads: 4,
});

const transitions = [
  { id: "compound_1", precursor: 500.3, product: 184.1, rt: 3.2, range: 0.4 },
];
const srmPeaks = quantion.findTransitions(sample, transitions, { threads: 2 });
```

`findTargets` returns one `Peak` per target in the same order. `findTransitions` needs the whole file, so a sample opened from a URL gives a `remote_file` error.

## Untargeted

```js
const features = await quantion.findFeatures(sample, {
  grid: { start: 100, end: 600 },
  threads: 4,
});

const matched = quantion.matchFeatures("samples/", {
  grouping: { minFrequency: 0.5 },
  threads: 8,
});
```

`matchFeatures` runs on Node only. Each matched `Feature` has `frequency`, the fraction of samples it was found in.

## Predict retention times

```js
const model = await quantion.loadModel("model.onnx");
const warp = quantion.loadWarp(model);
const predicted = await warp.predictRts(sample, [{ mz: 260.103, rt: 2.383 }]);

const calibrated = await warp.calibrate([anchor1, anchor2], [seenIn1, seenIn2]);

warp.dispose();
model.dispose();
```

## Errors

Every error is a `QuantionError` with `kind`, `code`, `function` and `argument`, the same in every quantion language:

```js
try {
  await quantion.calculateEic(sample, -1);
} catch (error) {
  error.kind; // "invalid_argument"
  error.code; // 1
  error.argument; // "mz"
  error.message; // "calculate_eic: mz must be greater than 0, got -1"
}
```
