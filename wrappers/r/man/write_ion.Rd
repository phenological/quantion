\name{write_ion}
\alias{write_ion}
\title{Write a sample as an Ionic file (.ion)}
\description{
Save a sample as an Ionic file (.ion), for example to convert mzML to .ion. With out, it writes a file to a path or uploads it to a URL with one PUT request. Without out, it returns the bytes.
}
\usage{
write_ion(file, options = write_ion_options())
}
\arguments{
  \item{file}{The sample to save, from parse_ion or parse_mzml. Type: \code{SampleFile}.}
  \item{options}{Settings from \code{\link{write_ion_options}()}: \code{out = NULL}, \code{overwrite = FALSE}, \code{level = 12L}, \code{f32_compress = FALSE}, \code{block_size = 8L}, \code{mz_window = 20L}, \code{headers}.
    \href{https://phenological.github.io/quantion/?language=r#WriteIonOptions}{WriteIonOptions documentation}
  }
}
\value{
The file's bytes when out is not given, or nothing when the file is written or uploaded. Type: \code{raw | NULL (invisible)}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#write_ion}
}
\examples{
\donttest{
if (curl::has_internet()) {
  library(quantion)

  fixtures <- "https://raw.githubusercontent.com/phenological/quantion/main/core/tests/fixtures"
  url <- paste0(fixtures, "/api/api.ion")
  data <- curl::curl_fetch_memory(url)$content
  sample <- parse_ion(data)

  bytes <- write_ion(sample)

  write_ion(sample, options = write_ion_options(out = tempfile(fileext = ".ion")))
  print(length(bytes))
}
}
}
\seealso{
\code{\link{write_ion_options}()}, \code{\link{header}()}, \code{\link{write_mzml}()}
}
