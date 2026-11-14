\name{get_peak_options}
\alias{get_peak_options}
\title{Settings for get_peak}
\description{
Options for get_peak, including the search window and the peak-finding settings.
}
\usage{
get_peak_options(range = 0, find_peaks = find_peaks_options())
}
\arguments{
  \item{range}{Half-width of the search window, in minutes. Only peaks within rt +/- range are considered. 0 searches the whole trace.}
  \item{find_peaks}{Options for find_peaks function. See \code{\link{find_peaks_options}()}.}
}
\value{
An options object for \code{\link{get_peak}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#GetPeakOptions}
}
\examples{
get_peak_options(range = 0.05)
}
\seealso{
\code{\link{get_peak}()}
}
