suppressMessages(library(quantion))

MASS <- 90.05550
FROM_RT <- 0.0
TO_RT <- 5.0
EIC_PPM <- 20.0
EIC_MZ <- 0.005

same_bits <- function(left, right) {
  if (length(left) != length(right)) return(FALSE)
  all(mapply(function(a, b) {
    identical(writeBin(a, raw(), size = 8), writeBin(b, raw(), size = 8))
  }, left, right))
}

.origin <- function(url) sub("^(https?://[^/]+).*$", "\\1", url)

.stats <- function(url, path) {
  text <- paste(readLines(paste0(.origin(url), path), warn = FALSE), collapse = "")
  numbers <- as.numeric(regmatches(text, gregexpr("[0-9]+", text))[[1]])
  list(bytes_sent = numbers[1], requests = numbers[2])
}

main <- function() {
  url <- Sys.getenv("QUANTION_REMOTE_URL", "")
  if (!nzchar(url)) {
    message("QUANTION_REMOTE_URL is not set")
    return(1L)
  }
  fixture <- "core/tests/fixtures/api/api.ion"

  invisible(.stats(url, "/reset"))
  remote <- parse_ion(url)
  opening <- .stats(url, "/stats")

  invisible(.stats(url, "/reset"))
  local <- parse_ion(fixture)

  eic_remote <- calculate_eic(remote, MASS, FROM_RT, TO_RT, EIC_PPM, EIC_MZ)
  eic_local <- calculate_eic(local, MASS, FROM_RT, TO_RT, EIC_PPM, EIC_MZ)
  one_query <- .stats(url, "/stats")

  total <- file.info(fixture)$size
  same <- same_bits(eic_remote$x, eic_local$x) && same_bits(eic_remote$y, eic_local$y)

  cat(sprintf("file_bytes = %.0f\n", total))
  cat(sprintf("opening_bytes = %.0f\n", opening$bytes_sent))
  cat(sprintf("opening_requests = %.0f\n", opening$requests))
  cat(sprintf("query_bytes = %.0f\n", one_query$bytes_sent))
  cat(sprintf("query_requests = %.0f\n", one_query$requests))
  cat(sprintf("matches_local = %s\n", if (same) "yes" else "no"))

  warp <- load_warp(load_model("core/tests/fixtures/shift_model.onnx"))
  target <- data.frame(id = "a", rt = 2.385, mz = 89.04768, range = 0.1)
  calls <- list(
    calculate_eic = function(file) calculate_eic(file, MASS, FROM_RT, TO_RT),
    get_scans = function(file) get_scans(file, rt = c(2.3, 2.5)),
    find_targets = function(file) find_targets(file, target, from = FROM_RT, to = TO_RT),
    find_transitions = function(file) find_transitions(file, data.frame(id = "a", precursor = 89.04768, product = 89.04768, rt = 2.385, range = 0.1)),
    find_features = function(file) find_features(file, from = FROM_RT, to = TO_RT),
    ion_to_mzml = function(file) ion_to_mzml(file),
    mzml_to_ion = function(file) mzml_to_ion(file),
    predict_rts = function(file) predict_rts(warp, file, data.frame(mz = 250.1, rt = 1.2))
  )
  for (name in names(calls)) {
    cat(sprintf("fresh_%s = %s\n", name, compare(calls[[name]], url, fixture)))
  }
  0L
}

compare <- function(call, url, fixture) {
  remote <- tryCatch(call(parse_ion(url)), error = function(e) e)
  if (inherits(remote, "error")) {
    text <- strsplit(conditionMessage(remote), "\n")[[1]][1]
    return(if (grepl("not a URL", text, fixed = TRUE)) "refused" else paste("error", substr(text, 1, 80)))
  }
  local <- call(parse_ion(fixture))
  if (identical(remote, local)) "same" else "differs"
}

quit(status = main())
