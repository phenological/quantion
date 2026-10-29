\name{load_warp}
\alias{load_warp}
\title{Make a retention-time warp model}
\description{
Make a retention-time warp model from a loaded model.
}
\usage{
load_warp(model)
}
\arguments{
  \item{model}{A model from \code{\link{load_model}()} that holds a warp.}
}
\value{
A warp model (an external pointer of class \code{quantion_warp_model}).
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#load_warp}
}
\examples{
warp <- load_warp(load_model(system.file("extdata", "model.onnx", package = "quantion")))
}
\seealso{
\code{\link{predict_rts}()}, \code{\link{calibrate}()}
}
