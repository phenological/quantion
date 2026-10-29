\name{find_noise_level}
\alias{find_noise_level}
\title{Estimate the noise level}
\description{
Estimate the noise level of a signal.
}
\usage{
find_noise_level(y)
}
\arguments{
  \item{y}{Intensities.}
}
\value{
A list with \code{width} (the window width the estimate came from) and \code{intensity}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#find_noise_level}
}
\examples{
trace <- draw_peaks(list(list(height = 1000, center = 2, fwhm = 0.05, tail = 0.01)),
                    draw_peaks_options(from = 1, to = 3, points = 400L))
noise <- find_noise_level(trace$y)$intensity
}
\seealso{
\code{\link{find_peaks}()}
}
