.DEFAULTS <- list(
  eic_ppm_tolerance     = 10.0,
  eic_mz_tolerance      = 0.005,
  grid_start            = 40,
  grid_end              = 1000,
  grid_step_size        = 0.005,
  group_ppm_tolerance   = 5.0,
  group_mz_tolerance    = 0.0025,
  group_rt_tolerance    = 0.05,
  frequency             = 1L,
  min_intensity         = 500,
  min_peak_width_points = 3L,
  auto_noise            = TRUE,
  auto_baseline         = TRUE,
  min_snr               = 2,
  min_r2                = 0.0,
  shape                 = "emg",
  kernel_size           = 0L
)

dispose <- function(object) {
  if (typeof(object) != "externalptr") stop("quantion: expected an external pointer")
  if (inherits(object, "quantion_warp_model"))
    return(invisible(.Call("C_dispose_warp_model", object, PACKAGE = "quantion")))
  if (inherits(object, "quantion_model"))
    return(invisible(.Call("C_dispose_model", object, PACKAGE = "quantion")))
  invisible(.Call("C_dispose_mzml", object, PACKAGE = "quantion"))
}

load_model <- function(source) {
  if (is.character(source) && length(source) == 1 && !is.na(source) &&
      (startsWith(tolower(source), "http://") || startsWith(tolower(source), "https://"))) {
    destination <- tempfile()
    on.exit(unlink(destination), add = TRUE)
    status <- tryCatch(
      utils::download.file(source, destination, method = "libcurl", quiet = TRUE, mode = "wb"),
      error = function(error) conditionMessage(error)
    )
    if (!identical(status, 0L))
      stop("quantion/load_model failed: the model was not read: GET ", source, " failed: ", status)
    source <- readBin(destination, "raw", file.info(destination)$size)
  }
  model <- if (is.raw(source)) {
    .Call("C_model_load_bytes", source, PACKAGE = "quantion")
  } else if (is.character(source) && length(source) == 1 && !is.na(source) && nchar(source) > 0) {
    .Call("C_model_load_path", path.expand(source), PACKAGE = "quantion")
  } else {
    stop("`source` must be a path, a URL, or raw bytes")
  }
  class(model) <- "quantion_model"
  attr(model, "metadata") <- .Call("C_model_metadata", model, PACKAGE = "quantion")
  model
}

load_warp <- function(model) {
  if (!inherits(model, "quantion_model")) stop("quantion: expected a model from load_model")
  warp <- .Call("C_warp_model_load", model, PACKAGE = "quantion")
  class(warp) <- "quantion_warp_model"
  attr(warp, "metadata") <- attr(model, "metadata")
  warp
}

get_metadata <- function(model) {
  if (!inherits(model, c("quantion_model", "quantion_warp_model")))
    stop("quantion: expected a model from load_model or a warp model from load_warp")
  attr(model, "metadata")
}

get_keys <- function(model) {
  get_metadata(model)$key
}

get_values <- function(model) {
  get_metadata(model)$value
}

get_value <- function(model, key) {
  metadata <- get_metadata(model)
  value <- metadata$value[metadata$key == key]
  if (length(value) == 0) NULL else value[[1]]
}

calibrate <- function(warp, anchor_samples, target_lists) {
  if (!inherits(warp, "quantion_warp_model")) stop("quantion: expected a warp model from load_warp")
  if (!is.list(anchor_samples) || !is.list(target_lists) || is.data.frame(target_lists))
    stop("`anchor_samples` and `target_lists` must be lists")
  if (length(anchor_samples) != length(target_lists))
    stop("calibrate needs one target list per sample, got ", length(anchor_samples),
         " samples and ", length(target_lists), " target lists")
  for (index in seq_along(target_lists)) {
    targets <- target_lists[[index]]
    if (!is.data.frame(targets) || !all(c("id", "mz", "rt") %in% names(targets)))
      stop("each target list must be a data.frame with columns id, mz and rt")
    prefetch_targets(warp, anchor_samples[[index]], targets)
  }
  get_column <- function(name, empty) {
    unlist(lapply(target_lists, function(targets)
      if (name %in% names(targets)) targets[[name]] else rep(empty, nrow(targets))), use.names = FALSE)
  }
  .Call("C_calibrate_rts", warp, anchor_samples,
        as.numeric(vapply(target_lists, nrow, integer(1))),
        as.character(get_column("id", "")), as.numeric(get_column("mz", NA)),
        as.numeric(get_column("rt", NA)), PACKAGE = "quantion")
}

