\name{fit_peak_options}
\alias{fit_peak_options}
\title{Settings for fit_peak}
\description{
Settings for \code{\link{fit_peak}()}. Every field is checked when the object is built; a misspelt name or a wrong type gives an error.
}
\usage{
fit_peak_options(shape = "EMG", intensity = 0)
}
\arguments{
  \item{shape}{\code{"EMG"} or \code{"GAUSSIAN"}: the curve to fit.}
  \item{intensity}{Apex intensity to start the fit from. 0 takes it from the peak at rt.}
}
\value{
An options object for \code{\link{fit_peak}()}.
}
\examples{
fit_peak_options(shape = "GAUSSIAN")
}
\seealso{
\code{\link{fit_peak}()}
}
