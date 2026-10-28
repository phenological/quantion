\name{calibrate}
\alias{calibrate}
\title{Calibrate targets from anchor samples}
\description{
Turns the retention times seen in anchor samples, such as LTRs or QCs, into
targets for \code{\link{predict_rts}()} in every other sample. Each target gets
the median calibrated retention time over the samples where it was seen. One
sample works, three are better, five or more are best.

Each target list must hold the retention times measured in its own anchor
sample. Passing the same generic target list for every sample still runs, but
the calibrated retention times are wrong.
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
  \item{target_lists}{A list with one data frame per sample, with columns \code{id}, \code{mz} and \code{rt}. Each \code{rt} must be the retention time seen in that anchor sample, not a generic library value: a shared target list gives wrong results with no error.}
}
\value{
A data frame with columns \code{id}, \code{mz} and \code{rt}, one row per id.
}
\examples{
\donttest{
model <- load_model(system.file("extdata", "model.onnx", package = "quantion"))
warp <- load_warp(model)
url <- "https://raw.githubusercontent.com/phenological/ion-files/main/data/p21_LTR_20_b1_w20.ion"
sample <- parse_ion(url)
seen <- data.frame(id = "Alanine", mz = 260.103, rt = 2.383)
calibrated <- calibrate(warp, list(sample), list(seen))
}
}
\seealso{
\code{\link{predict_rts}()}
}
