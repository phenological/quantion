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
  file,
  targets
)
}
\arguments{
  \item{warp}{A warp model from \code{\link{load_warp}()}.}
  \item{file}{Sample file from \code{\link{parse_ion}()}.}
  \item{targets}{A data frame with columns \code{mz} and \code{rt}.}
}
\value{
A data frame with columns \code{rt} (the predicted retention time) and
\code{uncertainty}, one row per target, in the same order.
}
\examples{
\donttest{
model <- load_model(system.file("extdata", "model.onnx", package = "quantion"))
warp <- load_warp(model)
url <- "https://raw.githubusercontent.com/phenological/ion-files/main/data/p21_LTR_20_b1_w20.ion"
sample <- parse_ion(url)
targets <- data.frame(mz = 260.103, rt = 2.383)
rts <- predict_rts(warp, sample, targets)
}
}
\seealso{
\code{\link{load_warp}()}, \code{\link{calibrate}()}, \code{\link{find_targets}()}
}
