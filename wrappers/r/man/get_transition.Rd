\name{get_transition}
\alias{get_transition}
\title{Read one stored chromatogram}
\description{
Read one stored chromatogram, such as the TIC or an SRM or MRM transition, by its index or by its precursor (Q1) and product (Q3), both within 0.005 m/z. Use it to draw a trace before get_peak or find_peaks.
}
\usage{
get_transition(file, query)
}
\arguments{
  \item{file}{The sample to read, from parse_ion or parse_mzml. Type: \code{SampleFile}.}
  \item{query}{Which chromatogram: an index counted from 0, or a precursor (Q1) and product (Q3) pair. Type: \code{numeric(1) | list(precursor, product)}.}
}
\value{
The chromatogram, with x as the retention times in minutes and y as the intensities. Both are empty when no stored chromatogram matches. Type: \code{DataXY list}. Fields: \code{x}, \code{y}.
}
\section{Warning}{
A sample opened from a URL gives a remote_file error; open it from a path or bytes.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#get_transition}
}
\examples{
\donttest{
if (curl::has_internet()) {
  library(quantion)

  url <- "https://raw.githubusercontent.com/phenological/quantion/main/core/tests/fixtures/srm.ion"
  sample <- parse_ion(curl::curl_fetch_memory(url)$content)
  first <- get_transition(sample, 0)
  alanine <- get_transition(sample, list(precursor = 89.04768, product = 89.04768))
  print(c(length(first$x), length(alanine$y)))
}
}
}
\seealso{
\code{\link{get_transition_metadata}()}, \code{\link{find_transitions}()}
}
