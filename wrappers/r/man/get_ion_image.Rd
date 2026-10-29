\name{get_ion_image}
\alias{get_ion_image}
\title{Build an ion image}
\description{
Build an imaging map of one m/z: summed intensity at each x, y position. Only .ion files: a sample from \code{\link{parse_mzml}()} gives an \code{invalid_argument} error.
}
\usage{
get_ion_image(file, mz, options = get_ion_image_options())
}
\arguments{
  \item{file}{A sample from \code{\link{parse_ion}()}.}
  \item{mz}{The m/z to map (Da).}
  \item{options}{Settings from \code{\link{get_ion_image_options}()}.}
}
\value{
A list with \code{width}, \code{height}, \code{min_x}, \code{min_y}, \code{min_z}, \code{max_z}, \code{data} (summed intensity per pixel, row by row) and \code{counts} (scans per pixel).
}
\examples{
\dontrun{
sample <- parse_ion("sample.ion")
image <- get_ion_image(sample, 760.585, options = get_ion_image_options(tolerance = 0.05))
}
}
\seealso{
\code{\link{get_ion_image_options}()}
}
