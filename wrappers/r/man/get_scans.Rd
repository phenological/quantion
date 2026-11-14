\name{get_scans}
\alias{get_scans}
\title{Read scans}
\description{
Read scans (spectra) by retention time, by precursor m/z, or by both. A range returns every scan inside it. One value returns the closest scan, as a list of one.
}
\usage{
get_scans(file, rt = NULL, mz = NULL, options = get_scans_options())
}
\arguments{
  \item{file}{The sample to read, from parse_ion or parse_mzml. Type: \code{SampleFile}.}
  \item{rt}{Retention time of the scans, as a range \{from, to\} or as one value for the closest scan. Optional when mz is given. Type: \code{numeric(1) | c(from, to)}.}
  \item{mz}{Precursor m/z of the scans, as a range \{from, to\} or as one value for the closest scan. Give rt, mz or both. Only MS2 scans have a precursor m/z, so set level to 2 (or 0 for every level) when you give mz. Type: \code{numeric(1) | c(from, to)}.}
  \item{options}{Settings from \code{\link{get_scans_options}()}: \code{level = 1L}, \code{time_unit = "MINUTES"}, \code{arrays = list(key = "mz", accession = "MS:1000514"), list(key = "intensity", accession = "MS:1000515")}.
    \href{https://phenological.github.io/quantion/?language=r#GetScansOptions}{GetScansOptions documentation}
  }
}
\value{
The scans found, in rt order. An empty list means no scan matched, so check rt, mz and level. Type: \code{data.frame of Scan}. Columns: \code{index}, \code{rt}, \code{mz}, \code{intensity}, \code{level}, \code{precursor_mz}, \code{base_peak_mz}, \code{base_peak_intensity}, \code{total_ion_current}, \code{polarity}, \code{x}, \code{y}, \code{z}.
}
\section{Warning}{
It is best to write the argument names rt and mz to avoid confusion. Without names, rt comes before mz, so a number given without a name is taken as rt.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#get_scans}
}
\examples{
\donttest{
if (curl::has_internet()) {
  # example 1
  library(quantion)

  url <- "https://raw.githubusercontent.com/phenological/ion-files/main/data/aa_p21_LTR_20_b1_w20.ion"
  sample <- parse_ion(url)
  scans <- get_scans(sample, rt = c(2.3, 2.4))
  closest <- get_scans(sample, rt = 143, options = get_scans_options(time_unit = "SECONDS"))
  print(nrow(scans))
  print(closest$mz)

  # example 2
  arrays <- list(list(key = "intensity", accession = "MS:1000515"))
  options <- get_scans_options(level = 2L, arrays = arrays)
  found <- get_scans(sample, mz = c(515.0, 515.01), options = options)
  print(found$intensity)

  # example 3
  nearest <- get_scans(sample, rt = c(2.3, 2.5), mz = 515.0, options = get_scans_options(level = 2L))
  print(nearest$precursor_mz)
}
}
}
\seealso{
\code{\link{get_scans_options}()}
}
