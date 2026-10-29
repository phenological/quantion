\name{find_targets_options}
\alias{find_targets_options}
\title{Settings for find_targets}
\description{
Settings for \code{\link{find_targets}()}. Every field is checked when the object is built; a misspelt name or a wrong type gives an error.
}
\usage{
find_targets_options(eic = calculate_eic_options(), find_peaks = find_peaks_options(), threads = 1L)
}
\arguments{
  \item{eic}{Retention-time range, tolerances and time unit from \code{\link{calculate_eic_options}()}.}
  \item{find_peaks}{Peak-picking settings from \code{\link{find_peaks_options}()}.}
  \item{threads}{Number of threads to use.}
}
\value{
An options object for \code{\link{find_targets}()}.
}
\examples{
find_targets_options(eic = calculate_eic_options(from = 0.5, to = 5), threads = 4L)
}
\seealso{
\code{\link{find_targets}()}
}
