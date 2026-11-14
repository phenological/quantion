\name{calculate_baseline}
\alias{calculate_baseline}
\title{Estimate a baseline}
\description{
Estimate the baseline (slow background drift) of the input chromatogram or signal using airPLS (Zhang et al., 2010).
}
\usage{
calculate_baseline(y, options = calculate_baseline_options())
}
\arguments{
  \item{y}{Intensities, for example the y of a chromatogram. Type: \code{numeric}.}
  \item{options}{Settings from \code{\link{calculate_baseline_options}()}: \code{lambda = 20}, \code{max_iterations = 4L}.
    \href{https://phenological.github.io/quantion/?language=r#CalculateBaselineOptions}{CalculateBaselineOptions documentation}
  }
}
\value{
The baseline: one value per point of y. Type: \code{numeric}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#calculate_baseline}
}
\examples{
\donttest{
if (curl::has_internet()) {
  library(quantion)

  url <- "https://raw.githubusercontent.com/phenological/ion-files/main/data/aa_p21_LTR_20_b1_w20.ion"
  sample <- parse_ion(url)
  eic <- calculate_eic(sample, 260.103, options = calculate_eic_options(from = 2, to = 3))
  baseline <- calculate_baseline(eic$y, options = calculate_baseline_options(lambda = 50))
  print(head(baseline))
}
}
}
\seealso{
\code{\link{calculate_baseline_options}()}
}