prefetch_targets <- function(warp, file, targets) {
  source <- attr(file, "quantion_source")
  if (is.null(source) || nrow(targets) == 0) return(invisible(NULL))
  wanted <- .Call("C_plan_predict_rts", warp, file, as.numeric(targets$mz), as.numeric(targets$rt),
                  PACKAGE = "quantion")
  .prefetch(source, wanted)
}

predict_rts <- function(warp, file, targets) {
  if (!inherits(warp, "quantion_warp_model")) stop("quantion: expected a warp model from load_warp")
  if (typeof(file) != "externalptr") stop("quantion: expected an external pointer (MzML sample)")
  if (!is.data.frame(targets) || !all(c("mz", "rt") %in% names(targets)))
    stop("`targets` must be a data.frame with columns mz and rt")
  mzs <- as.numeric(targets$mz)
  rts <- as.numeric(targets$rt)
  if (length(mzs) == 0) return(data.frame(rt = numeric(0), uncertainty = numeric(0)))
  prefetch_targets(warp, file, targets)
  values <- .Call("C_predict_rts", warp, file, mzs, rts, PACKAGE = "quantion")
  data.frame(rt = values[c(TRUE, FALSE)], uncertainty = values[c(FALSE, TRUE)])
}

parse_mzml <- function(data) {
  stopifnot(is.raw(data))
  .Call("C_parse_mzml", data, PACKAGE="quantion")
}

mzml_to_ion <- function(file, level = 12L, f32_compress = FALSE) {
  if (typeof(file) != "externalptr") stop("quantion: expected an external pointer (MzML sample)")

  if (!is.numeric(level) || length(level) != 1 || is.na(level))
    stop("`level` must be a single number 0..22")
  lvl <- as.integer(level)
  if (lvl < 0L || lvl > 22L)
    stop("`level` must be between 0 and 22 (inclusive)")

  if (!is.logical(f32_compress) || length(f32_compress) != 1 || is.na(f32_compress))
    stop("`f32_compress` must be TRUE/FALSE")

  .Call("C_mzml_to_ion", file, lvl, f32_compress, PACKAGE = "quantion")
}

mzml_to_ion_file <- function(input_path, output_path, level = 12L, f32_compress = FALSE) {
  if (!is.character(input_path) || length(input_path) != 1 || is.na(input_path))
    stop("`input_path` must be a single string")
  if (!is.character(output_path) || length(output_path) != 1 || is.na(output_path))
    stop("`output_path` must be a single string")

  if (!is.numeric(level) || length(level) != 1 || is.na(level))
    stop("`level` must be a single number 0..22")
  lvl <- as.integer(level)
  if (lvl < 0L || lvl > 22L)
    stop("`level` must be between 0 and 22 (inclusive)")

  if (!is.logical(f32_compress) || length(f32_compress) != 1 || is.na(f32_compress))
    stop("`f32_compress` must be TRUE/FALSE")

  .Call("C_mzml_to_ion_file", input_path, output_path, lvl, f32_compress, PACKAGE = "quantion")
  invisible(output_path)
}

ion_to_mzml <- function(file) {
  if (typeof(file) != "externalptr") stop("quantion: expected an external pointer (MzML sample)")
  .Call("C_ion_to_mzml", file, PACKAGE = "quantion")
}

