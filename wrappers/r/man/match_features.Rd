\name{match_features}
\alias{match_features}
\title{Find and align features across many files}
\description{
Detects features in every file in a folder and groups them by m/z and RT.
Returns a consensus feature table aligned across all samples.
}
\usage{
match_features(
  directory,
  from,
  to,
  eic_ppm_tolerance = .DEFAULTS$eic_ppm_tolerance,
  eic_mz_tolerance = .DEFAULTS$eic_mz_tolerance,
  grid_start = .DEFAULTS$grid_start,
  grid_end = .DEFAULTS$grid_end,
  grid_step_size = .DEFAULTS$grid_step_size,
  group_ppm_tolerance = .DEFAULTS$group_ppm_tolerance,
  group_mz_tolerance = .DEFAULTS$group_mz_tolerance,
  group_rt_tolerance = .DEFAULTS$group_rt_tolerance,
  frequency = .DEFAULTS$frequency,
  threads = 1L,
  min_integral = NaN,
  min_intensity = .DEFAULTS$min_intensity,
  min_peak_width_points = .DEFAULTS$min_peak_width_points,
  noise = NaN,
  auto_noise = .DEFAULTS$auto_noise,
  auto_baseline = .DEFAULTS$auto_baseline,
  lambda = 0L,
  max_iterations = 0L,
  allow_overlap = FALSE,
  min_snr = .DEFAULTS$min_snr,
  min_r2 = .DEFAULTS$min_r2,
  shape = .DEFAULTS$shape
)
}
\arguments{
  \item{directory}{Path to a folder with mzML or ion files.}
  \item{from}{Start retention time (minutes).}
  \item{to}{End retention time (minutes).}
  \item{eic_ppm_tolerance}{PPM tolerance for EIC extraction. Default 10.}
  \item{eic_mz_tolerance}{Absolute m/z tolerance (Da) for EIC extraction. Default 0.005.}
  \item{grid_start}{Lower m/z bound of the search grid (Da). Default 40.}
  \item{grid_end}{Upper m/z bound of the search grid (Da). Default 1000.}
  \item{grid_step_size}{Grid step size (ppm). Default 0.005.}
  \item{group_ppm_tolerance}{PPM tolerance used when grouping features across samples. Default 5.}
  \item{group_mz_tolerance}{Absolute m/z tolerance (Da) used when grouping features. Default 0.0025.}
  \item{group_rt_tolerance}{RT tolerance (minutes) used when grouping features. Default 0.05.}
  \item{frequency}{Minimum number of samples in which a feature must appear to be kept. Default 1.}
  \item{threads}{Number of threads to use. Default 1.}
  \item{min_integral}{Minimum peak area. \code{NaN} disables.}
  \item{min_intensity}{Minimum apex intensity. Default 1000.}
  \item{min_peak_width_points}{Minimum peak width in points. Default 5.}
  \item{noise}{Fixed noise level. \code{NaN} disables.}
  \item{auto_noise}{Estimate noise automatically. Default \code{TRUE}.}
  \item{auto_baseline}{Subtract baseline before detection. Default \code{TRUE}.}
  \item{lambda}{airPLS smoothing parameter for baseline estimation. \code{0} uses a default.}
  \item{max_iterations}{Maximum airPLS iterations. \code{0} uses a default.}
  \item{allow_overlap}{Allow overlapping peak boundaries. Default \code{FALSE}.}
  \item{min_snr}{Minimum signal-to-noise ratio. Default 1.}
  \item{min_r2}{Minimum fit R-squared (Gaussian or EMG) for the artifact filter; \code{0} disables it (keep peaks regardless of shape). Default 0.3.}
  \item{shape}{Peak shape used when fitting.}

}
\value{
A data frame with one row per feature, with values that are medians across the
samples where it was found:
\describe{
  \item{mz}{Median m/z of the feature (Da).}
  \item{rt}{Median retention time of the feature (minutes).}
  \item{from}{Start of the retention-time window (minutes).}
  \item{to}{End of the retention-time window (minutes).}
  \item{intensity}{Median apex intensity.}
  \item{integral}{Median peak area.}
  \item{n_points}{Median number of points across the peak.}
  \item{noise}{Median noise level.}
  \item{frequency}{Fraction of input samples in which this feature was
    detected (\code{n_samples / total_samples}, in \code{[0, 1]}).}
}
}
\examples{
\dontrun{
features <- match_features(
  "/path/to/samples",
  from = 0, to = 10,
  threads = 4L,
  frequency = 2L
)
}
}
\seealso{
\code{\link{find_features}()}, \code{\link{parse_mzml}()}, \code{\link{parse_ion}()}
}
