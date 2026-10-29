# quantion

R wrappers for the quantion library for LC-MS data processing.

## Install

```r
remotes::install_github("phenological/quantion", subdir = "wrappers/r")
```

The install builds the native library from its Rust source, so it needs `cargo` and `rustc` (1.88 or newer).

## Concepts

**Sample**: an opened file. Create it with `parse_ion()` or `parse_mzml()`. Its memory is freed by the garbage collector, or at once with `dispose()`.

**Options**: every function with settings takes one `options` argument built by a helper named after it, such as `find_targets_options()`. Groups of settings nest through their own helpers: `calculate_eic_options()`, `grid_options()`, `grouping_options()` and `find_peaks_options()`. A misspelt or wrong setting is an error as soon as the helper runs.

**Errors**: every error is a condition of class `quantion_error` with `kind`, `code`, `function`, `argument` and `message`, the same as in the other quantion languages.

```r
result <- tryCatch(
  calculate_eic(sample, -1),
  quantion_error = function(error) error$kind
)
# "invalid_argument"
```

## Files

```r
sample <- quantion::parse_ion("sample.ion")
remote <- quantion::parse_ion("https://example.org/sample.ion")
mzml <- quantion::parse_mzml("sample.mzML")
```

A path or URL is read lazily: only the bytes a query needs are fetched. Functions that need the whole file (`find_transitions`, `write_ion`, `write_mzml`) refuse a sample opened from a URL with a `remote_file` error.

Write a sample as .ion or mzML. Without `out`, you get the bytes back.

```r
bytes <- quantion::write_ion(sample)
quantion::write_ion(mzml, options = quantion::write_ion_options(out = "sample.ion", level = 12L))
quantion::write_mzml(sample, options = quantion::write_mzml_options(out = "sample.mzML"))
```

## Extract

```r
eic <- quantion::calculate_eic(sample, 260.103,
  options = quantion::calculate_eic_options(from = 0.5, to = 5, ppm_tolerance = 10))

scans <- quantion::get_scans(sample, rt = c(1.0, 2.5))
closest <- quantion::get_scans(sample, rt = 108,
  options = quantion::get_scans_options(level = 2L, time_unit = "SECONDS"))

image <- quantion::get_ion_image(sample, 760.585,
  options = quantion::get_ion_image_options(tolerance = 0.05))
```

`calculate_eic_options()` reads the whole run by default (`from = 0`, `to = Inf`) with 20 ppm or 0.005 Da, whichever is wider. `time_unit` is `"MINUTES"` or `"SECONDS"`.

## Traces

```r
peaks <- quantion::find_peaks(eic$x, eic$y,
  options = quantion::find_peaks_options(min_intensity = 1000, min_snr = 3))

peak <- quantion::get_peak(eic$x, eic$y, rt = 2.383,
  options = quantion::get_peak_options(range = 0.05))
anywhere <- quantion::get_peak(eic$x, eic$y, rt = 2.383)

fit <- quantion::fit_peak(eic$x, eic$y, rt = 2.383)
noise <- quantion::find_noise_level(eic$y)$intensity
baseline <- quantion::calculate_baseline(eic$y, options = quantion::calculate_baseline_options(lambda = 50))
```

A `get_peak` without a range searches the whole trace. No peak gives a peak with every field 0.

Simulate a chromatogram from peak curves. A `tail` of 0 draws a Gaussian, above 0 an EMG.

```r
peak_1 <- list(height = 9800, center = 2.38, fwhm = 0.021, tail = 0.006)
peak_2 <- list(height = 4200, center = 2.45, fwhm = 0.018, tail = 0)
chrom <- quantion::draw_peaks(list(peak_1, peak_2),
  options = quantion::draw_peaks_options(from = 2, to = 3, points = 2000L))
```

## Targets

Find the peak of each target in its EIC. `id` and `range` are optional; a target without `range` searches its whole EIC.

```r
targets <- data.frame(
  id    = c("compound_1", "compound_2"),
  mz    = c(260.103, 260.103),
  rt    = c(2.383, 2.189),
  range = c(0.05, 0.05)
)
peaks <- quantion::find_targets(sample, targets, options = quantion::find_targets_options(
  eic        = quantion::calculate_eic_options(from = 0.5, to = 5),
  find_peaks = quantion::find_peaks_options(min_intensity = 1000),
  threads    = 4L
))
```

Find peaks in the SRM chromatograms stored in a sample. Each transition picks the chromatogram whose precursor (Q1) and product (Q3) are both within 0.005 m/z.

```r
transitions <- data.frame(
  id        = c("compound_1", "compound_2"),
  precursor = c(500.3, 520.4),
  product   = c(184.1, 184.1),
  rt        = c(3.20, 3.65),
  range     = c(0.4, 0.4)
)
peaks <- quantion::find_transitions(sample, transitions,
  options = quantion::find_transitions_options(threads = 2L))
```

## Features

```r
features <- quantion::find_features(sample, options = quantion::find_features_options(
  grid    = quantion::grid_options(start = 100, end = 600),
  threads = 4L
))

matched <- quantion::match_features("samples/", options = quantion::match_features_options(
  grouping = quantion::grouping_options(min_frequency = 0.5),
  threads  = 8L
))
```

`frequency` in the matched table is the fraction of samples (0 to 1) where a feature was found; `min_frequency` keeps only features found in at least that fraction.

## Predict retention times

```r
model <- quantion::load_model("model.onnx")
quantion::get_value(model, "points")

warp <- quantion::load_warp(model)
predicted <- quantion::predict_rts(warp, sample, data.frame(id = "compound_1", mz = 260.103, rt = 2.383))
```

Without reference retention times, calibrate from anchor samples such as LTRs or QCs: give the samples and, for each one, the targets seen in it with their measured `rt`. Each target gets the median over the anchors.

```r
calibrated <- quantion::calibrate(warp, list(anchor_1, anchor_2), list(seen_in_anchor_1, seen_in_anchor_2))
predicted <- quantion::predict_rts(warp, sample, calibrated)

quantion::dispose(warp)
quantion::dispose(model)
```