get_peak <- function(
  x, y, rt, range,
  min_integral          = NaN,
  min_intensity         = .DEFAULTS$min_intensity,
  min_peak_width_points = .DEFAULTS$min_peak_width_points,
  noise                 = NaN,
  auto_noise            = .DEFAULTS$auto_noise,
  auto_baseline         = .DEFAULTS$auto_baseline,
  lambda                = 0L,
  max_iterations        = 0L,
  allow_overlap         = FALSE,
  min_snr               = .DEFAULTS$min_snr,
  min_r2                = .DEFAULTS$min_r2,
  shape                 = .DEFAULTS$shape,
  kernel_size           = .DEFAULTS$kernel_size
) {
  stopifnot(is.numeric(x), is.numeric(y))
  if (length(x) != length(y) || length(x) < 3) stop("x and y must have the same length (>= 3)")
  if (!is.logical(auto_noise) || length(auto_noise) != 1 || is.na(auto_noise)) stop("auto_noise must be logical TRUE/FALSE")
  if (!is.logical(allow_overlap) || length(allow_overlap) != 1 || is.na(allow_overlap)) stop("allow_overlap must be logical TRUE/FALSE")
  if (!is.logical(auto_baseline) || length(auto_baseline) != 1 || is.na(auto_baseline)) stop("auto_baseline must be logical TRUE/FALSE")
  filter_supplied <- !(missing(min_integral) && missing(min_intensity) &&
    missing(min_peak_width_points) && missing(noise) && missing(auto_noise) &&
    missing(auto_baseline) && missing(lambda) && missing(max_iterations) &&
    missing(allow_overlap) && missing(min_snr) && missing(min_r2) && missing(shape) &&
    missing(kernel_size))
  opt <- if (filter_supplied) list(
    min_integral=min_integral, min_intensity=min_intensity,
    min_peak_width_points=min_peak_width_points, noise=noise,
    auto_noise=auto_noise, auto_baseline=auto_baseline,
    lambda=lambda, max_iterations=max_iterations,
    allow_overlap=allow_overlap, min_snr=min_snr, min_r2=min_r2, shape=shape,
    kernel_size=kernel_size
  ) else NULL
  rows <- .Call("C_get_peak", as.numeric(x), as.numeric(y), as.numeric(rt), as.numeric(range), opt, PACKAGE="quantion")
  if (nrow(rows) == 0) NULL else as.list(rows[1, ])
}

find_targets <- function(
  file, targets, from=0.5, to=5, threads=1L,
  min_integral          = NaN,
  min_intensity         = .DEFAULTS$min_intensity,
  min_peak_width_points = .DEFAULTS$min_peak_width_points,
  noise                 = NaN,
  auto_noise            = .DEFAULTS$auto_noise,
  auto_baseline         = .DEFAULTS$auto_baseline,
  lambda                = 0L,
  max_iterations        = 0L,
  allow_overlap         = FALSE,
  min_snr               = .DEFAULTS$min_snr,
  min_r2                = .DEFAULTS$min_r2,
  shape                 = .DEFAULTS$shape,
  kernel_size           = .DEFAULTS$kernel_size
) {
  stopifnot(typeof(file) == "externalptr")
  if (!is.data.frame(targets)) stop("`targets` must be a data.frame")
  req <- c("id","rt","mz","range"); miss <- setdiff(req, names(targets))
  if (length(miss)) stop(paste0("missing columns: ", paste(miss, collapse=", ")))
  id <- as.character(targets$id)
  rts <- suppressWarnings(as.numeric(targets$rt))
  mzs <- suppressWarnings(as.numeric(targets$mz))
  ranges <- suppressWarnings(as.numeric(targets$range))
  n <- length(id)
  if (!(length(rts)==n && length(mzs)==n && length(ranges)==n)) stop("id, rt, mz, range must have the same length")
  id[is.na(id)] <- ""
  if (!is.logical(auto_noise) || length(auto_noise) != 1 || is.na(auto_noise)) stop("auto_noise must be logical TRUE/FALSE")
  if (!is.logical(allow_overlap) || length(allow_overlap) != 1 || is.na(allow_overlap)) stop("allow_overlap must be logical TRUE/FALSE")
  if (!is.logical(auto_baseline) || length(auto_baseline) != 1 || is.na(auto_baseline)) stop("auto_baseline must be logical TRUE/FALSE")
  filter_supplied <- !(missing(min_integral) && missing(min_intensity) &&
    missing(min_peak_width_points) && missing(noise) && missing(auto_noise) &&
    missing(auto_baseline) && missing(lambda) && missing(max_iterations) &&
    missing(allow_overlap) && missing(min_snr) && missing(min_r2) && missing(shape) &&
    missing(kernel_size))
  opt <- if (filter_supplied) list(
    min_integral=min_integral, min_intensity=min_intensity,
    min_peak_width_points=min_peak_width_points, noise=noise,
    auto_noise=auto_noise, auto_baseline=auto_baseline,
    lambda=lambda, max_iterations=max_iterations,
    allow_overlap=allow_overlap, min_snr=min_snr, min_r2=min_r2, shape=shape,
    kernel_size=kernel_size
  ) else NULL
  threads <- .validate_threads(threads)
  source <- attr(file, "quantion_source")
  if (!is.null(source)) {
    .prefetch(source, .Call("C_plan_find_targets", file, as.numeric(mzs),
                            as.numeric(from), as.numeric(to), PACKAGE = "quantion"))
  }
  out_json <- .Call("C_find_targets",
    file, as.numeric(rts), as.numeric(mzs), as.numeric(ranges), as.character(id),
    as.numeric(from), as.numeric(to), opt, as.integer(threads),
    PACKAGE="quantion"
  )
  .as_peak_table(out_json)
}

