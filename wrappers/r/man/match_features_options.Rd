\name{match_features_options}
\alias{match_features_options}
\title{Settings for match_features}
\description{
Settings for \code{\link{match_features}()}. Every field is checked when the object is built; a misspelt name or a wrong type gives an error.
}
\usage{
match_features_options(eic = calculate_eic_options(), grid = grid_options(), grouping = grouping_options(),
                       find_peaks = find_peaks_options(), threads = 1L)
}
\arguments{
  \item{eic}{Retention-time range, tolerances and time unit from \code{\link{calculate_eic_options}()}.}
  \item{grid}{The m/z grid from \code{\link{grid_options}()}.}
  \item{grouping}{How features are matched, from \code{\link{grouping_options}()}.}
  \item{find_peaks}{Peak-picking settings from \code{\link{find_peaks_options}()}.}
  \item{threads}{Number of threads to use.}
}
\value{
An options object for \code{\link{match_features}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#MatchFeaturesOptions}
}
\examples{
match_features_options(grouping = grouping_options(min_frequency = 0.5), threads = 8L)
}
\seealso{
\code{\link{match_features}()}
}
