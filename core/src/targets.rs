use crate::{
    CalculateEicOptions, FindPeaksOptions,
    error::{
        QuantionError, check_finite, check_mz_and_rt, check_not_negative, check_positive,
        check_threads,
    },
    files::{ScanParam, from_fast_error, read_error},
    sample_file::{FileSource, SampleFile},
    utilities::{
        find_targets::find_targets as read_targets,
        find_transitions::{
            find_ion_chromatogram, find_transitions as read_transitions, get_chromatogram,
            get_ion_transition, get_transition as read_transition,
        },
        get_scan_metadata::{TRANSITION_METADATA_TAGS, get_chromatogram_params, get_tag},
        structs::{DataXY, Peak, Target, Transition, TransitionQuery},
    },
};

/// **find_targets** · Targeted
///
/// For each target, build its EIC at mz and find its peak near rt ± range.
///
/// - `file`: The sample to read, from parse_ion or parse_mzml.
/// - `targets`: What to look for. A target without range searches its whole EIC. Each Target has `id`, `mz`, `rt`, `range`. See [Target](https://phenological.github.io/quantion/?language=rust#Target).
///   Example: `let target = Target { id: Some("compound_1".into()), mz: 260.103, rt: 2.383, range: Some(0.05) };`
/// - `options`: Optional settings: `eic`, `find_peaks`, `threads` = 1. See [FindTargetsOptions](https://phenological.github.io/quantion/?language=rust#FindTargetsOptions).
///
/// Returns: One Peak per target, in the same order. A target with no peak gets a Peak with every field 0.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#find_targets)
pub(crate) fn check_targets(
    function: &'static str,
    targets: &[Target],
) -> Result<(), QuantionError> {
    for (index, target) in targets.iter().enumerate() {
        check_mz_and_rt(function, &format!("targets[{index}]"), target.mz, target.rt)?;
        if let Some(range) = target.range {
            check_not_negative(function, &format!("targets[{index}].range"), range)?;
        }
    }
    Ok(())
}

/// **FindTargetsOptions** · Options
///
/// Options for find_targets, including the EIC window, the peak-finding settings and the threads.
///
/// Fields:
/// - `eic`: Options for the EICs, with the retention-time range (from, to), the m/z tolerances and the time unit. See [CalculateEicOptions](https://phenological.github.io/quantion/?language=rust#CalculateEicOptions).
/// - `find_peaks`: Options for find_peaks, used for every target. See [FindPeaksOptions](https://phenological.github.io/quantion/?language=rust#FindPeaksOptions).
/// - `threads`: Number of threads (CPU cores) used to process the targets in parallel. Default `1`.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#FindTargetsOptions)
#[derive(Clone, Debug, PartialEq)]
pub struct FindTargetsOptions {
    pub eic: CalculateEicOptions,
    pub find_peaks: FindPeaksOptions,
    pub threads: usize,
}

impl Default for FindTargetsOptions {
    fn default() -> Self {
        Self {
            eic: CalculateEicOptions::default(),
            find_peaks: FindPeaksOptions::default(),
            threads: 1,
        }
    }
}

/// **find_targets** · Targeted
///
/// Quantify a list of known compounds (targets) in one file. For each target, it builds the EIC at its mz and finds its peak near rt ± range. Use it for a targeted panel.
///
/// - `file`: The sample to read, from parse_ion or parse_mzml.
/// - `targets`: The compounds to look for, each with id, mz, rt and an optional range. A target without range searches its whole EIC. Each Target has `id`, `mz`, `rt`, `range`. See [Target](https://phenological.github.io/quantion/?language=rust#Target).
///   Example: `let target = Target { id: Some("compound_1".into()), mz: 260.103, rt: 2.383, range: Some(0.05) };`
/// - `options`: Optional settings: `eic`, `find_peaks`, `threads` = 1. See [FindTargetsOptions](https://phenological.github.io/quantion/?language=rust#FindTargetsOptions).
///
/// Returns: One peak per target, in the same order as targets. A target with no peak gets an empty peak (every field is 0).
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#find_targets)
pub fn find_targets(
    file: &mut SampleFile,
    targets: &[Target],
    options: FindTargetsOptions,
) -> Result<Vec<Peak>, QuantionError> {
    const FUNCTION: &str = "find_targets";
    options.eic.check(FUNCTION, "eic.")?;
    options.find_peaks.check(FUNCTION)?;
    check_threads(FUNCTION, options.threads)?;
    check_targets(FUNCTION, targets)?;
    let targets_in_minutes: Vec<Target> = targets
        .iter()
        .map(|target| Target {
            rt: options.eic.to_minutes(target.rt),
            range: target.range.map(|range| options.eic.to_minutes(range)),
            ..target.clone()
        })
        .collect();
    let settings = FindTargetsOptions {
        eic: options.eic.in_minutes(),
        ..options.clone()
    };
    read_targets(
        &mut file.reader(),
        &targets_in_minutes,
        options.eic.get_window(),
        settings,
    )
    .map_err(|error| from_fast_error(FUNCTION, error))
}

