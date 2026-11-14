use crate::{
    PeakShape,
    error::{QuantionError, check_finite, check_not_negative, check_positive},
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
            format!(
                "must have the same length as x ({}), got {}",
                data.x.len(),
                data.y.len()
            ),
        ));
    }
    if data.x.len() < MIN_TRACE_POINTS {
        return Err(QuantionError::invalid(
            function,
            "x",
            format!(
                "must have at least {MIN_TRACE_POINTS} points, got {}",
                data.x.len()
            ),
        ));
    }
    Ok(())
}

/// **FindPeaksOptions** · Options
///
/// Options for find_peaks. Every field is optional.
///
/// Fields:
/// - `min_integral`: Drop peaks whose area (integral) is below this value. Off when not given. Default `None`.
/// - `min_intensity`: Drop peaks whose apex intensity is below this value. Raise it to ignore small peaks. Default `500.0`.
/// - `min_peak_width_points`: Drop peaks with fewer data points than this. Default `3`.
/// - `noise`: Fixed noise level (intensity), used only when auto_noise is false. Default `None`.
/// - `auto_noise`: Estimate the noise from the chromatogram. Turn off and set noise to use a fixed value. Default `true`.
/// - `auto_baseline`: Subtract an airPLS baseline before finding peaks. Turn off for windows shorter than 1 minute: they do not have enough points to estimate the baseline and can give misleading results. Default `true`.
/// - `lambda`: Smoothness of the airPLS baseline. A larger value gives a smoother baseline that follows only slow drift. Default `20.0`.
/// - `max_iterations`: Maximum number of airPLS iterations used to estimate the baseline. Default `4`.
/// - `allow_overlap`: Let neighbouring peaks share points at their border. When off, overlapping peaks are split at the valley between them. Default `false`.
/// - `min_snr`: Minimum signal-to-noise ratio. A peak’s apex must be above min_snr × noise. Default `2.0`.
/// - `min_r2`: Minimum fit quality (0–1) of the peak shape. Peaks that fit the shape worse are dropped. 0 keeps every peak. Default `0.0`.
/// - `shape`: Curve used to check the peak shape for min_r2: EMG or GAUSSIAN. Default `EMG`.
/// - `kernel_size`: Savitzky–Golay smoothing window, in points, to smooth noisy chromatograms. 0 = no smoothing. A larger value gives a smoother chromatogram. Default `0`.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#FindPeaksOptions)
#[derive(Clone, Debug, PartialEq)]
pub struct FindPeaksOptions {
    pub min_integral: Option<f64>,
    pub min_intensity: f64,
    pub min_peak_width_points: usize,
    pub noise: Option<f64>,
    pub auto_noise: bool,
    pub auto_baseline: bool,
    pub lambda: f64,
    pub max_iterations: usize,
    pub allow_overlap: bool,
    pub min_snr: f64,
    pub min_r2: f64,
    pub shape: PeakShape,
    pub kernel_size: usize,
}

impl Default for FindPeaksOptions {
    fn default() -> Self {
        Self {
            min_integral: None,
            min_intensity: 500.0,
            min_peak_width_points: 3,
            noise: None,
            auto_noise: true,
            auto_baseline: true,
            lambda: 20.0,
            max_iterations: 4,
            allow_overlap: false,
            min_snr: 2.0,
            min_r2: 0.0,
            shape: PeakShape::EMG,
            kernel_size: 0,
        }
    }
}

impl FindPeaksOptions {
    pub(crate) fn check(&self, function: &'static str) -> Result<(), QuantionError> {
        if let Some(min_integral) = self.min_integral {
            check_not_negative(function, "find_peaks.min_integral", min_integral)?;
        }
        check_not_negative(function, "find_peaks.min_intensity", self.min_intensity)?;
        if let Some(noise) = self.noise {
            check_not_negative(function, "find_peaks.noise", noise)?;
        }
        check_positive(function, "find_peaks.lambda", self.lambda)?;
        if self.max_iterations == 0 {
            return Err(QuantionError::invalid(
                function,
                "find_peaks.max_iterations",
                "must be greater than 0, got 0",
            ));
        }
        check_not_negative(function, "find_peaks.min_snr", self.min_snr)?;
        if !(0.0..=1.0).contains(&self.min_r2) {
            return Err(QuantionError::invalid(
                function,
                "find_peaks.min_r2",
                format!("must be between 0 and 1, got {}", self.min_r2),
            ));
        }
        Ok(())
    }
}

