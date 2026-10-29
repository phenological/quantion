use crate::{
    error::{QuantionError, check_finite},
    options::{CalculateBaselineOptions, DrawPeaksOptions, FindPeaksOptions, FitPeakOptions, GetPeakOptions},
    utilities::{
        calculate_baseline::{STANDALONE_EDGE_SLOPE_LEVEL, calculate_baseline as read_baseline},
        find_noise_level::{NoiseLevel, find_noise_level as read_noise_level},
        find_peaks::find_peaks as pick_peaks,
        fit_peak::{PeakParameters, PeakSeed, draw_peaks as draw_curves, fit_peak as fit_curve},
        get_peak::get_peak as pick_peak,
        structs::{DataXY, Peak, Roi},
    },
};

const MIN_TRACE_POINTS: usize = 3;
const FIT_PEAK_RANGE: f64 = 0.05;

pub(crate) fn check_trace(function: &'static str, data: &DataXY) -> Result<(), QuantionError> {
    if data.x.len() != data.y.len() {
        return Err(QuantionError::invalid(
            function,
            "y",
            format!("must have the same length as x ({}), got {}", data.x.len(), data.y.len()),
        ));
    }
    if data.x.len() < MIN_TRACE_POINTS {
        return Err(QuantionError::invalid(
            function,
            "x",
            format!("must have at least {MIN_TRACE_POINTS} points, got {}", data.x.len()),
        ));
    }
    Ok(())
}

/// **find_peaks** · Traces
///
/// Find every peak in one trace.
///
/// - `data.x`: Retention times, ascending. At least 3 points.
/// - `data.y`: Intensities, same length as x.
/// - `options`: Optional settings. Every field and default is listed in FindPeaksOptions.
///
/// Returns: Every peak that passes the filters.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#find_peaks)
pub fn find_peaks(data: &DataXY, options: FindPeaksOptions) -> Result<Vec<Peak>, QuantionError> {
    const FUNCTION: &str = "find_peaks";
    check_trace(FUNCTION, data)?;
    options.check(FUNCTION)?;
    Ok(pick_peaks(data, &options))
}

/// **get_peak** · Traces
///
/// Find the one peak near rt. With a range, only rt ± range is searched; without one, the whole trace is searched and the peak is chosen by its size and its distance to rt.
///
/// - `data.x`: Retention times, ascending. At least 3 points.
/// - `data.y`: Intensities, same length as x.
/// - `rt`: Expected retention time of the peak.
/// - `options`: Optional settings. Every field and default is listed in GetPeakOptions.
///
/// Returns: The best peak near rt. No peak gives a Peak with every field 0.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#get_peak)
pub fn get_peak(data: &DataXY, rt: f64, options: GetPeakOptions) -> Result<Peak, QuantionError> {
    const FUNCTION: &str = "get_peak";
    check_trace(FUNCTION, data)?;
    check_finite(FUNCTION, "rt", rt)?;
    options.check(FUNCTION)?;
    let roi = Roi { rt, range: options.range };
    Ok(pick_peak(data, &roi, &options.find_peaks))
}

/// **fit_peak** · Traces
///
/// Fit a Gaussian or EMG curve to the peak near rt. rt says which peak; its apex intensity is the starting point unless you force one.
///
/// - `data.x`: Retention times, ascending. At least 3 points.
/// - `data.y`: Intensities, same length as x.
/// - `rt`: Expected retention time of the peak.
/// - `options`: Optional settings. Every field and default is listed in FitPeakOptions.
///
/// Returns: The fitted curve, or none when no peak is found or the fit fails.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#fit_peak)
pub fn fit_peak(data: &DataXY, rt: f64, options: FitPeakOptions) -> Result<Option<PeakParameters>, QuantionError> {
    const FUNCTION: &str = "fit_peak";
    check_trace(FUNCTION, data)?;
    check_finite(FUNCTION, "rt", rt)?;
    options.check(FUNCTION)?;
    let roi = Roi { rt, range: FIT_PEAK_RANGE };
    let peak = pick_peak(data, &roi, &FindPeaksOptions::default());
    if peak.intensity <= 0.0 || peak.to <= peak.from {
        return Ok(None);
    }
    let intensity = if options.intensity > 0.0 { options.intensity } else { peak.intensity };
    let seed = PeakSeed { rt: peak.rt, intensity };
    Ok(fit_curve(&get_section(data, peak.from, peak.to), &seed, options.shape))
}

fn get_section(data: &DataXY, from: f64, to: f64) -> DataXY {
    let (x, y) = data
        .x
        .iter()
        .zip(&data.y)
        .filter(|(rt, _)| **rt >= from && **rt <= to)
        .map(|(rt, value)| (*rt, *value))
        .unzip();
    DataXY { x, y }
}

/// **draw_peaks** · Traces
///
/// Simulate a chromatogram from one or more peak shapes. Every peak is drawn into the same chromatogram; where they overlap, intensities add up. No real data is needed.
///
/// - `params`: The peaks to draw, one PeakParameters per peak.
/// - `options`: Optional settings. Every field and default is listed in DrawPeaksOptions.
///
/// Returns: The simulated chromatogram: x = retention times, y = summed intensities.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#draw_peaks)
pub fn draw_peaks(params: &[PeakParameters], options: DrawPeaksOptions) -> Result<DataXY, QuantionError> {
    const FUNCTION: &str = "draw_peaks";
    options.check(FUNCTION)?;
    for (index, peak) in params.iter().enumerate() {
        check_finite(FUNCTION, &format!("params[{index}].center"), peak.center)?;
        if !(peak.fwhm.is_finite() && peak.fwhm > 0.0) {
            return Err(QuantionError::invalid(
                FUNCTION,
                format!("params[{index}].fwhm"),
                format!("must be greater than 0, got {}", peak.fwhm),
            ));
        }
        if !(peak.tail.is_finite() && peak.tail >= 0.0) {
            return Err(QuantionError::invalid(
                FUNCTION,
                format!("params[{index}].tail"),
                format!("must be 0 or more, got {}", peak.tail),
            ));
        }
    }
    Ok(draw_curves(params, options.from, options.to, options.points))
}

/// **find_noise_level** · Traces
///
/// Estimate the noise level of a signal.
///
/// - `y`: Intensities.
///
/// Returns: The noise intensity and the window width it came from.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#find_noise_level)
pub fn find_noise_level(y: &[f64]) -> NoiseLevel {
    read_noise_level(y)
}

/// **calculate_baseline** · Traces
///
/// Estimate a signal's baseline with airPLS.
///
/// - `y`: Intensities.
/// - `options`: Optional settings. Every field and default is listed in CalculateBaselineOptions.
///
/// Returns: The baseline, same length as y.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#calculate_baseline)
pub fn calculate_baseline(y: &[f64], options: CalculateBaselineOptions) -> Result<Vec<f64>, QuantionError> {
    const FUNCTION: &str = "calculate_baseline";
    options.check(FUNCTION)?;
    Ok(read_baseline(y, options, STANDALONE_EDGE_SLOPE_LEVEL))
}
