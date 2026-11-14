\name{deisotope_features_options}
\alias{deisotope_features_options}
\title{Settings for deisotope_features}
\description{
Options for deisotope_features, which say how close an isotope peak must be.
}
\usage{
deisotope_features_options(rt_tolerance = 0.02, mz_tolerance = 0.005)
}
\arguments{
  \item{rt_tolerance}{An isotope belongs to M only when its rt is within this many minutes of M.}
  \item{mz_tolerance}{A feature is an isotope only when its m/z is within this many Da of the expected isotope position.}
}
\value{
A \code{deisotope_features_options} object for \code{\link{deisotope_features}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#DeisotopeFeaturesOptions}
}
