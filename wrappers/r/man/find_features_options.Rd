\name{find_features_options}
\alias{find_features_options}
\title{Settings for find_features}
\description{
Settings for \code{\link{find_features}()}. Every field is checked when the object is built; a misspelt name or a wrong type gives an error.
}
\usage{
find_features_options(eic = calculate_eic_options(), grid = grid_options(),
                      find_peaks = find_peaks_options(), threads = 1L)
}
\arguments{
  \item{eic}{Retention-time range, tolerances and time unit from \code{\link{calculate_eic_options}()}.}
  \item{grid}{The m/z grid from \code{\link{grid_options}()}.}
  \item{find_peaks}{Peak-picking settings from \code{\link{find_peaks_options}()}.}
  \item{threads}{Number of threads to use.}
}
\value{
An options object for \code{\link{find_features}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#FindFeaturesOptions}
}
\examples{
find_features_options(grid = grid_options(start = 100, end = 600), threads = 4L)
}
\seealso{
\code{\link{find_features}()}
}
