\name{get_scans}
\alias{get_scans}
\title{Read scans}
\description{
Read scans by retention time or precursor m/z. A range returns every scan in it; one number returns the closest scan, as a list of one.
}
\usage{
get_scans(file, rt = NULL, mz = NULL, options = get_scans_options())
}
\arguments{
  \item{file}{The sample to read, from parse_ion or parse_mzml. Type: \code{SampleFile}.}
  \item{rt}{Which scans, by retention time: a range or one value. Type: \code{numeric(1) | c(from, to)}.}
  \item{mz}{Or by precursor m/z: a range or one value. Give rt or mz, not both. Type: \code{numeric(1) | c(from, to)}.}
  \item{options}{Settings from \code{\link{get_scans_options}()}: \code{level = 1L}, \code{time_unit = "MINUTES"}.
    \href{https://phenological.github.io/quantion/?language=r#GetScansOptions}{GetScansOptions documentation}
  }
}
\value{
The scans found, in rt order. Type: \code{data.frame of Scan}. Columns: \code{rt}, \code{mz}, \code{intensity}, \code{level}, \code{precursor_mz}, \code{base_peak_mz}, \code{base_peak_intensity}, \code{total_ion_current}, \code{polarity}, \code{x}, \code{y}, \code{z}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#get_scans}
}
\examples{
\donttest{
if (curl::has_internet()) {
  library(quantion)

  url <- "https://raw.githubusercontent.com/phenological/ion-files/main/data/p21_LTR_20_b1_w20.ion"
  sample <- parse_ion(url)
  scans <- get_scans(sample, rt = c(2.3, 2.4))
  closest <- get_scans(sample, rt = 143, options = get_scans_options(time_unit = "SECONDS"))
  print(nrow(scans))
  print(closest$rt)
}
}
}
\seealso{
\code{\link{get_scans_options}()}
}