/// **find_peaks** · Traces
///
/// Find every peak (peak picking) in one trace or signal, such as a chromatogram (TIC, BPC or EIC). For one known peak at an rt, use get_peak. To calculate EIC use calculate_eic.
///
/// - `data.x`: Retention times in minutes, ascending, at least 3 points. For a chromatogram, its x.
/// - `data.y`: Intensities, same length as x. For a chromatogram, its y.
/// - `options`: Optional settings: `min_integral` = None, `min_intensity` = 500.0, `min_peak_width_points` = 3, `noise` = None, `auto_noise` = true, `auto_baseline` = true, `lambda` = 20.0, `max_iterations` = 4, `allow_overlap` = false, `min_snr` = 2.0, `min_r2` = 0.0, `shape` = EMG, `kernel_size` = 0. See [FindPeaksOptions](https://phenological.github.io/quantion/?language=rust#FindPeaksOptions).
///
/// Returns: Every peak that passes the filters specified in the function options: the peak table, one row per peak.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#find_peaks)
pub fn find_peaks(data: &DataXY, options: FindPeaksOptions) -> Result<Vec<Peak>, QuantionError> {
    const FUNCTION: &str = "find_peaks";
    check_trace(FUNCTION, data)?;
    options.check(FUNCTION)?;
    Ok(pick_peaks(data, &options))
}

/// **GetPeakOptions** · Options
///
/// Options for get_peak, including the search window and the peak-finding settings.
///
/// Fields:
/// - `range`: Half-width of the search window, in minutes. Only peaks within rt ± range are considered. 0 searches the whole trace. Default `0.0`.
/// - `find_peaks`: Options for find_peaks function. See [FindPeaksOptions](https://phenological.github.io/quantion/?language=rust#FindPeaksOptions).
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#GetPeakOptions)
#[derive(Clone, Debug, PartialEq, Default)]
pub struct GetPeakOptions {
    pub range: f64,
    pub find_peaks: FindPeaksOptions,
}

impl GetPeakOptions {
    pub(crate) fn check(&self, function: &'static str) -> Result<(), QuantionError> {
        check_not_negative(function, "range", self.range)?;
        self.find_peaks.check(function)
    }
}

/// **get_peak** · Traces
///
/// Find the peak closest to rt. If range is given, only peaks within rt ± range are searched. If it is not, the whole trace is searched, and the peak is chosen by its size and its distance from rt.
///
/// - `data.x`: Retention times, ascending. At least 3 points.
/// - `data.y`: Intensities, same length as x.
/// - `rt`: Expected retention time of the target peak, in minutes.
/// - `options`: Optional settings: `range` = 0.0, `find_peaks`. See [GetPeakOptions](https://phenological.github.io/quantion/?language=rust#GetPeakOptions).
///
/// Returns: The chosen peak, or an empty peak (every field is 0) when no peak is found.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#get_peak)
pub fn get_peak(data: &DataXY, rt: f64, options: GetPeakOptions) -> Result<Peak, QuantionError> {
    const FUNCTION: &str = "get_peak";
    check_trace(FUNCTION, data)?;
    check_finite(FUNCTION, "rt", rt)?;
    options.check(FUNCTION)?;
    let roi = Roi {
        rt,
        range: options.range,
    };
    Ok(pick_peak(data, &roi, &options.find_peaks))
}

/// **FitPeakOptions** · Options
///
/// Options for fit_peak function.
///
/// Fields:
/// - `shape`: Curve to fit: EMG (with a right tail) or GAUSSIAN (symmetric). Default `EMG`.
/// - `intensity`: Apex intensity to start the fit from. 0 = use the apex of the peak found at rt. Default `0.0`.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#FitPeakOptions)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FitPeakOptions {
    pub shape: PeakShape,
    pub intensity: f64,
}

impl Default for FitPeakOptions {
    fn default() -> Self {
        Self {
            shape: PeakShape::EMG,
            intensity: 0.0,
        }
    }
}

impl FitPeakOptions {
    pub(crate) fn check(&self, function: &'static str) -> Result<(), QuantionError> {
        check_not_negative(function, "intensity", self.intensity)
    }
}

/// **fit_peak** · Traces
///
/// Fit a Gaussian or EMG curve to the target peak at rt. The function finds the peak within rt ± 0.05 min, then fits the points between its start and end to get its width (fwhm) and tailing (tail). The peak needs at least 5 points.
///
/// - `data.x`: Retention times, ascending. At least 3 points.
/// - `data.y`: Intensities, same length as x.
/// - `rt`: Retention time of the peak to fit, in minutes. The peak must be within ± 0.05 min of it.
/// - `options`: Optional settings: `shape` = EMG, `intensity` = 0.0. See [FitPeakOptions](https://phenological.github.io/quantion/?language=rust#FitPeakOptions).
///
/// Returns: The fitted parameters (height, center, fwhm, tail, r2) for the target peak, or none if no peak is found near rt or the fit fails.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#fit_peak)
pub fn fit_peak(
    data: &DataXY,
    rt: f64,
    options: FitPeakOptions,
) -> Result<Option<PeakParameters>, QuantionError> {
    const FUNCTION: &str = "fit_peak";
    check_trace(FUNCTION, data)?;
    check_finite(FUNCTION, "rt", rt)?;
    options.check(FUNCTION)?;
    let roi = Roi {
        rt,
        range: FIT_PEAK_RANGE,
    };
    let peak = pick_peak(data, &roi, &FindPeaksOptions::default());
    if peak.intensity <= 0.0 || peak.to <= peak.from {
        return Ok(None);
    }
    let intensity = if options.intensity > 0.0 {
        options.intensity
    } else {
        peak.intensity
    };
    let seed = PeakSeed {
        rt: peak.rt,
        intensity,
    };
    Ok(fit_curve(
        &get_section(data, peak.from, peak.to),
        &seed,
        options.shape,
    ))
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

/// **DrawPeaksOptions** · Options
///
/// Options for draw_peaks, including the time range and the number of points.
///
/// Fields:
/// - `from`: Start of the simulated chromatogram, in minutes. Leave from and to at 0 to cover all the peaks automatically. Default `0.0`.
/// - `to`: End of the simulated chromatogram, in minutes. Default `0.0`.
/// - `points`: Number of points drawn from from to to. Default `1000`.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#DrawPeaksOptions)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawPeaksOptions {
    pub from: f64,
    pub to: f64,
    pub points: usize,
}

