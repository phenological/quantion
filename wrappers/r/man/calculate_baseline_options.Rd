\name{calculate_baseline_options}
\alias{calculate_baseline_options}
\title{Settings for calculate_baseline}
\description{
Settings for \code{\link{calculate_baseline}()}. Every field is checked when the object is built; a misspelt name or a wrong type gives an error.
}
\usage{
calculate_baseline_options(lambda = 20, max_iterations = 4L)
}
\arguments{
  \item{lambda}{Smoothness. Larger is smoother.}
  \item{max_iterations}{Maximum iterations.}
}
\value{
An options object for \code{\link{calculate_baseline}()}.
}
\examples{
calculate_baseline_options(lambda = 50)
}
\seealso{
\code{\link{calculate_baseline}()}
}
