default_of <- function(work, name) eval(formals(work)[[name]], asNamespace("quantion"))

peak_finders <- function() list(
  find_peaks = quantion::find_peaks,
  get_peak = quantion::get_peak,
  match_features = quantion::match_features,
  find_targets = quantion::find_targets,
  find_transitions = quantion::find_transitions
)

shared_settings <- c("min_intensity", "min_peak_width_points", "auto_noise",
                     "auto_baseline", "min_snr", "min_r2", "shape")

test_that("find_transitions starts with the automatic noise and baseline on", {
  skip_if_not_installed("quantion")

  expect_true(default_of(quantion::find_transitions, "auto_noise"))
  expect_true(default_of(quantion::find_transitions, "auto_baseline"))
})

test_that("every peak finder starts from the same filter defaults", {
  skip_if_not_installed("quantion")

  finders <- peak_finders()
  for (name in shared_settings) {
    values <- lapply(finders, default_of, name = name)
    first <- values[[1]]
    for (value in values) expect_identical(value, first, info = name)
  }
})

test_that("the shared filter defaults are the ones the package publishes", {
  skip_if_not_installed("quantion")

  for (name in shared_settings) {
    expect_identical(
      default_of(quantion::find_transitions, name),
      quantion:::.DEFAULTS[[name]],
      info = name
    )
  }
})
