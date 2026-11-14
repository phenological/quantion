\name{find_targets_options}
\alias{find_targets_options}
\title{Settings for find_targets}
\description{
Options for find_targets, including the EIC window, the peak-finding settings and the threads.
}
\usage{
find_targets_options(eic = calculate_eic_options(), find_peaks = find_peaks_options(),
                     threads = 1L)
}
\arguments{
  \item{eic}{Options for the EICs, with the retention-time range (from, to), the m/z tolerances and the time unit. See \code{\link{calculate_eic_options}()}.}
  \item{find_peaks}{Options for find_peaks, used for every target. See \code{\link{find_peaks_options}()}.}
  \item{threads}{Number of threads (CPU cores) used to process the targets in parallel.}
}
\value{
An options object for \code{\link{find_targets}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#FindTargetsOptions}
}
\examples{
find_targets_options(eic = calculate_eic_options(from = 0.5, to = 5), threads = 4L)
}
\seealso{
\code{\link{find_targets}()}
}
