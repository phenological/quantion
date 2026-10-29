\name{write_mzml_options}
\alias{write_mzml_options}
\title{Settings for write_mzml}
\description{
Settings for \code{\link{write_mzml}()}. Every field is checked when the object is built; a misspelt name or a wrong type gives an error.
}
\usage{
write_mzml_options(out = NULL, overwrite = FALSE)
}
\arguments{
  \item{out}{A path. \code{NULL} returns the bytes.}
  \item{overwrite}{Replace a file that already exists.}
}
\value{
An options object for \code{\link{write_mzml}()}.
}
\examples{
write_mzml_options(out = "sample.mzML")
}
\seealso{
\code{\link{write_mzml}()}
}
