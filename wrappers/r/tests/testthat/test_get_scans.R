sample_file <- sample_ion_path()

test_that("an rt range gives every scan inside it", {
  skip_if(!nzchar(sample_file), "no api.ion sample next to the package")
  scans <- get_scans(parse_ion(sample_file), rt = c(2.3, 2.5))
  expect_gt(nrow(scans), 0)
  expect_true(all(scans$rt >= 2.3 & scans$rt <= 2.5))
})

test_that("one rt gives the closest scan", {
  skip_if(!nzchar(sample_file), "no api.ion sample next to the package")
  scans <- get_scans(parse_ion(sample_file), rt = 2.385)
  expect_equal(nrow(scans), 1)
  expect_equal(scans$rt, 2.385, tolerance = 0.05)
  expect_true(all(c("level", "precursor_mz", "base_peak_intensity", "x", "y", "z") %in% names(scans)))
})

test_that("rt and mz together are refused", {
  skip_if(!nzchar(sample_file), "no api.ion sample next to the package")
  expect_error(get_scans(parse_ion(sample_file), rt = 2.385, mz = 89), "rt or mz must be given, not both")
})

test_that("three numbers are refused", {
  skip_if(!nzchar(sample_file), "no api.ion sample next to the package")
  expect_error(get_scans(parse_ion(sample_file), rt = c(1, 2, 3)), "get_scans: rt must be one number or c\\(from, to\\)")
})
