\name{get_scans_options}
\alias{get_scans_options}
\title{Settings for get_scans}
\description{
Options for get_scans, including the MS level and which arrays to read.
}
\usage{
get_scans_options(level = 1L, time_unit = "MINUTES",
                  arrays = list(list(key = "mz", accession = "MS:1000514"),
                                list(key = "intensity", accession = "MS:1000515")))
}
\arguments{
  \item{level}{MS level of the scans, 1 for MS1 and 2 for MS2. 0 reads every level.}
  \item{time_unit}{Unit of the rt you give, minutes or seconds.}
  \item{arrays}{Which data arrays to read from each scan, such as m/z and intensity, given as keys with accessions. The other fields of Scan always come back. See \href{https://phenological.github.io/quantion/?language=r#ScanArray}{ScanArray}.}
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
