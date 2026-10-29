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
