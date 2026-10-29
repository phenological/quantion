\name{find_peaks}
\alias{find_peaks}
\title{Find every peak in a trace}
\description{
Find every peak in one trace.
}
\usage{
find_peaks(x, y, options = find_peaks_options())
}
\arguments{
  \item{x}{Retention times in minutes, ascending. At least 3 points.}
  \item{y}{Intensities, the same length as \code{x}.}
  \item{options}{Settings from \code{\link{find_peaks_options}()}.}
}
\value{
A data.frame with one row per peak: \code{rt}, \code{from}, \code{to}, \code{intensity}, \code{integral}, \code{n_points}, \code{noise}, \code{r2}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#find_peaks}
}
\examples{
trace <- draw_peaks(list(list(height = 1000, center = 2, fwhm = 0.05, tail = 0.01)),
                    draw_peaks_options(from = 1, to = 3, points = 400L))
peaks <- find_peaks(trace$x, trace$y, options = find_peaks_options(min_snr = 3))
}
\seealso{
\code{\link{find_peaks_options}()}, \code{\link{get_peak}()}
}