find_transitions <- function(
  file, targets, threads=1L,
  min_integral          = NaN,
  min_intensity         = .DEFAULTS$min_intensity,
  min_peak_width_points = .DEFAULTS$min_peak_width_points,
  noise                 = NaN,
  auto_noise            = .DEFAULTS$auto_noise,
  auto_baseline         = .DEFAULTS$auto_baseline,
  lambda                = 0L,
  max_iterations        = 0L,
  allow_overlap         = FALSE,
  min_snr               = .DEFAULTS$min_snr,
  min_r2                = .DEFAULTS$min_r2,
  shape                 = .DEFAULTS$shape
) {
  stopifnot(typeof(file) == "externalptr")
  if (!is.data.frame(targets)) stop("`targets` must be a data.frame")
  missing_columns <- setdiff(c("id", "precursor", "product", "rt", "range"), names(targets))
  if (length(missing_columns)) stop(paste0("missing columns: ", paste(missing_columns, collapse = ", ")))
  ids <- as.character(targets$id)
  ids[is.na(ids)] <- ""
  if (!is.logical(auto_noise) || length(auto_noise) != 1 || is.na(auto_noise)) stop("auto_noise must be logical TRUE/FALSE")
  if (!is.logical(allow_overlap) || length(allow_overlap) != 1 || is.na(allow_overlap)) stop("allow_overlap must be logical TRUE/FALSE")
  if (!is.logical(auto_baseline) || length(auto_baseline) != 1 || is.na(auto_baseline)) stop("auto_baseline must be logical TRUE/FALSE")
  filter_supplied <- !(missing(min_integral) && missing(min_intensity) &&
    missing(min_peak_width_points) && missing(noise) && missing(auto_noise) &&
    missing(auto_baseline) && missing(lambda) && missing(max_iterations) &&
    missing(allow_overlap) && missing(min_snr) && missing(min_r2) && missing(shape))
  opt <- if (filter_supplied) list(
    min_integral=min_integral, min_intensity=min_intensity,
    min_peak_width_points=min_peak_width_points, noise=noise,
    auto_noise=auto_noise, auto_baseline=auto_baseline,
    lambda=lambda, max_iterations=max_iterations,
    allow_overlap=allow_overlap, min_snr=min_snr, min_r2=min_r2, shape=shape
  ) else NULL
  threads <- .validate_threads(threads)
  out_json <- .Call("C_find_transitions",
    file, as.numeric(targets$precursor), as.numeric(targets$product),
    as.numeric(targets$rt), as.numeric(targets$range), ids, opt, as.integer(threads),
    PACKAGE = "quantion"
  )
  .as_peak_table(out_json)
}

.as_peak_table <- function(peaks) {
  peaks <- as.data.frame(peaks)
  peaks <- peaks[, c("rt", "from", "to", "intensity", "integral", "n_points", "noise", "r2"), drop = FALSE]
  rownames(peaks) <- NULL
  peaks
}

calculate_eic <- function(file, mz, from, to, ppm_tolerance=20, mz_tolerance=0.005) {
  if (typeof(file) != "externalptr") stop("quantion: expected an external pointer (MzML sample)")
  stopifnot(is.numeric(mz), length(mz) == 1)
  source <- attr(file, "quantion_source")
  if (!is.null(source)) {
    wanted <- .Call("C_plan_eic", file, as.numeric(mz), as.numeric(from),
                    as.numeric(to), as.numeric(ppm_tolerance), as.numeric(mz_tolerance),
                    PACKAGE = "quantion")
    .prefetch(source, wanted)
  }
  .Call("C_calculate_eic",
    file, as.numeric(mz), as.numeric(from), as.numeric(to),
    as.numeric(ppm_tolerance), as.numeric(mz_tolerance),
    PACKAGE="quantion"
  )
}

