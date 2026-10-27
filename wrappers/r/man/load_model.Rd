\name{load_model}
\alias{load_model}
\title{Load an ONNX model}
\description{
Loads an ONNX model. Read the keys and values stored in it with
\code{\link{get_keys}()}, and make a warp model from it with \code{\link{load_warp}()}.
}
\usage{
load_model(
  source
)
}
\arguments{
  \item{source}{A path, a URL starting with \code{http://} or \code{https://}, or a raw vector with the model bytes.}
}
\value{
A model for use in \code{\link{load_warp}()} and \code{\link{get_keys}()}.
}
\examples{
\donttest{
model <- load_model("https://example.org/model.onnx")
get_value(model, "points")
}
}
\seealso{
\code{\link{load_warp}()}, \code{\link{get_keys}()}, \code{\link{dispose}()}
}
