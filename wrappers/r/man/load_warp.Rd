\name{load_warp}
\alias{load_warp}
\title{Make a retention-time warp model}
\description{
When retention times shift between batches or within a batch, some peaks are easy to miss, e.g. low-intensity peaks. A deepwarp model predicts how each LC-MS run shifts, so peak detection finds the right peaks. Make a retention-time warp model from a loaded model. Use it with calibrate and predict_rts to correct rt shifts between runs.
}
\usage{
load_warp(model)
}
\arguments{
  \item{model}{A model from load_model that holds a warp. Type: \code{Model}.}
}
\value{
The warp model. Type: \code{WarpModel}.
}
\section{Warning}{
Works with any ONNX model that has the right inputs, outputs and keys. See them in deepwarp: https://github.com/josoriom/deepwarp#inputs-outputs-and-keys
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#load_warp}
}
\examples{
\donttest{
if (curl::has_internet()) {
  library(quantion)

  fixtures <- "https://raw.githubusercontent.com/phenological/quantion/main/core/tests/fixtures"
  model <- load_model(paste0(fixtures, "/shift_model.onnx"))
  warp <- load_warp(model)
  print(warp)

  dispose(warp)
  dispose(model)
}
}
}
\seealso{
\code{\link{predict_rts}()}, \code{\link{calibrate}()}
}
