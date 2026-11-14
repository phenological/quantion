\name{annotate_features_options}
\alias{annotate_features_options}
\title{Settings for annotate_features}
\description{
Options for annotate_features, including the library, the model, the tolerances and how many matches to keep.
}
\usage{
annotate_features_options(library = .DEFAULT_LIBRARY, model = .DEFAULT_MODEL,
                          matches = 3L, ppm_tolerance = 20, mz_tolerance = 0.005,
                          rt_range = 0.3, min_relative_intensity = 0.02)
}
\arguments{
  \item{library}{Spectral library, as a path or URL to an Ionic (.ion) file. Each spectrum has a precursor m/z, a vector array and the params the model names. Only Ionic (.ion) libraries work.}
  \item{model}{Spectrum model, as a path or URL to an ONNX file.}
  \item{matches}{How many matches to keep per file, best first.}
  \item{ppm_tolerance}{Precursor m/z tolerance in ppm. The larger of the ppm and Da tolerances is used.}
  \item{mz_tolerance}{Precursor m/z tolerance in Da. The larger of the ppm and Da tolerances is used.}
  \item{rt_range}{Largest distance, in minutes, from the peak apex to the MS2 scan.}
  \item{min_relative_intensity}{MS2 peaks below this share of the base peak are removed before matching, from 0 to 1.}
}
\value{
A \code{annotate_features_options} object for \code{\link{annotate_features}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#AnnotateFeaturesOptions}
}
