\name{match_features_options}
\alias{match_features_options}
\title{Settings for match_features}
\description{
Options for match_features. They are the find_features options plus the grouping across samples.
}
\usage{
match_features_options(eic = calculate_eic_options(), grid = grid_options(),
                       grouping = grouping_options(), find_peaks = find_peaks_options(),
                       threads = 1L)
}
\arguments{
  \item{eic}{Options for the EICs, with the retention-time range (from, to), the m/z tolerances and the time unit. See \code{\link{calculate_eic_options}()}.}
  \item{grid}{The m/z grid to scan. See \code{\link{grid_options}()}.}
  \item{grouping}{How features are matched across samples. See \code{\link{grouping_options}()}.}
  \item{find_peaks}{Options for find_peaks, used for every grid m/z. See \code{\link{find_peaks_options}()}.}
  \item{threads}{Number of threads (CPU cores) used to process the files in parallel.}
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
