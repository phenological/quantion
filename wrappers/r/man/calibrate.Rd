\name{calibrate}
\alias{calibrate}
\title{Calibrate targets from anchor samples}
\description{
Turns the retention times seen in anchor samples, such as LTRs or QCs, into
targets for \code{\link{predict_rts}()} in every other sample. Each target gets
the median calibrated retention time over the samples where it was seen. One
sample works, three are better, five or more are best.
}
\usage{
calibrate(
  warp,
  anchor_samples,
  target_lists
)
}
\arguments{
  \item{warp}{A warp model from \code{\link{load_warp}()}.}
  \item{anchor_samples}{A list of sample files from \code{\link{parse_ion}()}.}
  \item{target_lists}{A list with one data frame per sample, with columns \code{id}, \code{mz} and the \code{rt} seen in that sample.}
}
\value{
A data frame with columns \code{id}, \code{mz} and \code{rt}, one row per id.
}
\examples{
\donttest{
warp <- load_warp(load_model("https://example.org/model.onnx"))
ltr <- parse_ion("https://example.org/ltr.ion")
seen <- data.frame(id = "feature_1", mz = 300.1, rt = 2.38)
calibrated <- calibrate(warp, list(ltr), list(seen))
}
}
\seealso{
\code{\link{predict_rts}()}
}
