\name{get_peak}
\alias{get_peak}
\title{Find the one peak near a retention time}
\description{
Find the one peak near \code{rt}. With a range, only \code{rt} plus or minus the range is searched; without one, the whole trace is searched and the peak is chosen by its size and its distance to \code{rt}.
}
\usage{
get_peak(x, y, rt, options = get_peak_options())
}
\arguments{
  \item{x}{Retention times in minutes, ascending. At least 3 points.}
  \item{y}{Intensities, the same length as \code{x}.}
  \item{rt}{Expected retention time of the peak.}
  \item{options}{Settings from \code{\link{get_peak_options}()}: \code{range = 0}, \code{find_peaks}.
    \href{https://phenological.github.io/quantion/?language=r#GetPeakOptions}{GetPeakOptions documentation}
  }
}
\value{
A list with \code{rt}, \code{from}, \code{to}, \code{intensity}, \code{integral}, \code{n_points}, \code{noise}, \code{r2}. Every field is 0 when no peak is found.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#get_peak}
}
\examples{
trace <- draw_peaks(list(list(height = 1000, center = 2, fwhm = 0.05, tail = 0.01)),
                    draw_peaks_options(from = 1, to = 3, points = 400L))
peak <- get_peak(trace$x, trace$y, rt = 2, options = get_peak_options(range = 0.05))
anywhere <- get_peak(trace$x, trace$y, rt = 2)
}
\seealso{
\code{\link{get_peak_options}()}, \code{\link{find_peaks}()}
}
