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

pub fn find_peaks(data: &DataXY, options: FindPeaksOptions) -> Result<Vec<Peak>, QuantionError> {
    const FUNCTION: &str = "find_peaks";
    check_trace(FUNCTION, data)?;
    options.check(FUNCTION)?;
    Ok(pick_peaks(data, &options))
}

pub fn get_peak(data: &DataXY, rt: f64, options: GetPeakOptions) -> Result<Peak, QuantionError> {
    const FUNCTION: &str = "get_peak";
    check_trace(FUNCTION, data)?;
    check_finite(FUNCTION, "rt", rt)?;
    options.check(FUNCTION)?;
    let roi = Roi { rt, range: options.range };
    Ok(pick_peak(data, &roi, &options.find_peaks))
}

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

pub fn find_noise_level(y: &[f64]) -> NoiseLevel {
    read_noise_level(y)
}

pub fn calculate_baseline(y: &[f64], options: CalculateBaselineOptions) -> Result<Vec<f64>, QuantionError> {
    const FUNCTION: &str = "calculate_baseline";
    options.check(FUNCTION)?;
    Ok(read_baseline(y, options, STANDALONE_EDGE_SLOPE_LEVEL))
}
