\name{find_transitions_options}
\alias{find_transitions_options}
\title{Settings for find_transitions}
\description{
Options for find_transitions, including the peak-finding settings and the threads.
}
\usage{
find_transitions_options(find_peaks = find_peaks_options(), threads = 1L)
}
\arguments{
  \item{find_peaks}{Options for find_peaks, used for every transition. See \code{\link{find_peaks_options}()}.}
  \item{threads}{Number of threads (CPU cores) used to process the transitions in parallel.}
}
\value{
An options object for \code{\link{find_transitions}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#FindTransitionsOptions}
}
\examples{
find_transitions_options(threads = 2L)
}
\seealso{
\code{\link{find_transitions}()}
}
