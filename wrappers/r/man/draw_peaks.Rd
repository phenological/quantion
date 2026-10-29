\name{draw_peaks}
\alias{draw_peaks}
\title{Simulate a chromatogram}
\description{
Simulate a chromatogram from one or more peak curves. Every peak is drawn into the same chromatogram; where they overlap, intensities add up. A \code{tail} of 0 draws a Gaussian, above 0 an EMG.
}
\usage{
draw_peaks(params, options = draw_peaks_options())
}
\arguments{
  \item{params}{A list of peaks, each a list with \code{height}, \code{center}, \code{fwhm} and an optional \code{tail}, as returned by \code{\link{fit_peak}()}.}
  \item{options}{Settings from \code{\link{draw_peaks_options}()}.}
}
\value{
A list with \code{x} and \code{y}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#draw_peaks}
}
\examples{
peak_1 <- list(height = 9800, center = 2.38, fwhm = 0.021, tail = 0.006)
peak_2 <- list(height = 4200, center = 2.45, fwhm = 0.018, tail = 0)
chrom <- draw_peaks(list(peak_1, peak_2),
                    options = draw_peaks_options(from = 2, to = 3, points = 2000L))
fitted_range <- draw_peaks(list(peak_1, peak_2))
}
\seealso{
\code{\link{draw_peaks_options}()}, \code{\link{fit_peak}()}
}
