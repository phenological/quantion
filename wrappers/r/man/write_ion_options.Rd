\name{write_ion_options}
\alias{write_ion_options}
\title{Settings for write_ion}
\description{
Settings for write_ion.
}
\usage{
write_ion_options(out = NULL, overwrite = FALSE, level = 12L, f32_compress = FALSE, headers = list())
}
\arguments{
  \item{out}{Path or URL to write to. Not given = return the bytes.}
  \item{overwrite}{Replace an existing file.}
  \item{level}{Compression level, 0 to 22.}
  \item{f32_compress}{Store intensities as 32-bit floats.}
  \item{headers}{Extra HTTP headers. Only with a URL.}
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
