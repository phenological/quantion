\name{parse_ion}
\alias{parse_ion}
\title{Load an ion file}
\description{
Loads an ion file and returns a sample file. The input can be a file path,
which is read lazily from disk, or a raw vector already held in memory.
A string starting with \code{http://} or \code{https://} is read over the network,
fetching only the bytes it needs.
}
\usage{
parse_ion(
  source,
  max_cache_size = 0
)
}
\arguments{
  \item{source}{A file path or an http(s) URL to an ion file, or a raw vector of ion bytes.}
  \item{max_cache_size}{Cache size in bytes. \code{0} sets no limit. Larger values speed up repeated reads at the cost of RAM.}
}
\value{
A Sample file for use in other \code{quantion} functions.
}
\examples{
\donttest{
url <- "https://raw.githubusercontent.com/phenological/ion-files/main/data/p21_LTR_20_b1_w20.ion"
sample <- parse_ion(url)
}
}
\seealso{
\code{\link{parse_mzml}()}, \code{\link{mzml_to_ion}()}, \code{\link{dispose}()}
}
