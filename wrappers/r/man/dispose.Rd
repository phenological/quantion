\name{dispose}
\alias{dispose}
\title{Free native memory}
\description{
Free the native memory of a sample, a model or a warp model. It is also freed by the garbage collector.
}
\usage{
dispose(object)
}
\arguments{
  \item{object}{A sample, a model or a warp model.}
}
\value{
\code{NULL}, invisibly.
}
\examples{
model <- load_model(system.file("extdata", "model.onnx", package = "quantion"))
dispose(model)
}
