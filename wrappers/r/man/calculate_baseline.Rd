\name{calculate_baseline}
\alias{calculate_baseline}
\title{Estimate a baseline}
\description{
Estimate a signal's baseline with airPLS.
}
\usage{
calculate_baseline(y, options = calculate_baseline_options())
}
\arguments{
  \item{y}{Intensities.}
  \item{options}{Settings from \code{\link{calculate_baseline_options}()}.}
}
\value{
A numeric vector, the same length as \code{y}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#calculate_baseline}
}
\examples{
trace <- draw_peaks(list(list(height = 1000, center = 2, fwhm = 0.05, tail = 0.01)),
                    draw_peaks_options(from = 1, to = 3, points = 400L))
baseline <- calculate_baseline(trace$y, options = calculate_baseline_options(lambda = 50))
}
\seealso{
\code{\link{calculate_baseline_options}()}
}
