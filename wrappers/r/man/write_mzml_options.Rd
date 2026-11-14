\name{write_mzml_options}
\alias{write_mzml_options}
\title{Settings for write_mzml}
\description{
Options for write_mzml.
}
\usage{
write_mzml_options(out = NULL, overwrite = FALSE)
}
\arguments{
  \item{out}{Path to write to. If it is not given, the bytes are returned.}
  \item{overwrite}{Replace the file if it already exists.}
}
\value{
An options object for \code{\link{write_mzml}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#WriteMzmlOptions}
}
\examples{
write_mzml_options(out = "sample.mzML")
}
\seealso{
\code{\link{write_mzml}()}
}
