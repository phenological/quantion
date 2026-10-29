\name{find_features}
\alias{find_features}
\title{Find every feature in one file}
\description{
Untargeted: find every feature in one file by scanning an m/z grid.
}
\usage{
find_features(file, options = find_features_options())
}
\arguments{
  \item{file}{A sample from \code{\link{parse_ion}()} or \code{\link{parse_mzml}()}.}
  \item{options}{Settings from \code{\link{find_features_options}()}.}
}
\value{
A data.frame with one row per feature: \code{mz}, \code{rt}, \code{from}, \code{to}, \code{intensity}, \code{integral}, \code{n_points}, \code{noise}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#find_features}
}
\examples{
\dontrun{
sample <- parse_ion("sample.ion")
features <- find_features(sample, options = find_features_options(
  grid    = grid_options(start = 100, end = 600),
  threads = 4L
))
}
}
\seealso{
\code{\link{find_features_options}()}, \code{\link{match_features}()}
}