/// **FindTransitionsOptions** · Options
///
/// Options for find_transitions, including the peak-finding settings and the threads.
///
/// Fields:
/// - `find_peaks`: Options for find_peaks, used for every transition. See [FindPeaksOptions](https://phenological.github.io/quantion/?language=rust#FindPeaksOptions).
/// - `threads`: Number of threads (CPU cores) used to process the transitions in parallel. Default `1`.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#FindTransitionsOptions)
#[derive(Clone, Debug, PartialEq)]
pub struct FindTransitionsOptions {
    pub find_peaks: FindPeaksOptions,
    pub threads: usize,
}

impl Default for FindTransitionsOptions {
    fn default() -> Self {
        Self {
            find_peaks: FindPeaksOptions::default(),
            threads: 1,
        }
    }
}

/// **find_transitions** · Targeted
///
/// Quantify SRM or MRM transitions in one file. For each transition, it takes the stored chromatogram whose precursor (Q1) and product (Q3) are both within 0.005 m/z, and finds its peak near rt ± range.
///
/// **Warning:** Needs the whole file. A sample opened from a URL gives a remote_file error; open it from a path or bytes.
///
/// - `file`: The sample to read, from parse_ion or parse_mzml.
/// - `targets`: The transitions to look for, each with id, precursor, product, rt and range. Each Transition has `id`, `precursor`, `product`, `rt`, `range`. See [Transition](https://phenological.github.io/quantion/?language=rust#Transition).
///   Example: `let transition = Transition { id: Some("compound_1".into()), precursor: 89.04768, product: 89.04768, rt: 2.385, range: 0.1 };`
/// - `options`: Optional settings: `find_peaks`, `threads` = 1. See [FindTransitionsOptions](https://phenological.github.io/quantion/?language=rust#FindTransitionsOptions).
///
/// Returns: One peak per transition, in the same order. A transition with no matching chromatogram gets an empty peak (every field is 0).
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#find_transitions)
pub fn find_transitions(
    file: &mut SampleFile,
    targets: &[Transition],
    options: FindTransitionsOptions,
) -> Result<Vec<Peak>, QuantionError> {
    const FUNCTION: &str = "find_transitions";
    options.find_peaks.check(FUNCTION)?;
    check_threads(FUNCTION, options.threads)?;
    for (index, target) in targets.iter().enumerate() {
        check_positive(
            FUNCTION,
            &format!("targets[{index}].precursor"),
            target.precursor,
        )?;
        check_positive(
            FUNCTION,
            &format!("targets[{index}].product"),
            target.product,
        )?;
        check_finite(FUNCTION, &format!("targets[{index}].rt"), target.rt)?;
        check_positive(FUNCTION, &format!("targets[{index}].range"), target.range)?;
    }
    file.with_mzml(FUNCTION, |mzml| {
        Ok(read_transitions(mzml, targets, options.clone())
            .unwrap_or_else(|| vec![Peak::default(); targets.len()]))
    })
}

/// **get_transition** · Targeted
///
/// Read one stored chromatogram, such as the TIC or an SRM or MRM transition, by its index or by its precursor (Q1) and product (Q3), both within 0.005 m/z. Use it to draw a trace before get_peak or find_peaks.
///
/// **Warning:** A sample opened from a URL gives a remote_file error; open it from a path or bytes.
///
/// - `file`: The sample to read, from parse_ion or parse_mzml.
/// - `query`: Which chromatogram: an index counted from 0, or a precursor (Q1) and product (Q3) pair. Each TransitionQuery has `Index`, `Transition`. See [TransitionQuery](https://phenological.github.io/quantion/?language=rust#TransitionQuery).
///   Example: `let query = TransitionQuery::Transition { precursor: 89.04768, product: 89.04768 };`
///
/// Returns: The chromatogram, with x as the retention times in minutes and y as the intensities. Both are empty when no stored chromatogram matches.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#get_transition)
pub fn get_transition(
    file: &mut SampleFile,
    query: TransitionQuery,
) -> Result<DataXY, QuantionError> {
    const FUNCTION: &str = "get_transition";
    check_transition_query(FUNCTION, query)?;
    match &mut file.source {
        FileSource::Full(mzml) => Ok(read_transition(mzml, query)),
        FileSource::Lazy(ion) => {
            get_ion_transition(ion, query).map_err(|error| read_error(FUNCTION, error))
        }
        FileSource::Remote(_) => Err(QuantionError::remote_file(FUNCTION)),
    }
}

