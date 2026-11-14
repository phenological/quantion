\name{match_features}
\alias{match_features}
\title{Find and match features across files}
\description{
Find the features in every file of a folder and match them across samples (untargeted). The values of each feature are medians over the samples where it was found. Instead of aligning retention times, it groups features with close m/z and rt. Small rt differences between runs are covered by rt_tolerance.
}
\usage{
match_features(directory, options = match_features_options())
}
\arguments{
  \item{directory}{Folder with the Ionic (.ion) or mzML files, one file per sample. Type: \code{character}.}
  \item{options}{Settings from \code{\link{match_features_options}()}: \code{eic}, \code{grid}, \code{grouping}, \code{find_peaks}, \code{threads = 1L}.
    \href{https://phenological.github.io/quantion/?language=r#MatchFeaturesOptions}{MatchFeaturesOptions documentation}
  }
}
\value{
One feature per matched feature (the feature table). frequency is the fraction of samples where it was found, from 0 to 1. Type: \code{data.frame of Feature}. Columns: \code{mz}, \code{rt}, \code{from}, \code{to}, \code{intensity}, \code{integral}, \code{n_points}, \code{frequency}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#match_features}
}
\examples{
\dontrun{
library(quantion)

features <- match_features("samples/", options = match_features_options(
  grouping = grouping_options(min_frequency = 0.5),
  threads  = 8L
))
print(features)
}
}
\seealso{
\code{\link{match_features_options}()}, \code{\link{find_features}()}
}
