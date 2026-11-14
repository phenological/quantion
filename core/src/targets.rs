use crate::{
    CalculateEicOptions, FindPeaksOptions,
    error::{
        QuantionError, check_finite, check_mz_and_rt, check_not_negative, check_positive,
        check_threads,
    },
    files::from_fast_error,
    sample_file::SampleFile,
    utilities::{
        find_targets::find_targets as read_targets,
        find_transitions::find_transitions as read_transitions,
        structs::{Peak, Target, Transition},
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
