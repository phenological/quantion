\name{match_features}
\alias{match_features}
\title{Find and match features across files}
\description{
Untargeted across samples: find features in every file of a folder and match them. Values are medians over the samples where a feature was found.
}
\usage{
match_features(directory, options = match_features_options())
}
\arguments{
  \item{directory}{A folder with .ion or mzML files.}
  \item{options}{Settings from \code{\link{match_features_options}()}.}
}
\value{
A data.frame like \code{\link{find_features}()} plus \code{frequency}, the fraction of samples (0 to 1) where the feature was found.
}
\examples{
\dontrun{
features <- match_features("samples/", options = match_features_options(
  grouping = grouping_options(min_frequency = 0.5),
  threads  = 8L
))
}
}
\seealso{
\code{\link{match_features_options}()}, \code{\link{find_features}()}
}
