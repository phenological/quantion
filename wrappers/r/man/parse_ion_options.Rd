\name{parse_ion_options}
\alias{parse_ion_options}
\title{Settings for parse_ion}
\description{
Settings for \code{\link{parse_ion}()}. Every field is checked when the object is built; a misspelt name or a wrong type gives an error.
}
\usage{
parse_ion_options(max_cache_size = 0)
}
\arguments{
  \item{max_cache_size}{Cache size in bytes. 0 means no limit.}
}
\value{
An options object for \code{\link{parse_ion}()}.
}
\examples{
parse_ion_options(max_cache_size = 64e6)
}
\seealso{
\code{\link{parse_ion}()}
}
