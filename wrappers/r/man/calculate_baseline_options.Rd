\name{calculate_baseline_options}
\alias{calculate_baseline_options}
\title{Settings for calculate_baseline}
\description{
Options for calculate_baseline, which estimates the baseline with airPLS.
}
\usage{
calculate_baseline_options(lambda = 20, max_iterations = 4L)
}
\arguments{
  \item{lambda}{Smoothness of the baseline. A larger value gives a smoother baseline that follows only slow drift.}
  \item{max_iterations}{Maximum number of iterations used to estimate the baseline.}
}
\value{
An options object for \code{\link{calculate_baseline}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#CalculateBaselineOptions}
}
\examples{
calculate_baseline_options(lambda = 50)
}
\seealso{
\code{\link{calculate_baseline}()}
}
