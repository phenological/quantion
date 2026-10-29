\name{load_model}
\alias{load_model}
\title{Load an ONNX model}
\description{
Load an ONNX model from a path, a URL or raw bytes, and read the keys and values stored in it.
}
\usage{
load_model(source)
}
\arguments{
  \item{source}{A path, an http(s) URL, or a raw vector of model bytes.}
}
\value{
A model (an external pointer of class \code{quantion_model}).
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#load_model}
}
\examples{
model <- load_model(system.file("extdata", "model.onnx", package = "quantion"))
get_value(model, "points")
}
\seealso{
\code{\link{load_warp}()}, \code{\link{get_keys}()}
}
