sample_file <- sample_ion_path()
fixtures <- if (nzchar(sample_file)) dirname(dirname(sample_file)) else ""
model_file <- file.path(fixtures, "shift_model.onnx")
empty_file <- file.path(fixtures, "test.ion")
targets <- data.frame(id = c("first", "second"), mz = c(250.1, 310.2), rt = c(1.2, 3.1))

test_that("a warp model gives one prediction per target, in order", {
  skip_if(!nzchar(sample_file), "no api.ion sample next to the package")
  warp <- load_warp_model(model_file)
  bin <- parse_ion(sample_file)
  predicted <- predict_rts(warp, bin, targets)
  expect_equal(names(predicted), c("rt", "uncertainty"))
  expect_equal(nrow(predicted), nrow(targets))
  expect_true(all(is.finite(predicted$rt)))
  dispose(warp)
  dispose(bin)
})

test_that("a warp model from bytes or a URL predicts like one from a path", {
  skip_if_not_installed("callr")
  skip_if(!nzchar(sample_file), "no api.ion sample next to the package")
  bin <- parse_ion(sample_file)
  from_path <- predict_rts(load_warp_model(model_file), bin, targets)
  bytes <- readBin(model_file, "raw", file.info(model_file)$size)
  expect_identical(predict_rts(load_warp_model(bytes), bin, targets), from_path)
  with_served_file(model_file, FALSE, function(url) {
    expect_identical(predict_rts(load_warp_model(url), bin, targets), from_path)
  })
})

test_that("a remote file predicts like a local one", {
  skip_if_not_installed("callr")
  skip_if(!nzchar(sample_file), "no api.ion sample next to the package")
  warp <- load_warp_model(model_file)
  local <- predict_rts(warp, parse_ion(sample_file), targets)
  with_served_file(sample_file, TRUE, function(url) {
    expect_identical(predict_rts(warp, parse_ion_remote(url), targets), local)
  })
})

test_that("a target without mz is named by its index", {
  skip_if(!nzchar(sample_file), "no api.ion sample next to the package")
  broken <- data.frame(id = c("first", "broken"), mz = c(250.1, NA), rt = c(1.2, 2.0))
  expect_error(predict_rts(load_warp_model(model_file), parse_ion(sample_file), broken),
               "predict_rts failed: target 2 has no valid m/z or rt")
})

test_that("a file whose scans do not cover the model range is refused", {
  skip_if(!nzchar(sample_file), "no api.ion sample next to the package")
  expect_error(predict_rts(load_warp_model(model_file), parse_ion(empty_file), targets),
               "predict_rts failed: the file has no scans, the warp model needs")
})

test_that("a disposed warp model cannot be used", {
  skip_if(!nzchar(sample_file), "no api.ion sample next to the package")
  warp <- load_warp_model(model_file)
  dispose(warp)
  expect_error(predict_rts(warp, parse_ion(sample_file), targets), "disposed warp model")
})
