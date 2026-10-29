\name{find_targets}
\alias{find_targets}
\title{Find the peak of each target}
\description{
For each target, build its EIC at \code{mz} and find its peak near \code{rt}. A target without \code{range} (or with 0) searches its whole EIC.
}
\usage{
find_targets(file, targets, options = find_targets_options())
}
\arguments{
  \item{file}{A sample from \code{\link{parse_ion}()} or \code{\link{parse_mzml}()}.}
  \item{targets}{A data.frame with columns \code{mz} and \code{rt}, and optional \code{id} and \code{range}.}
  \item{options}{Settings from \code{\link{find_targets_options}()}.}
}
\value{
A data.frame with one row per target in the same order: \code{rt}, \code{from}, \code{to}, \code{intensity}, \code{integral}, \code{n_points}, \code{noise}, \code{r2}. A target with no peak gets a row of zeros.
}
\examples{
\dontrun{
sample <- parse_ion("sample.ion")
targets <- data.frame(
  id    = c("compound_1", "compound_2"),
  mz    = c(260.103, 260.103),
  rt    = c(2.383, 2.189),
  range = c(0.05, 0.05)
)
peaks <- find_targets(sample, targets, options = find_targets_options(
  eic        = calculate_eic_options(from = 0.5, to = 5),
  find_peaks = find_peaks_options(min_intensity = 1000),
  threads    = 4L
))
}
}
\seealso{
\code{\link{find_targets_options}()}, \code{\link{find_transitions}()}
}
