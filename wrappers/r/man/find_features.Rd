\name{find_features}
\alias{find_features}
\title{Find all features in one file}
\description{
Scans the full m/z grid and finds all chromatographic features in one file.
No target list is needed.
}
\usage{
find_features(
  file,
  from,
  to,
  eic_ppm_tolerance = .DEFAULTS$eic_ppm_tolerance,
  eic_mz_tolerance = .DEFAULTS$eic_mz_tolerance,
  grid_start = .DEFAULTS$grid_start,
  grid_end = .DEFAULTS$grid_end,
  grid_step_size = .DEFAULTS$grid_step_size,
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
  \item{file}{Sample file from \code{\link{parse_mzml}()} or \code{\link{parse_ion}()}.}
  \item{from}{Start retention time (minutes).}
  \item{to}{End retention time (minutes).}
  \item{eic_ppm_tolerance}{PPM tolerance for EIC extraction.}
  \item{eic_mz_tolerance}{Absolute m/z tolerance (Da) for EIC extraction.}
  \item{grid_start}{Lower m/z bound of the search grid (Da). Default 40.}
  \item{grid_end}{Upper m/z bound of the search grid (Da). Default 1000.}
  \item{grid_step_size}{Grid step size (ppm). Default 0.005.}
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
  \item{shape}{Peak shape used when fitting.}

  \item{min_r2}{Minimum fit R-squared (Gaussian or EMG) for the artifact filter; \code{0} disables it (keep peaks regardless of shape). Default 0.3.}
}
\value{
A data frame with one row per detected feature. Columns include \code{mz},
\code{rt}, \code{from}, \code{to}, \code{intensity}, \code{integral},
\code{n_points}.
}
\examples{
\donttest{
url <- "https://raw.githubusercontent.com/phenological/ion-files/main/data/p21_LTR_20_b1_w20.ion"
sample <- parse_ion(url)
features <- find_features(sample, from = 2.3, to = 2.45, grid_start = 255, grid_end = 265)
}
}
\seealso{
\code{\link{match_features}()}
}
