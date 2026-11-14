\name{find_features_options}
\alias{find_features_options}
\title{Settings for find_features}
\description{
Options for find_features, including the EIC window, the m/z grid, the peak-finding settings and the threads.
}
\usage{
find_features_options(eic = calculate_eic_options(), grid = grid_options(),
                      find_peaks = find_peaks_options(), threads = 1L)
}
\arguments{
  \item{eic}{Options for the EICs, with the retention-time range (from, to), the m/z tolerances and the time unit. See \code{\link{calculate_eic_options}()}.}
  \item{grid}{The m/z grid to scan. See \code{\link{grid_options}()}.}
  \item{find_peaks}{Options for find_peaks, used for every grid m/z. See \code{\link{find_peaks_options}()}.}
  \item{threads}{Number of threads (CPU cores) used to scan the grid in parallel.}
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
