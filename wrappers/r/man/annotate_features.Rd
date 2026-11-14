\name{annotate_features}
\alias{annotate_features}
\title{Name features from their MS2 spectra}
\description{
Name features (annotation) by matching their MS2 spectra against a spectral library. For each feature and sample, it takes the positive MS2 scan closest to the peak apex, whose precursor matches the feature m/z and whose rt is within rt_range. A model turns the scan into a vector, which is scored against the library spectra with the same precursor m/z. By default, the library and the model come from the josoriom/cipher releases.
}
\usage{
annotate_features(files, features, options = annotate_features_options())
}
\arguments{
  \item{files}{The samples, from parse_ion or parse_mzml. It must not be empty. Type: \code{list of SampleFile}.}
  \item{features}{The features to name, each with mz and rt. The output of find_features and match_features works as is. An empty list gives an empty result. Type: \code{data.frame(mz, rt)}.}
  \item{options}{Settings from \code{\link{annotate_features_options}()}: \code{library = "https://github.com/josoriom/cipher/releases/latest/download/library_positive.ion"}, \code{model = "https://github.com/josoriom/cipher/releases/latest/download/model.onnx"}, \code{matches = 3L}, \code{ppm_tolerance = 20}, \code{mz_tolerance = 0.005}, \code{rt_range = 0.3}, \code{min_relative_intensity = 0.02}.
    \href{https://phenological.github.io/quantion/?language=r#AnnotateFeaturesOptions}{AnnotateFeaturesOptions documentation}
  }
}
\value{
One annotation per feature, in the same order as features. Type: \code{data.frame of FeatureAnnotation}. Columns: \code{mz}, \code{rt}, \code{label}, \code{id}, \code{database}, \code{frequency}, \code{score}, \code{cosine}, \code{files}.
}
\section{Warning}{
This is an experimental feature.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#annotate_features}
}
\examples{
\dontrun{
library(quantion)

url <- "https://raw.githubusercontent.com/phenological/ion-files/main/data/aa_p21_LTR_20_b1_w20.ion"
file <- parse_ion(url)
features <- find_features(file, find_features_options(
  eic  = calculate_eic_options(from = 2.3, to = 2.45),
  grid = grid_options(start = 255, end = 265)
))
annotations <- annotate_features(list(file), features, annotate_features_options(matches = 3L))
print(annotations[, c("mz", "rt", "label", "id", "database", "frequency", "score")])
}
}
