\name{draw_peaks_options}
\alias{draw_peaks_options}
\title{Settings for draw_peaks}
\description{
Settings for \code{\link{draw_peaks}()}. Every field is checked when the object is built; a misspelt name or a wrong type gives an error.
}
\usage{
draw_peaks_options(from = 0, to = 0, points = 1000L)
}
\arguments{
  \item{from}{Start of the chromatogram. With \code{to}, 0 and 0 fit the range around all the peaks.}
  \item{to}{End of the chromatogram.}
  \item{points}{Number of points drawn.}
}
\value{
An options object for \code{\link{draw_peaks}()}.
}
\examples{
draw_peaks_options(from = 2, to = 3, points = 2000L)
}
\seealso{
\code{\link{draw_peaks}()}
}
