\name{find_transitions_options}
\alias{find_transitions_options}
\title{Settings for find_transitions}
\description{
Settings for \code{\link{find_transitions}()}. Every field is checked when the object is built; a misspelt name or a wrong type gives an error.
}
\usage{
find_transitions_options(find_peaks = find_peaks_options(), threads = 1L)
}
\arguments{
  \item{find_peaks}{Peak-picking settings from \code{\link{find_peaks_options}()}.}
  \item{threads}{Number of threads to use.}
}
\value{
An options object for \code{\link{find_transitions}()}.
}
\examples{
find_transitions_options(threads = 2L)
}
\seealso{
\code{\link{find_transitions}()}
}
