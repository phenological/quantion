\name{write_ion_options}
\alias{write_ion_options}
\title{Settings for write_ion}
\description{
Settings for \code{\link{write_ion}()}. Every field is checked when the object is built; a misspelt name or a wrong type gives an error.
}
\usage{
write_ion_options(out = NULL, overwrite = FALSE, level = 12L, f32_compress = FALSE, headers = list())
}
\arguments{
  \item{out}{A path or an http(s) URL. \code{NULL} returns the bytes.}
  \item{overwrite}{Replace a file that already exists.}
  \item{level}{Compression level, 0 (none) to 22 (smallest).}
  \item{f32_compress}{Store intensities as 32-bit floats.}
  \item{headers}{A list of \code{\link{header}()}; only allowed with a URL.}
}
\value{
An options object for \code{\link{write_ion}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#WriteIonOptions}
}
\examples{
write_ion_options(out = "sample.ion", level = 12L)
}
\seealso{
\code{\link{write_ion}()}
}
