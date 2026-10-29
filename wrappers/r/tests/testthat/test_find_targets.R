sample_file <- sample_ion_path()

test_that("targets with a range column give one peak per target", {
  skip_if(!nzchar(sample_file), "no api.ion sample next to the package")
  targets <- data.frame(id = c("a", "b"), rt = c(2.385, 2.18), mz = c(89.04768, 89.04768), range = c(0.1, 0.1))
  peaks <- find_targets(parse_ion(sample_file), targets, from = 0, to = 5, threads = 2L)
  expect_equal(nrow(peaks), 2)
  expect_true(all(peaks$intensity > 0))
})

test_that("a ranges column is refused", {
  skip_if(!nzchar(sample_file), "no api.ion sample next to the package")
  targets <- data.frame(id = "a", rt = 2.385, mz = 89.04768, ranges = 0.1)
  expect_error(find_targets(parse_ion(sample_file), targets), "missing columns: range")
})
