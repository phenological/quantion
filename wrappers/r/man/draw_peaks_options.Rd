\name{draw_peaks_options}
\alias{draw_peaks_options}
\title{Settings for draw_peaks}
\description{
Options for draw_peaks, including the time range and the number of points.
}
\usage{
draw_peaks_options(from = 0, to = 0, points = 1000L)
}
\arguments{
  \item{from}{Start of the simulated chromatogram, in minutes. Leave from and to at 0 to cover all the peaks automatically.}
  \item{to}{End of the simulated chromatogram, in minutes.}
  \item{points}{Number of points drawn from from to to.}
}
\value{
An options object for \code{\link{draw_peaks}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#DrawPeaksOptions}
}
\examples{
draw_peaks_options(from = 2, to = 3, points = 2000L)
}
\seealso{
\code{\link{draw_peaks}()}
}
