\name{find_noise_level}
\alias{find_noise_level}
\title{Estimate the noise level}
\description{
Estimate the noise level of a chromatogram or any signal: how noisy it is. Small, narrow peaks are grouped together, and their maximum intensity and width (in number of points) are reported as the noise.
}
\usage{
find_noise_level(y)
}
\arguments{
  \item{y}{Intensities, for example the y of a chromatogram. Type: \code{numeric}.}
}
\value{
The noise intensity and the width (in points) of the noise regions it was estimated from. Type: \code{NoiseLevel list}. Fields: \code{width}, \code{intensity}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#find_noise_level}
}
\examples{
\donttest{
if (curl::has_internet()) {
  library(quantion)

  url <- "https://raw.githubusercontent.com/phenological/ion-files/main/data/aa_p21_LTR_20_b1_w20.ion"
  sample <- parse_ion(url)
  eic <- calculate_eic(sample, 260.103, options = calculate_eic_options(from = 2, to = 3))
  noise <- find_noise_level(eic$y)$intensity
  print(noise)
}
}
}
\seealso{
\code{\link{find_peaks}()}
}
