\name{write_ion}
\alias{write_ion}
\title{Write a sample as .ion}
\description{
Write a sample as .ion. With no \code{out}, the bytes are returned. A path writes a file; a URL uploads it in one PUT request. A sample opened from a URL is refused, because writing needs the whole file.
}
\usage{
write_ion(file, options = write_ion_options())
}
\arguments{
  \item{file}{A sample from \code{\link{parse_ion}()} or \code{\link{parse_mzml}()}.}
  \item{options}{Settings from \code{\link{write_ion_options}()}.}
}
\value{
A raw vector when \code{out} is not given, otherwise \code{NULL} (invisible).
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#write_ion}
}
\examples{
\dontrun{
sample <- parse_ion("sample.ion")
data <- write_ion(sample)
write_ion(sample, options = write_ion_options(out = "sample.ion"))
write_ion(sample, options = write_ion_options(
  out = "https://example.org/upload/sample.ion",
  headers = list(header("Authorization", "Bearer <token>"))
))
}
}
\seealso{
\code{\link{write_ion_options}()}, \code{\link{header}()}, \code{\link{write_mzml}()}
}
