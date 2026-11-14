\name{calculate_eic_options}
\alias{calculate_eic_options}
\title{Settings for an EIC}
\description{
Options for calculate_eic, including the retention-time range to read and the m/z window. The same options are used by find_targets, find_features and match_features. The wider of the ppm and Da windows is used.
}
\usage{
calculate_eic_options(from = 0, to = Inf, ppm_tolerance = 20, mz_tolerance = 0.005,
                      time_unit = "MINUTES")
}
\arguments{
  \item{from}{Start of the retention-time range to read.}
  \item{to}{End of the retention-time range to read. By default, the run is read to the end.}
  \item{ppm_tolerance}{Half-width of the m/z window, in ppm of the m/z.}
  \item{mz_tolerance}{Half-width of the m/z window, in Da.}
  \item{time_unit}{Unit of from and to, minutes or seconds.}
}
\value{
An options object for \code{\link{calculate_eic}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#CalculateEicOptions}
}
\examples{
calculate_eic_options(from = 0.5, to = 5, ppm_tolerance = 10)
}
\seealso{
\code{\link{calculate_eic}()}
}
