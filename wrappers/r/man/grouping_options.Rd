\name{grouping_options}
\alias{grouping_options}
\title{How features are matched across samples}
\description{
How match_features decides that features from different samples are the same feature.
}
\usage{
grouping_options(ppm_tolerance = 5, mz_tolerance = 0.0025, rt_tolerance = 0.05,
                 min_frequency = 0)
}
\arguments{
  \item{ppm_tolerance}{Features are the same when their m/z is within this many ppm, or within mz_tolerance.}
  \item{mz_tolerance}{Features are the same when their m/z is within this many Da, or within ppm_tolerance.}
  \item{rt_tolerance}{Features are the same only when their rt is also within this many minutes.}
  \item{min_frequency}{Keep a feature only if it is found in at least this fraction of samples, from 0 to 1. For example, 0.8 keeps features seen in 80\% of the samples.}
}
\value{
An options object for \code{\link{match_features_options}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#GroupingOptions}
}
\examples{
grouping_options(min_frequency = 0.5)
}
\seealso{
\code{\link{match_features_options}()}
}
