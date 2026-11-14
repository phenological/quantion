\name{write_ion_options}
\alias{write_ion_options}
\title{Settings for write_ion}
\description{
Options for write_ion, including where to write and how to compress.
}
\usage{
write_ion_options(out = NULL, overwrite = FALSE, level = 12L, f32_compress = FALSE,
                  headers = list(), block_size = 8L, mz_window = 20L)
}
\arguments{
  \item{out}{Path or URL to write to. If it is not given, the bytes are returned.}
  \item{overwrite}{Replace the file if it already exists.}
  \item{level}{Compression level, 0 to 22. A higher level gives a smaller file but takes longer to write.}
  \item{f32_compress}{Store intensities as 32-bit floats, for a smaller file with slightly less precision.}
  \item{block_size}{Size of each data block before compression, in MB. A whole number, 1 or more.}
  \item{mz_window}{Width of the m/z windows the spectra are split into, in Da, so a query reads only the windows it needs. 0 keeps each spectrum whole. It only changes how the file is stored. It is not the EIC window.}
  \item{headers}{Extra HTTP headers for the upload, such as a token. Only used with a URL.}
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
