\name{get_scans}
\alias{get_scans}
\title{Get scans by RT or m/z query}
\description{
Fetches scans from a loaded file. Give exactly one of \code{rt} or \code{mz} per call.
}
\usage{
get_scans(
  file,
  rt = NULL,
  mz = NULL,
  level = 1L
)
}
\arguments{
  \item{file}{Sample file from \code{\link{parse_mzml}()} or \code{\link{parse_ion}()}.}
  \item{rt}{\code{c(from, to)} in minutes for every scan in that range, or one RT in
    minutes for the single closest scan.}
  \item{mz}{\code{c(from, to)} in Da for every scan whose precursor m/z is in that range,
    or one precursor m/z in Da for the single closest scan.}
  \item{level}{MS level to query (integer 0–255). Default 1.}
}
\value{
Range query (\code{rt} or \code{mz} as \code{c(from, to)}): a list of scans,
each with \code{rt}, \code{mz}, \code{intensity}, and metadata.

Point query (\code{rt} or \code{mz}): a single scan list, or \code{NULL} if none found.
}
\examples{
\donttest{
url <- "https://raw.githubusercontent.com/phenological/ion-files/main/data/p21_LTR_20_b1_w20.ion"
sample <- parse_ion(url)
scans <- get_scans(sample, rt = c(2.3, 2.4))
scan <- get_scans(sample, rt = 2.383)
}
}
\seealso{
\code{\link{calculate_eic}()}, \code{\link{parse_mzml}()}, \code{\link{parse_ion}()}
}