find_peaks <- function(
  x, y,
  min_integral          = NaN,
  min_intensity         = .DEFAULTS$min_intensity,
  min_peak_width_points = .DEFAULTS$min_peak_width_points,
  noise                 = NaN,
  auto_noise            = .DEFAULTS$auto_noise,
  auto_baseline         = .DEFAULTS$auto_baseline,
  lambda                = 0L,
  max_iterations        = 0L,
  allow_overlap         = FALSE,
  min_snr               = .DEFAULTS$min_snr,
  min_r2                = .DEFAULTS$min_r2,
  shape                 = .DEFAULTS$shape,
  kernel_size           = .DEFAULTS$kernel_size
) {
  stopifnot(is.numeric(x), is.numeric(y))
  if (length(x) != length(y) || length(x) < 3) stop("x and y must have the same length >= 3")
  if (!is.logical(auto_noise) || length(auto_noise) != 1 || is.na(auto_noise)) stop("auto_noise must be logical TRUE/FALSE")
  if (!is.logical(allow_overlap) || length(allow_overlap) != 1 || is.na(allow_overlap)) stop("allow_overlap must be logical TRUE/FALSE")
  if (!is.logical(auto_baseline) || length(auto_baseline) != 1 || is.na(auto_baseline)) stop("auto_baseline must be logical TRUE/FALSE")
  filter_supplied <- !(missing(min_integral) && missing(min_intensity) &&
    missing(min_peak_width_points) && missing(noise) && missing(auto_noise) &&
    missing(auto_baseline) && missing(lambda) && missing(max_iterations) &&
    missing(allow_overlap) && missing(min_snr) && missing(min_r2) && missing(shape) &&
    missing(kernel_size))
  opt <- if (filter_supplied) list(
    min_integral=min_integral, min_intensity=min_intensity,
    min_peak_width_points=min_peak_width_points, noise=noise,
    auto_noise=auto_noise, auto_baseline=auto_baseline,
    lambda=lambda, max_iterations=max_iterations,
    allow_overlap=allow_overlap, min_snr=min_snr, min_r2=min_r2, shape=shape,
    kernel_size=kernel_size
  ) else NULL
  out_json <- .Call("C_find_peaks", as.numeric(x), as.numeric(y), opt, PACKAGE="quantion")
  out_json
}

calculate_baseline <- function(y, lambda=0L, max_iterations=0L) {
  stopifnot(is.numeric(y))
  .Call("C_calculate_baseline",
        as.numeric(y),
        as.integer(lambda),
        as.integer(max_iterations),
        PACKAGE="quantion")
}

find_noise_level <- function(y) .Call("C_find_noise_level", as.numeric(y), PACKAGE="quantion")

.shape_code <- function(shape) {
  s <- tolower(as.character(shape))
  if (s == "gaussian") return(0L)
  if (s == "emg")      return(1L)
  stop("shape must be 'gaussian' or 'emg'")
}

fit_peak <- function(x, y, rt, intensity, shape = "emg") {
  stopifnot(is.numeric(x), is.numeric(y), length(x) == length(y), length(x) >= 5)
  rows <- .Call("C_fit_peak",
        as.numeric(x), as.numeric(y),
        as.numeric(rt), as.numeric(intensity), .shape_code(shape),
        PACKAGE="quantion")
  if (nrow(rows) == 0) return(NULL)
  fit <- as.list(rows[1, ])
  fit$shape <- c("gaussian", "emg")[fit$shape + 1]
  fit
}

draw_peak <- function(x, params) {
  stopifnot(is.numeric(x), is.list(params))
  tail <- if (is.null(params$tail)) 0 else params$tail
  .Call("C_draw_peak",
        as.numeric(x), .shape_code(params$shape),
        as.numeric(params$height), as.numeric(params$center),
        as.numeric(params$fwhm), as.numeric(tail),
        PACKAGE="quantion")
}

