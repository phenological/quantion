\name{calibrate}
\alias{calibrate}
\title{Calibrate reference retention times}
\description{
Turn the retention times seen in anchor samples (LTRs, QCs) into targets for \code{\link{predict_rts}()}. Each id gets the median rt over the anchors.
}
\usage{
calibrate(warp, anchor_samples, target_lists)
}
\arguments{
  \item{warp}{A warp model from \code{\link{load_warp}()}.}
  \item{anchor_samples}{A list of samples.}
  \item{target_lists}{A list of data.frames, one per anchor sample, with columns \code{id}, \code{mz} and \code{rt}.}
}
\value{
A data.frame with \code{id}, \code{mz} and \code{rt}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#calibrate}
}
\examples{
\dontrun{
warp <- load_warp(load_model("model.onnx"))
seen <- data.frame(id = "compound_1", mz = 260.103, rt = 2.383)
calibrated <- calibrate(warp, list(parse_ion("anchor_1.ion"), parse_ion("anchor_2.ion")), list(seen, seen))
}
}
\seealso{
\code{\link{predict_rts}()}
}
