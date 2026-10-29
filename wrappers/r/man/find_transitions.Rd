\name{find_transitions}
\alias{find_transitions}
\title{Find the peak of each SRM transition}
\description{
For each SRM transition, take the stored chromatogram whose precursor (Q1) and product (Q3) are both within 0.005 m/z, and find its peak near \code{rt} plus or minus \code{range}. Needs the whole file: a sample opened from a URL gives a \code{remote_file} error.
}
\usage{
find_transitions(file, targets, options = find_transitions_options())
}
\arguments{
  \item{file}{A sample from \code{\link{parse_ion}()} or \code{\link{parse_mzml}()}.}
  \item{targets}{A data.frame with columns \code{precursor}, \code{product}, \code{rt} and \code{range}, and an optional \code{id}.}
  \item{options}{Settings from \code{\link{find_transitions_options}()}.}
}
\value{
A data.frame with one row per transition, like \code{\link{find_targets}()}. No matching chromatogram gives a row of zeros.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#find_transitions}
}
\examples{
\dontrun{
sample <- parse_ion("sample.ion")
transitions <- data.frame(id = "compound_1", precursor = 500.3, product = 184.1, rt = 3.2, range = 0.4)
peaks <- find_transitions(sample, transitions, options = find_transitions_options(threads = 2L))
}
}
\seealso{
\code{\link{find_transitions_options}()}, \code{\link{find_targets}()}
}