find_features <- function(
  file, from, to,
  eic_ppm_tolerance     = .DEFAULTS$eic_ppm_tolerance,
  eic_mz_tolerance      = .DEFAULTS$eic_mz_tolerance,
  grid_start            = .DEFAULTS$grid_start,
  grid_end              = .DEFAULTS$grid_end,
  grid_step_size        = .DEFAULTS$grid_step_size,
  threads               = 1L,
  min_integral          = NaN,
  min_intensity         = .DEFAULTS$min_intensity,
  min_peak_width_points = .DEFAULTS$min_peak_width_points,
  noise                 = NaN,
  auto_noise            = .DEFAULTS$auto_noise,
  auto_baseline         = .DEFAULTS$auto_baseline,
  lambda                = 0L,
  max_iterations        = 0L,
  allow_overlap         = FALSE,
  min_snr               = .DEFAULTS$min_snr,
  min_r2                = .DEFAULTS$min_r2,
  shape                 = .DEFAULTS$shape
) {
  stopifnot(typeof(file) == "externalptr")
  threads <- .validate_threads(threads)

  opt <- list(
    min_integral=min_integral, min_intensity=min_intensity,
    min_peak_width_points=min_peak_width_points, noise=noise,
    auto_noise=auto_noise, auto_baseline=auto_baseline,
    lambda=lambda, max_iterations=max_iterations,
    allow_overlap=allow_overlap, min_snr=min_snr, min_r2=min_r2, shape=shape
  )

  source <- attr(file, "quantion_source")
  if (!is.null(source)) {
    .prefetch(source, .Call("C_plan_find_features", file,
                            as.numeric(from), as.numeric(to),
                            as.numeric(eic_ppm_tolerance), as.numeric(eic_mz_tolerance),
                            as.numeric(grid_start), as.numeric(grid_end), as.numeric(grid_step_size),
                            PACKAGE = "quantion"))
  }
  out_json <- .Call(
    "C_find_features", file,
    as.numeric(from), as.numeric(to),
    as.numeric(eic_ppm_tolerance), as.numeric(eic_mz_tolerance),
    as.numeric(grid_start), as.numeric(grid_end), as.numeric(grid_step_size),
    opt, as.integer(threads),
    PACKAGE = "quantion"
  )

  df <- out_json
  if (!is.data.frame(df)) df <- as.data.frame(df)
  want <- c("mz","rt","from","to","intensity","integral","n_points")
  present <- intersect(want, names(df))
  df <- df[, c(present, setdiff(names(df), present)), drop = FALSE]
  rownames(df) <- NULL
  df
}

parse_ion <- function(source, max_cache_size = 0) {
  if (is.character(source)) {
    looks_like_url <- length(source) == 1 && !is.na(source) && grepl("^https?://", source, ignore.case = TRUE)
    if (looks_like_url) return(.parse_ion_remote(source, max_cache_size))
    return(.parse_ion_path(source, max_cache_size))
  }
  .parse_ion_raw(source, max_cache_size)
}

.parse_ion_raw <- function(source, max_cache_size = 0) {
  if (!is.raw(source)) stop("`source` must be a raw vector of ion bytes")
  if (!is.numeric(max_cache_size) || length(max_cache_size) != 1 || is.na(max_cache_size) || max_cache_size < 0)
    stop("`max_cache_size` must be a single non-negative number")
  .Call("C_parse_ion", source, as.numeric(max_cache_size), PACKAGE = "quantion")
}

.parse_ion_path <- function(path, max_cache_size = 0) {
  if (!is.character(path) || length(path) != 1 || is.na(path) || nchar(path) == 0)
    stop("`path` must be a single non-empty string")
  full_path <- path.expand(path)
  if (!file.exists(full_path)) stop("`path` does not point to an existing file: ", path)
  if (!is.numeric(max_cache_size) || length(max_cache_size) != 1 || is.na(max_cache_size) || max_cache_size < 0)
    stop("`max_cache_size` must be a single non-negative number")
  .Call("C_parse_ion_path", full_path, as.numeric(max_cache_size), PACKAGE = "quantion")
}

.fetch_range <- function(url, offset, span) {
  destination <- tempfile()
  on.exit(unlink(destination), add = TRUE)
  last <- offset + span - 1
  status <- utils::download.file(
    url, destination, method = "libcurl", quiet = TRUE, mode = "wb",
    headers = c(Range = sprintf("bytes=%.0f-%.0f", offset, last))
  )
  if (!identical(status, 0L)) stop("quantion: the range request failed for ", url)
  bytes <- readBin(destination, "raw", file.info(destination)$size)
  if (length(bytes) != span)
    stop(sprintf("quantion: the server returned %d bytes for a %d byte range; it does not support range requests",
                 length(bytes), span))
  bytes
}

.LARGEST_GAP <- 131072

