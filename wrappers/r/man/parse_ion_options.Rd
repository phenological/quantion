\name{parse_ion_options}
\alias{parse_ion_options}
\title{Settings for parse_ion}
\description{
Options for parse_ion.
}
\usage{
parse_ion_options(max_cache_size = 0)
}
\arguments{
  \item{max_cache_size}{Maximum memory, in bytes, kept for the parts of the file already read. 0 = no limit. Set it to limit memory on large or remote files.}
}
\value{
An options object for \code{\link{parse_ion}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#ParseIonOptions}
}
\examples{
parse_ion_options(max_cache_size = 64e6)
}
\seealso{
\code{\link{parse_ion}()}
}
