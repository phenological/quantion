\name{annotate_features}
\alias{annotate_features}
\title{Name features from their MS2 spectra}
\description{
Name features from their MS2 spectra. For each feature, every sample gives its closest MS2 scan; a spectrum model turns it into a vector, which is compared with the library spectra of the same precursor m/z. The library and the model are read from a path or a URL; over a URL only the bytes needed are fetched, in one batch per source.
}
\usage{
annotate_features(files, features, options = annotate_features_options())
}
\arguments{
  \item{files}{The samples, from parse_ion or parse_mzml. Must not be empty. Type: \code{list of SampleFile}.}
  \item{features}{The features to name: mz and rt each. find_features and match_features output works as is. Empty gives an empty result. Type: \code{data.frame(mz, rt)}.}
  \item{options}{Settings from \code{\link{annotate_features_options}()}: \code{library = "https://github.com/josoriom/cipher/releases/latest/download/library_positive.ion"}, \code{model = "https://github.com/josoriom/cipher/releases/latest/download/model.onnx"}, \code{matches = 3L}, \code{ppm_tolerance = 20}, \code{mz_tolerance = 0.005}, \code{rt_range = 0.3}, \code{min_relative_intensity = 0.02}.
    \href{https://phenological.github.io/quantion/?language=r#AnnotateFeaturesOptions}{AnnotateFeaturesOptions documentation}
  }
}
\value{
One FeatureAnnotation per feature, in the same order. Type: \code{data.frame of FeatureAnnotation}. Columns: \code{mz}, \code{rt}, \code{label}, \code{id}, \code{database}, \code{frequency}, \code{score}, \code{cosine}, \code{files}.
}
\section{Warning}{
The model sets how spectra are read through its metadata keys: max_peaks, max_mz, intensity_scale (SQRT or NONE), vector_accession (the library array that holds the vectors), and compound, id, database and label (the library params that name each spectrum). A missing or invalid key gives an error.
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