.gap_for <- function(total) {
  min(.LARGEST_GAP, total / 8)
}

.merge_ranges <- function(ranges, gap = .LARGEST_GAP) {
  wanted <- ranges[ranges[, 2] > 0, , drop = FALSE]
  if (nrow(wanted) == 0) return(wanted)
  wanted <- wanted[order(wanted[, 1]), , drop = FALSE]

  merged <- list(c(wanted[1, 1], wanted[1, 2]))
  for (row in seq_len(nrow(wanted))[-1]) {
    last <- merged[[length(merged)]]
    if (wanted[row, 1] - (last[1] + last[2]) <= gap) {
      end <- max(last[1] + last[2], wanted[row, 1] + wanted[row, 2])
      merged[[length(merged)]] <- c(last[1], end - last[1])
    } else {
      merged[[length(merged) + 1]] <- c(wanted[row, 1], wanted[row, 2])
    }
  }
  do.call(rbind, merged)
}

.already_held <- function(source, offset, span) {
  for (held in source$held) {
    if (offset >= held[1] && offset + span <= held[1] + held[2]) return(TRUE)
  }
  FALSE
}

.prefetch <- function(source, ranges) {
  wanted <- ranges[!apply(ranges, 1, function(row) .already_held(source, row[1], row[2])), , drop = FALSE]
  if (nrow(wanted) == 0) return(invisible(NULL))

  spans <- .merge_ranges(wanted, .gap_for(source$total))
  for (row in seq_len(nrow(spans))) {
    offset <- spans[row, 1]
    span <- spans[row, 2]
    if (.already_held(source, offset, span)) next
    bytes <- .fetch_range(source$url, offset, span)
    .Call("C_store_add", source$store, offset, bytes, PACKAGE = "quantion")
    source$held[[length(source$held) + 1]] <- c(offset, length(bytes))
  }
  invisible(NULL)
}

.parse_ion_remote <- function(url, max_cache_size = 0) {
  if (!is.character(url) || length(url) != 1 || is.na(url) || nchar(url) == 0)
    stop("`url` must be a single non-empty string")
  if (!startsWith(tolower(url), "http://") && !startsWith(tolower(url), "https://"))
    stop("`url` must start with http:// or https://")

  source <- new.env(parent = emptyenv())
  source$url <- url
  source$store <- .Call("C_store_new", PACKAGE = "quantion")
  source$held <- list()
  source$total <- 0

  header <- .fetch_range(url, 0, 1024)
  .Call("C_store_add", source$store, 0, header, PACKAGE = "quantion")
  source$held[[1]] <- c(0, length(header))

  planned <- .Call("C_plan_open", header, PACKAGE = "quantion")
  source$total <- if (nrow(planned)) max(planned[, 1] + planned[, 2]) else 0
  .prefetch(source, planned)

  handle <- .Call("C_parse_ion_source", source$store, as.numeric(max_cache_size),
                  PACKAGE = "quantion")
  attr(handle, "quantion_source") <- source
  handle
}

.QUERY_RT_RANGE   <- 0L
.QUERY_CLOSEST_RT <- 1L
.QUERY_MZ_RANGE   <- 2L
.QUERY_CLOSEST_MZ <- 3L

.get_scan_query <- function(value, name, range_query, closest_query) {
  value <- as.numeric(value)
  if (length(value) == 2) {
    if (!all(is.finite(value))) stop("get_scans: ", name, " must be c(from, to) with finite numbers")
    return(list(type = range_query, from = value[1], to = value[2]))
  }
  if (length(value) != 1 || !is.finite(value) || (name == "mz" && value <= 0))
    stop("get_scans: ", name, " must be one number or c(from, to)")
  list(type = closest_query, from = value, to = NaN)
}

