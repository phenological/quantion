\name{calculate_eic}
\alias{calculate_eic}
\title{Extract an ion chromatogram}
\description{
Calculate the extracted ion chromatogram (EIC, also called XIC) of one m/z. For each MS1 scan, the intensities within the m/z window are summed. Use it to get the chromatogram before find_peaks or get_peak.
}
\usage{
calculate_eic(file, mz, options = calculate_eic_options())
}
\arguments{
  \item{file}{The sample to read, from parse_ion or parse_mzml. Type: \code{SampleFile}.}
  \item{mz}{The m/z of the ion to extract (Da), not the neutral mass of the compound. For example, for the protonated molecule [M+H]+ in positive mode, use M + 1.00728, and for the deprotonated molecule [M-H]- in negative mode, use M - 1.00728, where M is the monoisotopic mass. Type: \code{numeric}.}
  \item{options}{Settings from \code{\link{calculate_eic_options}()}: \code{from = 0}, \code{to = Inf}, \code{ppm_tolerance = 20}, \code{mz_tolerance = 0.005}, \code{time_unit = "MINUTES"}.
    \href{https://phenological.github.io/quantion/?language=r#CalculateEicOptions}{CalculateEicOptions documentation}
  }
}
\value{
The chromatogram, with x as the retention times in minutes and y as the summed intensities. Type: \code{DataXY list}. Fields: \code{x}, \code{y}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#calculate_eic}
}
\examples{
\donttest{
if (curl::has_internet()) {
  library(quantion)

  url <- "https://raw.githubusercontent.com/phenological/ion-files/main/data/aa_p21_LTR_20_b1_w20.ion"
  sample <- parse_ion(url)
  options <- calculate_eic_options(from = 2, to = 3, ppm_tolerance = 10)
  eic <- calculate_eic(sample, 260.103, options = options)
  print(head(data.frame(rt = eic$x, intensity = eic$y)))
}
}
}
\seealso{
\code{\link{calculate_eic_options}()}, \code{\link{find_peaks}()}
}
