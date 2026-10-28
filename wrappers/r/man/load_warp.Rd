\name{load_warp}
\alias{load_warp}
\title{Make a warp model}
\description{
Makes a warp model from a model loaded with \code{\link{load_model}()}. The model
stays usable, and the warp model keeps working if the model is disposed.
}
\usage{
load_warp(
  model
)
}
\arguments{
  \item{model}{A model from \code{\link{load_model}()}.}
}
\value{
A warp model for use in \code{\link{predict_rts}()} and \code{\link{calibrate}()}.
}
\examples{
model <- load_model(system.file("extdata", "model.onnx", package = "quantion"))
warp <- load_warp(model)
}
\seealso{
\code{\link{load_model}()}, \code{\link{predict_rts}()}, \code{\link{calibrate}()}
}
