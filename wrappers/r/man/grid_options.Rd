\name{grid_options}
\alias{grid_options}
\title{The m/z grid searched for features}
\description{
The m/z grid that find_features and match_features scan, from start to end in steps of step_size.
}
\usage{
grid_options(start = 40, end = 1000, step_size = 0.005)
}
\arguments{
  \item{start}{Lowest m/z of the grid.}
  \item{end}{Highest m/z of the grid.}
  \item{step_size}{Distance between two grid m/z values, in Da. A smaller step is finer but slower.}
}
\value{
An options object for \code{\link{find_features_options}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#GridOptions}
}
\examples{
grid_options(start = 100, end = 600)
}
\seealso{
\code{\link{find_features_options}()}
}
