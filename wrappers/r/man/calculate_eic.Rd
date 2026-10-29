\name{calculate_eic}
\alias{calculate_eic}
\title{Get an extracted ion chromatogram (EIC)}
\description{
Extracts an EIC for one target m/z over a given retention-time range.
}
\usage{
calculate_eic(
  file,
  mz,
  from,
  to,
  ppm_tolerance = 20,
  mz_tolerance = 0.005
)
}
\arguments{
  \item{file}{Sample file from \code{\link{parse_mzml}()} or \code{\link{parse_ion}()}.}
  \item{mz}{The m/z to extract (Da).}
  \item{from}{Start retention time (minutes).}
  \item{to}{End retention time (minutes).}
  \item{ppm_tolerance}{Mass tolerance in ppm.}
  \item{mz_tolerance}{Absolute mass tolerance in Da.}
}
\value{
A list with \code{x} (retention time, numeric) and \code{y} (intensity, numeric).
}
\examples{
\donttest{
url <- "https://raw.githubusercontent.com/phenological/ion-files/main/data/p21_LTR_20_b1_w20.ion"
sample <- parse_ion(url)
eic <- calculate_eic(sample, 260.103, from = 0.5, to = 5)
}
}
\seealso{
\code{\link{get_scans}()}, \code{\link{find_peaks}()}, \code{\link{get_peak}()}
}
