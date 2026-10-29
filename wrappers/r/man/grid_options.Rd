\name{grid_options}
\alias{grid_options}
\title{The m/z grid searched for features}
\description{
The m/z grid that \code{\link{find_features}()} and \code{\link{match_features}()} search. Every field is checked when the object is built; a misspelt name or a wrong type gives an error.
}
\usage{
grid_options(start = 40, end = 1000, step_size = 0.005)
}
\arguments{
  \item{start}{Lowest m/z.}
  \item{end}{Highest m/z.}
  \item{step_size}{Step between grid m/z values (Da).}
}
\value{
An options object for \code{\link{find_features_options}()}.
}
\examples{
grid_options(start = 100, end = 600)
}
\seealso{
\code{\link{find_features_options}()}
}
