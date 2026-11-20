\name{find_features}
\alias{find_features}
\title{Find every feature in one file}
\description{
Find every feature in one file without a target list (untargeted). It scans an m/z grid, builds the EIC of each grid m/z and finds its peaks.
}
\usage{
find_features(file, options = find_features_options())
}
\arguments{
  \item{file}{The file to read, from parse_ion or parse_mzml. Type: \code{SampleFile}.}
  \item{options}{Settings from \code{\link{find_features_options}()}: \code{eic}, \code{grid}, \code{find_peaks}, \code{threads = 1L}.
    \href{https://phenological.github.io/quantion/?language=r#FindFeaturesOptions}{FindFeaturesOptions documentation}
  }
}
\value{
Every feature found (the feature table), without frequency. Type: \code{data.frame of Feature}. Columns: \code{mz}, \code{rt}, \code{from}, \code{to}, \code{intensity}, \code{integral}, \code{n_points}, \code{frequency}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#find_features}
}
\examples{
\donttest{
if (curl::has_internet()) {
  library(quantion)

  base <- "https://raw.githubusercontent.com/phenological/ion-files/main/data/"
  url <- paste0(base, "untargeted_hilpos_p011_PQC_31_b1_w20.ion")
  options <- find_features_options(
    eic     = calculate_eic_options(from = 0.5, to = 1.0),
    grid    = grid_options(start = 195, end = 196),
    threads = 2L
  )

  file <- parse_ion(url)
  features <- find_features(file, options = options)
  print(features)
}
}
}
\seealso{
\code{\link{find_features_options}()}, \code{\link{match_features}()}
}
