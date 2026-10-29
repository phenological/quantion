\name{write_mzml}
\alias{write_mzml}
\title{Write a sample as mzML}
\description{
Write a sample as mzML. With no \code{out}, the bytes are returned. Only a path is allowed, no URL.
}
\usage{
write_mzml(file, options = write_mzml_options())
}
\arguments{
  \item{file}{A sample from \code{\link{parse_ion}()} or \code{\link{parse_mzml}()}.}
  \item{options}{Settings from \code{\link{write_mzml_options}()}.}
}
\value{
A raw vector when \code{out} is not given, otherwise \code{NULL} (invisible).
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#write_mzml}
}
\examples{
\dontrun{
sample <- parse_ion("sample.ion")
write_mzml(sample, options = write_mzml_options(out = "sample.mzML"))
}
}
\seealso{
\code{\link{write_mzml_options}()}, \code{\link{write_ion}()}
}
