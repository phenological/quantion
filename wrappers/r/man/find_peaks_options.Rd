\name{find_peaks_options}
\alias{find_peaks_options}
\title{Peak-picking settings}
\description{
Options for find_peaks. Every field is optional.
}
\usage{
find_peaks_options(min_integral = NA, min_intensity = 500, min_peak_width_points = 3L,
                   noise = NA, auto_noise = TRUE, auto_baseline = TRUE, lambda = 20,
                   max_iterations = 4L, allow_overlap = FALSE, min_snr = 2, min_r2 = 0,
                   shape = "EMG", kernel_size = 0L)
}
\arguments{
  \item{min_integral}{Drop peaks whose area (integral) is below this value. Off when not given.}
  \item{min_intensity}{Drop peaks whose apex intensity is below this value. Raise it to ignore small peaks.}
  \item{min_peak_width_points}{Drop peaks with fewer data points than this.}
  \item{noise}{Fixed noise level (intensity), used only when auto_noise is false.}
  \item{auto_noise}{Estimate the noise from the chromatogram. Turn off and set noise to use a fixed value.}
  \item{auto_baseline}{Subtract an airPLS baseline before finding peaks. Turn off for windows shorter than 1 minute: they do not have enough points to estimate the baseline and can give misleading results.}
  \item{lambda}{Smoothness of the airPLS baseline. A larger value gives a smoother baseline that follows only slow drift.}
  \item{max_iterations}{Maximum number of airPLS iterations used to estimate the baseline.}
  \item{allow_overlap}{Let neighbouring peaks share points at their border. When off, overlapping peaks are split at the valley between them.}
  \item{min_snr}{Minimum signal-to-noise ratio. A peak's apex must be above min_snr x noise.}
  \item{min_r2}{Minimum fit quality (0-1) of the peak shape. Peaks that fit the shape worse are dropped. 0 keeps every peak.}
  \item{shape}{Curve used to check the peak shape for min_r2: EMG or GAUSSIAN.}
  \item{kernel_size}{Savitzky-Golay smoothing window, in points, to smooth noisy chromatograms. 0 = no smoothing. A larger value gives a smoother chromatogram.}
}
\value{
An options object for \code{\link{find_peaks}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#FindPeaksOptions}
}
\examples{
find_peaks_options(min_intensity = 1000, min_snr = 3)
}
\seealso{
\code{\link{find_peaks}()}
}
