\name{match_features}
\alias{match_features}
\title{Find and match features across files}
\description{
Find the features in every file of a folder and match them across files (untargeted). The values of each feature are medians over the files where it was found. Instead of aligning retention times, it groups features with close m/z and rt. Small rt differences between runs are covered by rt_tolerance.
}
\usage{
match_features(directory, options = match_features_options())
}
\arguments{
  \item{directory}{Folder with the Ionic (.ion) or mzML files, one file per injection. Type: \code{character}.}
  \item{options}{Settings from \code{\link{match_features_options}()}: \code{eic}, \code{grid}, \code{grouping}, \code{find_peaks}, \code{threads = 1L}, \code{warp = NULL}.
    \href{https://phenological.github.io/quantion/?language=r#MatchFeaturesOptions}{MatchFeaturesOptions documentation}
  }
}
\value{
One feature per matched feature (the feature table). frequency is the fraction of files where it was found, from 0 to 1. Type: \code{data.frame of Feature}. Columns: \code{mz}, \code{rt}, \code{from}, \code{to}, \code{intensity}, \code{integral}, \code{n_points}, \code{frequency}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#match_features}
}
\examples{
\donttest{
if (curl::has_internet()) {
  library(quantion)

  folder <- file.path(tempdir(), "files")
  dir.create(folder, showWarnings = FALSE)
  url <- "https://raw.githubusercontent.com/phenological/ion-files/main/data/aa_p21_LTR_20_b1_w20.ion"
  utils::download.file(url, file.path(folder, "file.ion"), mode = "wb", quiet = TRUE)
  features <- match_features(folder, options = match_features_options(
    eic      = calculate_eic_options(from = 2.3, to = 2.45),
    grid     = grid_options(start = 255, end = 265),
    grouping = grouping_options(min_frequency = 0.5),
    threads  = 1L
  ))
  print(features)
}
}
}
\seealso{
\code{\link{match_features_options}()}, \code{\link{find_features}()}
}
