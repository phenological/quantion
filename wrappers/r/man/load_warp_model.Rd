\name{load_warp_model}
\alias{load_warp_model}
\title{Load a warp model}
\description{
Loads an ONNX warp model that predicts retention times. Load it once and use it
with \code{\link{predict_rts}()} for every file.
}
\usage{
load_warp_model(
  source
)
}
\arguments{
  \item{source}{A path, a URL starting with \code{http://} or \code{https://}, or a raw vector with the model bytes.}
}
\value{
A warp model for use in \code{\link{predict_rts}()}.
}
\examples{
\donttest{
warp <- load_warp_model("https://example.org/model.onnx")
}
}
\seealso{
\code{\link{predict_rts}()}, \code{\link{dispose}()}
}
