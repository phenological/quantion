\name{calculate_eic}
\alias{calculate_eic}
\title{Extract an ion chromatogram}
\description{
Extract the ion chromatogram of one m/z. The retention-time range and the tolerances come from \code{\link{calculate_eic_options}()}.
}
\usage{
calculate_eic(file, mz, options = calculate_eic_options())
}
\arguments{
  \item{file}{A sample from \code{\link{parse_ion}()} or \code{\link{parse_mzml}()}.}
  \item{mz}{The m/z to extract (Da).}
  \item{options}{Settings from \code{\link{calculate_eic_options}()}: \code{from = 0}, \code{to = Inf}, \code{ppm_tolerance = 20}, \code{mz_tolerance = 0.005}, \code{time_unit = "MINUTES"}.
    \href{https://phenological.github.io/quantion/?language=r#CalculateEicOptions}{CalculateEicOptions documentation}
  }
}
\value{
A list with \code{x} (retention times in minutes) and \code{y} (summed intensities).
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#calculate_eic}
}
\examples{
\dontrun{
sample <- parse_ion("sample.ion")
eic <- calculate_eic(sample, 260.103, options = calculate_eic_options(from = 0.5, to = 5, ppm_tolerance = 10))
whole_run <- calculate_eic(sample, 260.103)
}
}
\seealso{
\code{\link{calculate_eic_options}()}, \code{\link{find_peaks}()}
}
