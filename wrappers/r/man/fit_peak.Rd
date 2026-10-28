\name{fit_peak}
\alias{fit_peak}
\title{Fit a peak shape to XY data}
\description{
Fits a Gaussian or an exponentially modified Gaussian to XY data around one
retention time, and returns the shape parameters of the best fit.
}
\usage{
fit_peak(
  x,
  y,
  rt,
  intensity,
  shape = "emg"
)
}
\arguments{
  \item{x}{A numeric vector of retention times. At least 5 points.}
  \item{y}{A numeric vector of intensities, the same length as \code{x}.}
  \item{rt}{The retention time to start the fit from.}
  \item{intensity}{The intensity at \code{rt}, used as the starting height.}
  \item{shape}{\code{"gaussian"} or \code{"emg"}.}
}
\value{
A list with \code{shape}, \code{height}, \code{center}, \code{fwhm}, \code{tail}
and \code{r2}. \code{tail} is \code{0} for a Gaussian.
}
\examples{
\donttest{
url <- "https://raw.githubusercontent.com/phenological/ion-files/main/data/p21_LTR_20_b1_w20.ion"
sample <- parse_ion(url)
eic <- calculate_eic(sample, 260.103, from = 0.5, to = 5)
peak <- get_peak(eic$x, eic$y, rt = 2.383, range = 0.05)
params <- fit_peak(eic$x, eic$y, peak$rt, peak$intensity)
}
}
\seealso{
\code{\link{draw_peak}()}, \code{\link{get_peak}()}, \code{\link{find_peaks}()}
}