impl Default for DrawPeaksOptions {
    fn default() -> Self {
        Self {
            from: 0.0,
            to: 0.0,
            points: 1000,
        }
    }
}

impl DrawPeaksOptions {
    pub(crate) fn check(&self, function: &'static str) -> Result<(), QuantionError> {
        check_finite(function, "from", self.from)?;
        check_finite(function, "to", self.to)?;
        let fit_range = self.from == 0.0 && self.to == 0.0;
        if !fit_range && self.to <= self.from {
            return Err(QuantionError::invalid(
                function,
                "to",
                format!("must be greater than from ({}), got {}", self.from, self.to),
            ));
        }
        if self.points == 0 {
            return Err(QuantionError::invalid(
                function,
                "points",
                "must be greater than 0, got 0",
            ));
        }
        Ok(())
    }
}

/// **draw_peaks** · Traces
///
/// Simulate a chromatogram (a synthetic trace) from peak curves (height, center, fwhm, tail). Overlapping peaks add up. Useful for tests, no data needed.
///
/// - `params`: The peaks to draw, one PeakParameters per peak. Each PeakParameters has `height`, `center`, `fwhm`, `tail`, `r2`. See [PeakParameters](https://phenological.github.io/quantion/?language=rust#PeakParameters).
///   Example: `let peak = PeakParameters { height: 9800.0, center: 2.38, fwhm: 0.021, tail: 0.006, r2: 0.0 };`
/// - `options`: Optional settings: `from` = 0.0, `to` = 0.0, `points` = 1000. See [DrawPeaksOptions](https://phenological.github.io/quantion/?language=rust#DrawPeaksOptions).
///
/// Returns: The simulated chromatogram: x = retention times in minutes, y = summed intensities.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#draw_peaks)
pub fn draw_peaks(
    params: &[PeakParameters],
    options: DrawPeaksOptions,
) -> Result<DataXY, QuantionError> {
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
    Ok(draw_curves(
        params,
        options.from,
        options.to,
        options.points,
    ))
}

/// **find_noise_level** · Traces
///
/// Estimate the noise level of a chromatogram or any signal: how noisy it is. Small, narrow peaks are grouped together, and their maximum intensity and width (in number of points) are reported as the noise.
///
/// - `y`: Intensities, for example the y of a chromatogram.
///
/// Returns: The noise intensity and the width (in points) of the noise regions it was estimated from.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#find_noise_level)
pub fn find_noise_level(y: &[f64]) -> NoiseLevel {
    read_noise_level(y)
}

/// **CalculateBaselineOptions** · Options
///
/// Options for calculate_baseline, which estimates the baseline with airPLS.
///
/// Fields:
/// - `lambda`: Smoothness of the baseline. A larger value gives a smoother baseline that follows only slow drift. Default `20.0`.
/// - `max_iterations`: Maximum number of iterations used to estimate the baseline. Default `4`.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#CalculateBaselineOptions)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalculateBaselineOptions {
    pub lambda: f64,
    pub max_iterations: usize,
}

impl Default for CalculateBaselineOptions {
    fn default() -> Self {
        Self {
            lambda: 20.0,
            max_iterations: 4,
        }
    }
}

impl CalculateBaselineOptions {
    pub(crate) fn check(&self, function: &'static str) -> Result<(), QuantionError> {
        check_positive(function, "lambda", self.lambda)?;
        if self.max_iterations == 0 {
            return Err(QuantionError::invalid(
                function,
                "max_iterations",
                "must be greater than 0, got 0",
            ));
        }
        Ok(())
    }
}

/// **calculate_baseline** · Traces
///
/// Estimate the baseline (slow background drift) of the input chromatogram or signal using airPLS (Zhang et al., 2010).
///
/// - `y`: Intensities, for example the y of a chromatogram.
/// - `options`: Optional settings: `lambda` = 20.0, `max_iterations` = 4. See [CalculateBaselineOptions](https://phenological.github.io/quantion/?language=rust#CalculateBaselineOptions).
///
/// Returns: The baseline: one value per point of y.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#calculate_baseline)
pub fn calculate_baseline(
    y: &[f64],
    options: CalculateBaselineOptions,
) -> Result<Vec<f64>, QuantionError> {
    const FUNCTION: &str = "calculate_baseline";
    options.check(FUNCTION)?;
    Ok(read_baseline(y, options, STANDALONE_EDGE_SLOPE_LEVEL))
}
