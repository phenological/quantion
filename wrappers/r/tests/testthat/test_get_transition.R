srm_file <- {
  root <- find_sample_root()
  if (nzchar(root)) file.path(root, "core", "tests", "fixtures", "srm.ion") else ""
}

alanine <- 89.04768
serine <- 90.0555

expect_refused <- function(query, argument) {
  error <- tryCatch(get_transition(parse_ion(srm_file), query), quantion_error = function(error) error)
  expect_identical(error$kind, "invalid_argument")
  expect_identical(error$argument, argument)
}

test_that("an index reads each stored chromatogram", {
  skip_if(!nzchar(srm_file), "no srm.ion fixture next to the package")
  sample <- parse_ion(srm_file)
  for (index in 0:1) {
    trace <- get_transition(sample, index)
    expect_gt(length(trace$x), 0)
    expect_equal(length(trace$y), length(trace$x))
  }
})

test_that("an index past the end gives an empty chromatogram", {
  skip_if(!nzchar(srm_file), "no srm.ion fixture next to the package")
  trace <- get_transition(parse_ion(srm_file), 2)
  expect_length(trace$x, 0)
  expect_length(trace$y, 0)
})

test_that("a transition reads the same chromatogram as its index", {
  skip_if(!nzchar(srm_file), "no srm.ion fixture next to the package")
  sample <- parse_ion(srm_file)
  trace <- get_transition(sample, list(precursor = alanine, product = alanine))
  by_index <- lapply(0:1, function(index) get_transition(sample, index)$y)
  expect_gt(length(trace$y), 0)
  expect_true(any(vapply(by_index, identical, logical(1), trace$y)))
})

test_that("a named vector works as a transition", {
  skip_if(!nzchar(srm_file), "no srm.ion fixture next to the package")
  sample <- parse_ion(srm_file)
  from_vector <- get_transition(sample, c(precursor = alanine, product = alanine))
  from_list <- get_transition(sample, list(precursor = alanine, product = alanine))
  expect_identical(from_vector$y, from_list$y)
})

test_that("a transition matches within the tolerance", {
  skip_if(!nzchar(srm_file), "no srm.ion fixture next to the package")
  sample <- parse_ion(srm_file)
  near <- get_transition(sample, list(precursor = 89.0526, product = 89.0436))
  exact <- get_transition(sample, list(precursor = alanine, product = alanine))
  expect_identical(near$y, exact$y)
})

test_that("a wrong product gives an empty chromatogram", {
  skip_if(!nzchar(srm_file), "no srm.ion fixture next to the package")
  trace <- get_transition(parse_ion(srm_file), list(precursor = alanine, product = serine))
  expect_length(trace$x, 0)
})

test_that("a bad query is refused", {
  skip_if(!nzchar(srm_file), "no srm.ion fixture next to the package")
  expect_refused(-1, "query")
  expect_refused(list(precursor = alanine), "query")
  expect_refused("0", "query")
  expect_refused(list(precursor = -1, product = alanine), "precursor")
})

read_metadata <- function(query, tags = character(0)) {
  get_transition_metadata(parse_ion(srm_file), query, get_transition_metadata_options(tags = tags))
}

test_that("an index reads every metadata row of its chromatogram", {
  skip_if(!nzchar(srm_file), "no srm.ion fixture next to the package")
  rows <- read_metadata(0)
  expect_identical(nrow(rows), 11L)
  expect_identical(rows$path[1:5], c("chromatogram", "chromatogram", "chromatogram",
                                     "chromatogram/precursor/isolationWindow",
                                     "chromatogram/product/isolationWindow"))
  expect_identical(rows$name[1:4], c("index", "id", "defaultArrayLength", "isolation window target m/z"))
  expect_identical(rows$value[1:5], c("0", "SRM SIC Q1=89.04768 Q3=89.04768", "1035", "89.04768", "89.04768"))
  expect_identical(rows$kind[4], "CV")
  expect_identical(rows$accession[4], "MS:1000827")
  expect_true(is.na(rows$accession[1]))
})

test_that("a transition reads the same metadata rows as its index", {
  skip_if(!nzchar(srm_file), "no srm.ion fixture next to the package")
  expect_identical(read_metadata(list(precursor = serine, product = serine)), read_metadata(1))
  expect_identical(read_metadata(c(precursor = 89.0526, product = 89.0436)), read_metadata(0))
})

test_that("tags keep only their metadata rows", {
  skip_if(!nzchar(srm_file), "no srm.ion fixture next to the package")
  expect_identical(nrow(read_metadata(0, "chromatogram")), 3L)
  expect_identical(read_metadata(0, "isolationWindow")$path,
                   c("chromatogram/precursor/isolationWindow", "chromatogram/product/isolationWindow"))
})

test_that("no matching chromatogram gives no metadata rows", {
  skip_if(!nzchar(srm_file), "no srm.ion fixture next to the package")
  expect_identical(nrow(read_metadata(2)), 0L)
  expect_identical(nrow(read_metadata(list(precursor = alanine, product = serine))), 0L)
})

test_that("bad metadata arguments are refused", {
  skip_if(!nzchar(srm_file), "no srm.ion fixture next to the package")
  tags_error <- tryCatch(get_transition_metadata_options(tags = "scan"), quantion_error = function(error) error)
  expect_identical(tags_error$argument, "tags")
  query_error <- tryCatch(read_metadata(-1), quantion_error = function(error) error)
  expect_identical(query_error$argument, "query")
  product_error <- tryCatch(read_metadata(list(precursor = alanine, product = -1)),
                            quantion_error = function(error) error)
  expect_identical(product_error$kind, "invalid_argument")
  expect_identical(product_error$argument, "product")
})