get_scans <- function(file, rt = NULL, mz = NULL, level = 1L) {
  if (typeof(file) != "externalptr") stop("quantion: expected an external pointer (MzML sample)")
  if (!is.numeric(level) || length(level) != 1) stop("level must be a single numeric")
  if (is.null(rt) == is.null(mz)) stop("get_scans: give exactly one of rt or mz")

  query <- if (!is.null(rt)) {
    .get_scan_query(rt, "rt", .QUERY_RT_RANGE, .QUERY_CLOSEST_RT)
  } else {
    .get_scan_query(mz, "mz", .QUERY_MZ_RANGE, .QUERY_CLOSEST_MZ)
  }
  query_type <- query$type; a <- query$from; b <- query$to

  source <- attr(file, "quantion_source")
  if (!is.null(source)) {
    wanted <- .Call("C_plan_scans", file, as.integer(query_type), a, b,
                    as.integer(level), PACKAGE = "quantion")
    .prefetch(source, wanted)
  }

  scans <- .Call("C_get_scans",
    file, as.integer(query_type), a, b, as.integer(level),
    PACKAGE = "quantion"
  )
  if (query_type %in% c(.QUERY_CLOSEST_RT, .QUERY_CLOSEST_MZ)) {
    if (nrow(scans) == 0) {
      NULL
    } else {
      list(
        rt = scans$rt[[1]],
        mz = as.list(scans$mz[[1]]),
        intensity = as.list(scans$intensity[[1]]),
        metadata = as.list(scans$metadata[1, ])
      )
    }
  } else {
    scans
  }
}

match_features <- function(
  directory, from, to,
  eic_ppm_tolerance     = .DEFAULTS$eic_ppm_tolerance,
  eic_mz_tolerance      = .DEFAULTS$eic_mz_tolerance,
  grid_start            = .DEFAULTS$grid_start,
  grid_end              = .DEFAULTS$grid_end,
  grid_step_size        = .DEFAULTS$grid_step_size,
  group_ppm_tolerance   = .DEFAULTS$group_ppm_tolerance,
  group_mz_tolerance    = .DEFAULTS$group_mz_tolerance,
  group_rt_tolerance    = .DEFAULTS$group_rt_tolerance,
  frequency             = .DEFAULTS$frequency,
  threads               = 1L,
  min_integral          = NaN,
  min_intensity         = .DEFAULTS$min_intensity,
  min_peak_width_points = .DEFAULTS$min_peak_width_points,
  noise                 = NaN,
  auto_noise            = .DEFAULTS$auto_noise,
  auto_baseline         = .DEFAULTS$auto_baseline,
  lambda                = 0L,
  max_iterations        = 0L,
  allow_overlap         = FALSE,
  min_snr               = .DEFAULTS$min_snr,
  min_r2                = .DEFAULTS$min_r2,
  shape                 = .DEFAULTS$shape
) {
  if (!is.character(directory) || length(directory) != 1) stop("directory must be a single string")
  threads <- .validate_threads(threads)

  opt <- list(
    min_integral=min_integral, min_intensity=min_intensity,
    min_peak_width_points=min_peak_width_points, noise=noise,
    auto_noise=auto_noise, auto_baseline=auto_baseline,
    lambda=lambda, max_iterations=max_iterations,
    allow_overlap=allow_overlap, min_snr=min_snr, min_r2=min_r2, shape=shape
  )

  out_json <- .Call("C_match_features",
    directory,
    as.numeric(from), as.numeric(to),
    as.numeric(eic_ppm_tolerance), as.numeric(eic_mz_tolerance),
    as.numeric(grid_start), as.numeric(grid_end), as.numeric(grid_step_size),
    as.numeric(group_ppm_tolerance), as.numeric(group_mz_tolerance), as.numeric(group_rt_tolerance),
    as.integer(frequency),
    opt, as.integer(threads),
    PACKAGE = "quantion"
  )

  df <- out_json
  if (!is.data.frame(df)) df <- as.data.frame(df)
  rownames(df) <- NULL
  df
}

.validate_threads <- function(threads) {
  if (!is.numeric(threads) || length(threads) != 1L || is.na(threads)) stop("threads must be a single number")

  if (threads != floor(threads)) stop("threads must be a single number")

  threads <- as.integer(threads)
  if (is.na(threads) || threads < 1L) stop("threads must be a single number")

  maxc <- NA_integer_
  if (requireNamespace("parallel", quietly = TRUE)) {
    maxc <- suppressWarnings(tryCatch(parallel::detectCores(logical = FALSE), error = function(e) NA_integer_))
    if (is.na(maxc) || maxc < 1L) {
      maxc <- suppressWarnings(tryCatch(parallel::detectCores(logical = TRUE), error = function(e) NA_integer_))
    }
  }
  if (!is.na(maxc) && threads > maxc) {
    warning(sprintf("threads (%d) exceeds detected threads (%d)", threads, maxc))
  }

  threads
}

