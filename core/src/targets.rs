use crate::{
    error::{QuantionError, check_finite, check_not_negative, check_positive},
    files::from_fast_error,
    options::{FindTargetsOptions, FindTransitionsOptions, check_threads},
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
pub fn find_targets(file: &mut SampleFile, targets: &[Target], options: FindTargetsOptions) -> Result<Vec<Peak>, QuantionError> {
    const FUNCTION: &str = "find_targets";
    options.eic.check(FUNCTION, "eic.")?;
    options.find_peaks.check(FUNCTION)?;
    check_threads(FUNCTION, options.threads)?;
    for (index, target) in targets.iter().enumerate() {
        check_positive(FUNCTION, &format!("targets[{index}].mz"), target.mz)?;
        check_finite(FUNCTION, &format!("targets[{index}].rt"), target.rt)?;
        if let Some(range) = target.range {
            check_not_negative(FUNCTION, &format!("targets[{index}].range"), range)?;
        }
    }
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
    read_targets(&mut file.reader(), &targets_in_minutes, options.eic.get_window(), settings)
        .map_err(|error| from_fast_error(FUNCTION, error))
}

/// **find_transitions** · Targeted
///
/// For each SRM transition, take the stored chromatogram whose precursor (Q1) and product (Q3) are both within 0.005 m/z, and find its peak near rt ± range.
///
/// **Warning:** Needs the whole file. A sample opened from a URL gives a remote_file error; open it from a path or bytes.
///
/// - `file`: The sample to read, from parse_ion or parse_mzml.
/// - `targets`: The transitions. Each Transition has `id`, `precursor`, `product`, `rt`, `range`. See [Transition](https://phenological.github.io/quantion/?language=rust#Transition).
///   Example: `let transition = Transition { id: Some("compound_1".into()), precursor: 500.3, product: 184.1, rt: 3.2, range: 0.4 };`
/// - `options`: Optional settings: `find_peaks`, `threads` = 1. See [FindTransitionsOptions](https://phenological.github.io/quantion/?language=rust#FindTransitionsOptions).
///
/// Returns: One Peak per transition, in the same order. No match gives a Peak with every field 0.
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
        check_positive(FUNCTION, &format!("targets[{index}].precursor"), target.precursor)?;
        check_positive(FUNCTION, &format!("targets[{index}].product"), target.product)?;
        check_finite(FUNCTION, &format!("targets[{index}].rt"), target.rt)?;
        check_positive(FUNCTION, &format!("targets[{index}].range"), target.range)?;
    }
    file.with_mzml(FUNCTION, |mzml| {
        Ok(read_transitions(mzml, targets, options.clone()).unwrap_or_else(|| vec![Peak::default(); targets.len()]))
    })
}
