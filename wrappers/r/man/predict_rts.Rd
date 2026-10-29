\name{predict_rts}
\alias{predict_rts}
\title{Predict retention times}
\description{
Predict where each target elutes in one sample.
}
\usage{
predict_rts(warp, file, targets)
}
\arguments{
  \item{warp}{A warp model from \code{\link{load_warp}()}.}
  \item{file}{A sample from \code{\link{parse_ion}()} or \code{\link{parse_mzml}()}.}
  \item{targets}{A data.frame with columns \code{mz} and \code{rt}; \code{id} and \code{range} are optional and not used.}
}
\value{
A data.frame with \code{rt} and \code{uncertainty}, one row per target in the same order.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#predict_rts}
}
\examples{
\dontrun{
warp <- load_warp(load_model("model.onnx"))
predicted <- predict_rts(warp, parse_ion("sample.ion"),
                         data.frame(id = "compound_1", mz = 260.103, rt = 2.383))
}
}
\seealso{
\code{\link{load_warp}()}, \code{\link{calibrate}()}
}
