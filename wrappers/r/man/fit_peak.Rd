\name{fit_peak}
\alias{fit_peak}
\title{Fit a curve to a peak}
\description{
Fit a Gaussian or EMG curve to the peak at \code{rt}. The apex intensity of that peak is the starting point unless one is forced.
}
\usage{
fit_peak(x, y, rt, options = fit_peak_options())
}
\arguments{
  \item{x}{Retention times in minutes, ascending. At least 3 points.}
  \item{y}{Intensities, the same length as \code{x}.}
  \item{rt}{Retention time of the peak to fit.}
  \item{options}{Settings from \code{\link{fit_peak_options}()}: \code{shape = "EMG"}, \code{intensity = 0}.
    \href{https://phenological.github.io/quantion/?language=r#FitPeakOptions}{FitPeakOptions documentation}
  }
}
\value{
A list with \code{height}, \code{center}, \code{fwhm}, \code{tail} (0 for a Gaussian) and \code{r2}, or \code{NULL} when no peak is found or the fit fails.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#fit_peak}
}
\examples{
trace <- draw_peaks(list(list(height = 1000, center = 2, fwhm = 0.05, tail = 0.01)),
                    draw_peaks_options(from = 1, to = 3, points = 400L))
params <- fit_peak(trace$x, trace$y, rt = 2)
gaussian <- fit_peak(trace$x, trace$y, rt = 2, options = fit_peak_options(shape = "GAUSSIAN"))
}
\seealso{
\code{\link{fit_peak_options}()}, \code{\link{draw_peaks}()}
}
