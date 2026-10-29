\name{get_scans}
\alias{get_scans}
\title{Read scans}
\description{
Read scans by retention time or precursor m/z. A range \code{c(from, to)} returns every scan in it; one number returns the closest scan.
}
\usage{
get_scans(file, rt = NULL, mz = NULL, options = get_scans_options())
}
\arguments{
  \item{file}{A sample from \code{\link{parse_ion}()} or \code{\link{parse_mzml}()}.}
  \item{rt}{One retention time, or \code{c(from, to)}.}
  \item{mz}{One precursor m/z, or \code{c(from, to)}. Give \code{rt} or \code{mz}, not both.}
  \item{options}{Settings from \code{\link{get_scans_options}()}.}
}
\value{
A data.frame with one row per scan: \code{rt}, \code{mz} and \code{intensity} (list columns), \code{level}, \code{precursor_mz}, \code{base_peak_mz}, \code{base_peak_intensity}, \code{total_ion_current}, \code{polarity}, \code{x}, \code{y}, \code{z}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#get_scans}
}
\examples{
\dontrun{
sample <- parse_ion("sample.ion")
scans <- get_scans(sample, rt = c(1.0, 2.5))
closest <- get_scans(sample, rt = 108,
                     options = get_scans_options(level = 2L, time_unit = "SECONDS"))
}
}
\seealso{
\code{\link{get_scans_options}()}
}
