\name{get_scans_options}
\alias{get_scans_options}
\title{Settings for get_scans}
\description{
Settings for get_scans.
}
\usage{
get_scans_options(level = 1L, time_unit = "MINUTES")
}
\arguments{
  \item{level}{MS level. 0 = every level.}
  \item{time_unit}{Unit of rt in the query.}
  \item{arrays}{Which arrays to read from each scan, as keys with accessions. Every other field of Scan always comes back. See \code{\link{scan_array}()}.}
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
