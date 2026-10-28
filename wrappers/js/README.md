# quantion-js

JavaScript wrappers for the quantion library for LC-MS data processing.

## Install

```bash
npm install quantion
```

## Concepts

**Sample**: A parsed mzML or opened ion source. Create samples with `parseMzML()` or `parseIon()`. Samples are automatically garbage collected when no longer referenced.

**XY data**: A pair of numeric arrays where `x` is retention time and `y` is intensity.

## Quick start

Load a file and extract peaks in a few lines:

```js
import * as quantion from "quantion";

const raw = await fetch("sample.mzML").then((r) => r.arrayBuffer());

const sample = await quantion.parseMzML(raw);
const eic = await quantion.calculateEic(sample, 174.112, { from: 1, to: 15 });
const peaks = quantion.findPeaks(eic.x, eic.y, {
  autoNoise: true,
  minSnr: 3.0,
});

peaks.forEach((p) => {
  console.log(`rt=${p.rt.toFixed(3)}  intensity=${p.intensity.toFixed(0)}`);
});
```

## Load files

### Load mzML

Load an mzML file into a sample.

```js
const sample = await quantion.parseMzML(buffer);
```

### Load ion

Open an ion binary source into a sample.

```js
const sample = await quantion.parseIon(buffer);
const sample = await quantion.parseIon(buffer, { maxCacheSize: 1000000 });
const sample = await quantion.parseIon("./data/sample.ion");
const sample = await quantion.parseIon(
  new URL("https://example.com/sample.ion"),
);
```

`string` inputs are filesystem paths only. URL inputs must use `new URL(...)`. URL range reads are supported by the Node.js backend.

## Convert

### Sample to mzML

Get mzML text from a sample.

```js
const mzml = quantion.ionToMzml(sample);
```

### Sample to ion

Convert a sample to compressed ion bytes.

```js
const ionBytes = quantion.mzmlToIon(sample, { level: 12, f32Compress: false });
```

## Extract from samples

### Extract EIC

Get an extracted ion chromatogram for one m/z from a sample.

```js
const eic = await quantion.calculateEic(
  sample,
  174.112,
  { from: 1, to: 15 },
  20.0, // ppm tolerance
  0.005, // m/z tolerance
);
// eic.x and eic.y are Float64Array
```

### Get scans

Get scans from a sample by retention time range, m/z range, or closest value.

```js
// Get scans in RT range
const scans = await quantion.getScans(sample, { rt: { from: 1, to: 15 } }, 1);

// Get scan closest to RT
const scan = await quantion.getScans(sample, { rt: 5.3 }, 1);

// Get scans in m/z range
const scans = await quantion.getScans(sample, { mz: { from: 100, to: 500 } }, 2);

// Get scan closest to m/z
const scan = await quantion.getScans(sample, { mz: 174.112 }, 2);
```

## Process XY data

### Calculate baseline

Estimate a baseline for XY data.

```js
const baseline = quantion.calculateBaseline(y, {
  baselineWindow: 101,
  baselineWindowFactor: 3,
});
```

### Find noise level

Estimate the noise level in XY data. The result has `width`, the number of
points the estimate came from, and `intensity`, the noise level itself.

```js
const { width, intensity } = quantion.findNoiseLevel(y);
```

### Find peaks

Find all peaks in XY data.

```js
const peaks = quantion.findPeaks(x, y, {
  autoNoise: true,
  minSnr: 3.0,
});
```

### Get single peak

Find one peak near a target retention time in XY data.

```js
const peak = quantion.getPeak(x, y, 5.3, 0.6, { autoNoise: true });
```

## Predict retention times

Load an ONNX model from a path (Node only), a URL, or bytes, read the keys and
values stored in it, and make a warp model from it. The model stays usable after
`loadWarp`, and the warp keeps working if the model is disposed.

