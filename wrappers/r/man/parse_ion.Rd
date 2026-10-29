\name{parse_ion}
\alias{parse_ion}
\title{Open an .ion file}
\description{
Open an .ion file from a path, an http(s) URL or raw bytes. Paths and URLs are read lazily: only the bytes a query needs are fetched. Errors are conditions of class \code{quantion_error} with fields \code{kind}, \code{code}, \code{function}, \code{argument} and \code{message}.
}
\usage{
parse_ion(source, options = parse_ion_options())
}
\arguments{
  \item{source}{A path, an http(s) URL, or a raw vector of .ion bytes.}
  \item{options}{Settings from \code{\link{parse_ion_options}()}.}
}
\value{
A sample (an external pointer of class \code{quantion_sample}).
}
\examples{
\dontrun{
sample <- parse_ion("sample.ion")
remote <- parse_ion("https://example.org/sample.ion",
                    options = parse_ion_options(max_cache_size = 64e6))
}
}
\seealso{
\code{\link{parse_mzml}()}, \code{\link{write_ion}()}, \code{\link{dispose}()}
}
