\name{get_scans_options}
\alias{get_scans_options}
\title{Settings for get_scans}
\description{
Settings for \code{\link{get_scans}()}. Every field is checked when the object is built; a misspelt name or a wrong type gives an error.
}
\usage{
get_scans_options(level = 1L, time_unit = "MINUTES")
}
\arguments{
  \item{level}{MS level: 1 = MS1, 2 = MS2, 0 = every level.}
  \item{time_unit}{\code{"MINUTES"} or \code{"SECONDS"}: the unit of \code{rt}.}
}
\value{
An options object for \code{\link{get_scans}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#GetScansOptions}
}
\examples{
get_scans_options(level = 2L, time_unit = "SECONDS")
}
\seealso{
\code{\link{get_scans}()}
}
