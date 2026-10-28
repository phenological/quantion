sample_file <- sample_ion_path()

fit_on_sample <- function(shape) {
  sample <- parse_ion(sample_file)
  eic <- calculate_eic(sample, 89.04768, from = 0, to = 5)
  peak <- get_peak(eic$x, eic$y, rt = 2.385, range = 0.1)
  list(eic = eic, fit = fit_peak(eic$x, eic$y, peak$rt, peak$intensity, shape))
}

test_that("the fit names its shape", {
  skip_if(!nzchar(sample_file), "no api.ion sample next to the package")
  expect_identical(fit_on_sample("emg")$fit$shape, "emg")
  expect_identical(fit_on_sample("gaussian")$fit$shape, "gaussian")
})

test_that("a fit can be drawn", {
  skip_if(!nzchar(sample_file), "no api.ion sample next to the package")
  fitted <- fit_on_sample("emg")
  expect_length(draw_peak(fitted$eic$x, fitted$fit), length(fitted$eic$x))
})
