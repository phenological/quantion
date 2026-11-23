\name{get_transition_metadata_options}
\alias{get_transition_metadata_options}
\title{Settings for get_transition_metadata}
\description{
Options for get_transition_metadata.
}
\usage{
get_transition_metadata_options(tags = character(0))
}
\arguments{
  \item{tags}{Which tags to keep, by name. A row is kept when the last tag of its path matches, so precursor keeps chromatogram/precursor but not its isolationWindow. An empty list keeps every tag. The names are chromatogram, referenceableParamGroupRef, precursor, isolationWindow, selectedIonList, selectedIon, activation, product, binaryDataArrayList and binaryDataArray.}
}
\value{
An options object for \code{\link{get_transition_metadata}()}.
}
\section{Documentation}{
\url{https://phenological.github.io/quantion/?language=r#GetTransitionMetadataOptions}
}
\examples{
get_transition_metadata_options(tags = c("precursor", "isolationWindow"))
}
\seealso{
\code{\link{get_transition_metadata}()}
}
