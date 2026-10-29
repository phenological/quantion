\name{parse_mzml}
\alias{parse_mzml}
\title{Open an mzML file}
\description{
Open an mzML file from a path or raw bytes. It is always loaded fully into memory. URLs are refused.
}
\usage{
parse_mzml(source)
}
\arguments{
  \item{source}{A path or a raw vector of mzML bytes.}
}
\value{
A sample (an external pointer of class \code{quantion_sample}).
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#parse_mzml}
}
\examples{
\dontrun{
sample <- parse_mzml("sample.mzML")
}
}
\seealso{
\code{\link{parse_ion}()}, \code{\link{write_mzml}()}
}
