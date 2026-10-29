# quantion-py

Python wrappers for the quantion Rust library for LC-MS data processing.

## Install

```bash
pip install git+https://github.com/phenological/quantion#subdirectory=wrappers/python
```

The install downloads the native library for your platform from the GitHub release that matches the package version.

## Concepts

**Sample**: an opened .ion or mzML file, from `parse_ion()` or `parse_mzml()`. Use it as a context manager (`with`) to free it automatically, or call `dispose()`.

**Options**: every function with settings takes one `options` dict. Leave a key out to use its default. An unknown key raises `QuantionError` with kind `"unknown_option"`.

**Errors**: every failure raises `quantion.QuantionError`, with `kind`, `code`, `function`, `argument` and a message like `"calculate_eic: mz must be greater than 0, got -1"`.

## Quick start

```python
import quantion

sample = quantion.parse_ion("sample.ion")
eic = quantion.calculate_eic(sample, 174.112, {"from": 1, "to": 15})
peaks = quantion.find_peaks(eic["x"], eic["y"], {"min_snr": 3})
sample.dispose()

for peak in peaks:
    print(f"rt={peak['rt']:.3f}  intensity={peak['intensity']:.0f}")
```

The library loads automatically on import.

## Files

### Open

`parse_ion` reads a path, an http(s) URL or bytes. Paths and URLs are read lazily: only the bytes a query needs are fetched.

```python
sample = quantion.parse_ion("sample.ion")
remote = quantion.parse_ion("https://example.org/sample.ion", {"max_cache_size": 64_000_000})
```

`parse_mzml` reads a path or bytes and loads the whole file. URLs are refused.

```python
sample = quantion.parse_mzml("sample.mzML")
```

### Write

Without `out` you get the bytes back. With a path the file is written; with a URL it is uploaded in one PUT request.

```python
data = quantion.write_ion(sample)
quantion.write_ion(sample, {"out": "sample.ion", "level": 12})
quantion.write_ion(sample, {
    "out": "https://example.org/upload/sample.ion",
    "headers": [{"name": "Authorization", "value": "Bearer <token>"}],
})
quantion.write_mzml(sample, {"out": "sample.mzML"})
```

### Scans

A range gives every scan in it; one number gives a list with the closest scan.

```python
scans = quantion.get_scans(sample, rt={"from": 1.0, "to": 2.5})
closest = quantion.get_scans(sample, rt=108, options={"level": 2, "time_unit": quantion.TimeUnit.SECONDS})
```

### EIC

The range is `from` and `to` in the options; the default is the whole run.

```python
eic = quantion.calculate_eic(sample, 174.112, {"from": 1, "to": 15, "ppm_tolerance": 10})
```

### Ion image

Only .ion files.

```python
image = quantion.get_ion_image(sample, 760.585, {"tolerance": 0.05})
```

## Traces

```python
peaks = quantion.find_peaks(x, y, {"min_snr": 3})
peak = quantion.get_peak(x, y, 5.3, {"range": 0.6})
anywhere = quantion.get_peak(x, y, 5.3)
params = quantion.fit_peak(x, y, 5.3, {"shape": quantion.PeakShape.GAUSSIAN})
chromatogram = quantion.draw_peaks([params], {"from": 4.5, "to": 6.0, "points": 500})
noise = quantion.find_noise_level(y)["intensity"]
baseline = quantion.calculate_baseline(y, {"lambda": 50})
```

`get_peak` without a range searches the whole trace. A peak with every field 0 means no peak was found.

## Targets

```python
targets = [
    {"id": "compound_A", "mz": 174.112, "rt": 5.3, "range": 0.5},
    {"id": "compound_B", "mz": 203.156, "rt": 7.1, "range": 0.5},
]
peaks = quantion.find_targets(sample, targets, {
    "eic": {"from": 0.5, "to": 10.0},
    "find_peaks": {"min_intensity": 1000},
    "threads": 2,
})
```

Stored SRM chromatograms: each transition picks the chromatogram whose precursor (Q1) and product (Q3) m/z are both within 0.005. Needs the whole file.

```python
transitions = [{"id": "compound_1", "precursor": 500.3, "product": 184.1, "rt": 3.20, "range": 0.4}]
peaks = quantion.find_transitions(sample, transitions, {"threads": 2})
```

## Features

```python
features = quantion.find_features(sample, {
    "grid": {"start": 100, "end": 600},
    "threads": 4,
})

matched = quantion.match_features("samples/", {
    "grouping": {"min_frequency": 0.5},
    "threads": 8,
})
```

`frequency` in each matched feature is the fraction of samples where it was found.

## Retention-time models

```python
model = quantion.load_model("model.onnx")
model.get_value("points")

warp = quantion.load_warp(model)
rts = warp.predict_rts(sample, [{"id": "feature_1", "mz": 300.1, "rt": 2.38}])

calibrated = warp.calibrate([ltr_1, ltr_2, ltr_3], [seen_in_ltr_1, seen_in_ltr_2, seen_in_ltr_3])
rts = warp.predict_rts(other_sample, calibrated)

warp.dispose()
model.dispose()
```

Each rt given to `calibrate` must be the one measured in that anchor sample.

## Options and defaults

| Options | Keys (defaults) |
|---|---|
| `FindPeaksOptions` | `min_integral` (off), `min_intensity` (500), `min_peak_width_points` (3), `noise` (off), `auto_noise` (True), `auto_baseline` (True), `lambda` (20), `max_iterations` (4), `allow_overlap` (False), `min_snr` (2), `min_r2` (0), `shape` ("EMG"), `kernel_size` (0) |
| `CalculateEicOptions` | `from` (0), `to` (inf), `ppm_tolerance` (20), `mz_tolerance` (0.005), `time_unit` ("MINUTES") |
| `GridOptions` | `start` (40), `end` (1000), `step_size` (0.005) |
| `GroupingOptions` | `ppm_tolerance` (5), `mz_tolerance` (0.0025), `rt_tolerance` (0.05), `min_frequency` (0) |
| `GetPeakOptions` | `range` (0), `find_peaks` |
| `FitPeakOptions` | `shape` ("EMG"), `intensity` (0 = taken from the peak) |
| `DrawPeaksOptions` | `from` (0), `to` (0; both 0 fit the range around the peaks), `points` (1000) |
| `CalculateBaselineOptions` | `lambda` (20), `max_iterations` (4) |
| `FindTargetsOptions` | `eic`, `find_peaks`, `threads` (1) |
| `FindTransitionsOptions` | `find_peaks`, `threads` (1) |
| `FindFeaturesOptions` | `eic`, `grid`, `find_peaks`, `threads` (1) |
| `MatchFeaturesOptions` | `eic`, `grid`, `grouping`, `find_peaks`, `threads` (1) |
| `GetScansOptions` | `level` (1), `time_unit` ("MINUTES") |
| `ParseIonOptions` | `max_cache_size` (0 = no limit) |
| `WriteIonOptions` | `out`, `overwrite` (False), `level` (12), `f32_compress` (False), `headers` ([]) |
| `WriteMzmlOptions` | `out`, `overwrite` (False) |
| `GetIonImageOptions` | `tolerance` (0.125), `level` (1), `on_progress` |

`TimeUnit` is `"MINUTES"` or `"SECONDS"`; `PeakShape` is `"GAUSSIAN"` or `"EMG"`. Both accept the enum or the uppercase text.
