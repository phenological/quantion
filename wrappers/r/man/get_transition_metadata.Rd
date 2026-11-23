\name{get_transition_metadata}
\alias{get_transition_metadata}
\title{Read the params of one stored chromatogram}
\description{
Read the details stored for one chromatogram, such as an SRM or MRM transition (attributes, precursor, product, cvParams and userParams), one row each, parents before children. Find it by its index or by its precursor (Q1) and product (Q3), both within 0.005 m/z.
}
\usage{
get_transition_metadata(file, query, options = get_transition_metadata_options())
}
\arguments{
  \item{file}{The sample to read, from parse_ion or parse_mzml. Type: \code{SampleFile}.}
  \item{query}{Which chromatogram: an index counted from 0, or a precursor (Q1) and product (Q3) pair. Type: \code{numeric(1) | list(precursor, product)}.}
  \item{options}{Settings from \code{\link{get_transition_metadata_options}()}: \code{tags = character(0)}.
    \href{https://phenological.github.io/quantion/?language=r#GetTransitionMetadataOptions}{GetTransitionMetadataOptions documentation}
  }
}
\value{
One row per attribute or param, in the order of the mzML tree, with paths from chromatogram down. Empty when no stored chromatogram matches. A field the file does not give is missing (None in Rust and Python, null in JavaScript, NA in R). Type: \code{data.frame of ScanParam}. Columns: \code{path}, \code{kind}, \code{accession}, \code{name}, \code{value}, \code{unit}, \code{unit_accession}.
}
\section{Warning}{
Needs the whole file. A sample opened from a URL gives a remote_file error; open it from a path or bytes.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#get_transition_metadata}
}
\examples{
\donttest{
if (curl::has_internet()) {
  library(quantion)

  url <- "https://raw.githubusercontent.com/phenological/quantion/main/core/tests/fixtures/srm.ion"
  sample <- parse_ion(curl::curl_fetch_memory(url)$content)
  rows <- get_transition_metadata(sample, 0)
  windows <- get_transition_metadata(sample, list(precursor = 89.04768, product = 89.04768),
                                     options = get_transition_metadata_options(tags = "isolationWindow"))
  print(rows)
  print(windows)
}
}
}
\seealso{
\code{\link{get_transition_metadata_options}()}, \code{\link{get_transition}()}, \code{\link{get_scan_metadata}()}
}
