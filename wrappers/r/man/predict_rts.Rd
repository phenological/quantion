\name{predict_rts}
\alias{predict_rts}
\title{Predict retention times with a warp model}
\description{
Predicts the retention time of each target in a sample file. The file is read
once and the model runs once. The file's scans must cover the time range of the
model, or this stops with an error.
}
\usage{
predict_rts(
  warp,
  bin,
  targets
)
}
\arguments{
  \item{warp}{A warp model from \code{\link{load_warp_model}()}.}
  \item{bin}{Sample file from \code{\link{parse_ion}()}.}
  \item{targets}{A data frame with columns \code{mz} and \code{rt}.}
}
\value{
A data frame with columns \code{rt} (the predicted retention time) and
\code{uncertainty}, one row per target, in the same order.
}
\examples{
\donttest{
warp <- load_warp_model("https://example.org/model.onnx")
sample <- parse_ion("https://example.org/file.ion")
targets <- data.frame(mz = c(250.1, 310.2), rt = c(1.2, 3.1))
rts <- predict_rts(warp, sample, targets)
targets$rt <- rts$rt
}
}
\seealso{
\code{\link{load_warp_model}()}, \code{\link{get_peaks_from_eic}()}
}
