\name{dispose}
\alias{dispose}
\title{Free a sample file or a warp model}
\description{
Releases native memory held by a sample file or a warp model. Call this when you want to free
memory immediately rather than waiting for R garbage collection.
}
\usage{
dispose(
  bin
)
}
\arguments{
  \item{bin}{Sample file from \code{\link{parse_mzml}()} or \code{\link{parse_ion}()}, or a warp model from \code{\link{load_warp_model}()}.}
}
\value{
\code{NULL}, called for its side effect.
}
\seealso{
\code{\link{parse_mzml}()}, \code{\link{parse_ion}()}, \code{\link{load_warp_model}()}
}
