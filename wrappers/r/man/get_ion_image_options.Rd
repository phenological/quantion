\name{get_ion_image_options}
\alias{get_ion_image_options}
\title{Settings for get_ion_image}
\description{
Options for get_ion_image, including the m/z window and the MS level.
}
\usage{
get_ion_image_options(tolerance = 0.125, level = 1L, on_progress = NULL)
}
\arguments{
  \item{tolerance}{Half-width of the m/z window, in Da. There is no ppm option here.}
  \item{level}{MS level of the scans used.}
  \item{on_progress}{Function called while reading, with the bytes fetched, the total bytes and the bytes held in memory. Useful for a progress bar on remote files.}
}
\value{
An options object for \code{\link{get_ion_image}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#GetIonImageOptions}
}
\examples{
get_ion_image_options(tolerance = 0.05)
}
\seealso{
\code{\link{get_ion_image}()}
}