fn check_transition_query(
    function: &'static str,
    query: TransitionQuery,
) -> Result<(), QuantionError> {
    if let TransitionQuery::Transition { precursor, product } = query {
        check_positive(function, "precursor", precursor)?;
        check_positive(function, "product", product)?;
    }
    Ok(())
}

/// **GetTransitionMetadataOptions** · Options
///
/// Options for get_transition_metadata.
///
/// Fields:
/// - `tags`: Which tags to keep, by name. A row is kept when the last tag of its path matches, so precursor keeps chromatogram/precursor but not its isolationWindow. An empty list keeps every tag. The names are chromatogram, referenceableParamGroupRef, precursor, isolationWindow, selectedIonList, selectedIon, activation, product, binaryDataArrayList and binaryDataArray. Default `[]`.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#GetTransitionMetadataOptions)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GetTransitionMetadataOptions {
    pub tags: Vec<String>,
}

impl GetTransitionMetadataOptions {
    pub(crate) fn check(&self, function: &'static str) -> Result<(), QuantionError> {
        for tag in &self.tags {
            if !TRANSITION_METADATA_TAGS.contains(&tag.as_str()) {
                return Err(QuantionError::invalid(
                    function,
                    "tags",
                    format!(
                        "must be one of {}, got {tag:?}",
                        TRANSITION_METADATA_TAGS.join(", ")
                    ),
                ));
            }
        }
        Ok(())
    }

    fn keeps(&self, path: &str) -> bool {
        self.tags.is_empty() || self.tags.iter().any(|tag| tag == get_tag(path))
    }
}

/// **get_transition_metadata** · Targeted
///
/// Read the details stored for one chromatogram, such as an SRM or MRM transition (attributes, precursor, product, cvParams and userParams), one row each, parents before children. Find it by its index or by its precursor (Q1) and product (Q3), both within 0.005 m/z.
///
/// **Warning:** A sample opened from a URL gives a remote_file error; open it from a path or bytes.
///
/// - `file`: The sample to read, from parse_ion or parse_mzml.
/// - `query`: Which chromatogram: an index counted from 0, or a precursor (Q1) and product (Q3) pair. Each TransitionQuery has `Index`, `Transition`. See [TransitionQuery](https://phenological.github.io/quantion/?language=rust#TransitionQuery).
///   Example: `let query = TransitionQuery::Transition { precursor: 89.04768, product: 89.04768 };`
/// - `options`: Optional settings: `tags` = []. See [GetTransitionMetadataOptions](https://phenological.github.io/quantion/?language=rust#GetTransitionMetadataOptions).
///
/// Returns: One row per attribute or param, in the order of the mzML tree, with paths from chromatogram down. Empty when no stored chromatogram matches. A field the file does not give is missing (None in Rust and Python, null in JavaScript, NA in R).
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#get_transition_metadata)
pub fn get_transition_metadata(
    file: &mut SampleFile,
    query: TransitionQuery,
    options: GetTransitionMetadataOptions,
) -> Result<Vec<ScanParam>, QuantionError> {
    const FUNCTION: &str = "get_transition_metadata";
    check_transition_query(FUNCTION, query)?;
    options.check(FUNCTION)?;
    let rows = match &mut file.source {
        FileSource::Full(mzml) => get_chromatogram(mzml, query)
            .map(get_chromatogram_params)
            .unwrap_or_default(),
        FileSource::Lazy(ion) => find_ion_chromatogram(ion, query)
            .map_err(|error| read_error(FUNCTION, error))?
            .map(|(_, chrom)| get_chromatogram_params(&chrom))
            .unwrap_or_default(),
        FileSource::Remote(_) => return Err(QuantionError::remote_file(FUNCTION)),
    };
    Ok(rows
        .into_iter()
        .filter(|row| options.keeps(&row.path))
        .collect())
}
