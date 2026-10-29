\name{find_transitions}
\alias{find_transitions}
\title{Find peaks from stored chromatograms}
\description{
Finds a peak in the stored chromatogram of each target. A target picks the
chromatogram whose precursor (Q1) and product (Q3) m/z are both within 0.005.
}
\usage{
find_transitions(
  file,
  targets,
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
  \item{targets}{Data frame with columns \code{id}, \code{precursor} (Q1 m/z),
    \code{product} (Q3 m/z), \code{rt} and \code{range} (RT half-window).}
  \item{threads}{Number of threads to use. Default 1.}
  \item{min_integral}{Minimum peak area. \code{NaN} disables.}
  \item{min_intensity}{Minimum apex intensity. Default 1000.}
  \item{min_peak_width_points}{Minimum peak width in points. Default 5.}
  \item{noise}{Fixed noise level. \code{NaN} disables.}
  \item{auto_noise}{Estimate noise automatically. Default \code{FALSE}.}
  \item{auto_baseline}{Subtract baseline before detection. Default \code{FALSE}.}
  \item{lambda}{airPLS smoothing parameter for baseline estimation. \code{0} uses a default.}
  \item{max_iterations}{Maximum airPLS iterations. \code{0} uses a default.}
  \item{allow_overlap}{Allow overlapping peak boundaries. Default \code{FALSE}.}
  \item{min_snr}{Minimum signal-to-noise ratio. Default 1.}
  \item{min_r2}{Minimum fit R-squared (Gaussian or EMG) for the artifact filter; \code{0} disables it (keep peaks regardless of shape). Default 0.3.}
  \item{shape}{Peak shape used when fitting.}

}
\value{
A data frame with one row per target, in the same order, with the peak columns
\code{rt}, \code{from}, \code{to}, \code{intensity}, \code{integral},
\code{n_points}, \code{noise} and \code{r2}. A target with no matching chromatogram gets zeros.
}
\examples{
\dontrun{
targets <- data.frame(
  id        = c("compound_1", "compound_2"),
  precursor = c(500.3, 520.4),
  product   = c(184.1, 184.1),
  rt        = c(3.20, 3.65),
  range     = c(0.4, 0.4)
)
find_transitions(sample, targets, threads = 2L)
}
}
\seealso{
\code{\link{find_targets}()}, \code{\link{get_peak}()}
}