```js
const model = await quantion.loadModel("model.onnx");
model.getKeys(); // ["points", "time_from", ..., "targets"]
model.getValue("points"); // "1024"

const warp = quantion.loadWarp(model);
warp.getValue("targets"); // the warp has the same keys and values
```

Predict each target's retention time in a sample. You get one
`{ rt, uncertainty }` per target, in the same order.

```js
const targets = [{ id: "feature_1", mz: 300.1, rt: 2.38 }];
const rts = await warp.predictRts(sample, targets);
```

Without reference RTs, calibrate from anchor samples such as LTRs or QCs: give
the samples and, for each one, the targets seen in it. Each target gets the
median calibrated RT over the samples where it was seen. One sample works, three
are better, five or more are best. Each RT must be the one measured in
that anchor sample: a generic target list runs without error but gives wrong RTs.

```js
const anchorSamples = [ltr1, ltr2, ltr3];
const targetLists = [seenInLtr1, seenInLtr2, seenInLtr3];
const calibrated = await warp.calibrate(anchorSamples, targetLists);
const otherRts = await warp.predictRts(otherSample, calibrated);

warp.dispose();
model.dispose();
```

## Find peaks from samples

### From EICs

Find peaks for many targets using extracted ion chromatograms from a sample.

```js
const targets = [
  { rt: 5.3, mz: 174.112, range: 0.5, id: "compound_A" },
  { rt: 7.1, mz: 203.156, range: 0.5, id: "compound_B" },
];

const peaks = await quantion.getPeaksFromEic(
  sample,
  targets,
  { from: 0.5, to: 10.0 },
  { autoNoise: true, threads: 2 },
);
```

### From chromatograms

Find peaks in the SRM chromatograms stored in a sample. Each target picks the
chromatogram whose precursor (Q1) and product (Q3) m/z are both within 0.005.

```js
const targets = [
  { id: "compound_1", precursor: 500.3, product: 184.1, rt: 3.20, range: 0.4 },
  { id: "compound_2", precursor: 520.4, product: 184.1, rt: 3.65, range: 0.4 },
];

const peaks = quantion.getPeaksFromChrom(sample, targets, { autoNoise: true, threads: 2 });
```

## Untargeted

### Feature detection

Find targeted features by m/z and retention time from a sample

```js
const targets = [
  { rt: 5.3, mz: 174.112, range: 0.5, id: "A" },
  { rt: 7.1, mz: 203.156, range: 0.5, id: "B" },
];

const features = await quantion.findFeature(sample, targets, {
  scanEic: { ppmTolerance: 10, mzTolerance: 0.003 },
  eic: { ppmTolerance: 20, mzTolerance: 0.005 },
  findPeak: { autoNoise: true },
  threads: 2,
});
```

### From single sample

Find all features in a sample (Node.js only).

```js
const features = await quantion.findFeatures(
  sample,
  { from: 1, to: 15 },
  {
    eic: { ppmTolerance: 10, mzTolerance: 0.005 },
    grid: { start: 40, end: 1000, stepSize: 0.005 },
    findPeak: { autoNoise: true, minSnr: 3 },
    threads: 4,
  },
);
```

### Find and align features across samples

Find and align features across many samples (Node.js only).

```js
const features = quantion.getFeatures(
  "/path/to/ion/dir",
  { from: 0, to: 20 },
  {
    eic: { ppmTolerance: 5 },
    grid: { start: 50, end: 1000, stepSize: 0.01 },
    grouping: {
      ppmTolerance: 5,
      mzTolerance: 0.0025,
      rtTolerance: 0.05,
      frequency: 2,
    },
    threads: 8,
  },
);
```

## Peak detection options

Pass peak detection options as an object:

```js
const options = {
  integralThreshold: 0.0,
  intensityThreshold: 150.0,
  widthThreshold: 5,
  noise: NaN,
  autoNoise: true,
  autoBaseline: true,
  baselineWindow: 0,
  baselineWindowFactor: 0,
  allowOverlap: false,
  windowSize: 0,
  minSnr: 3.0,
};
```
