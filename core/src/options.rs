use std::path::PathBuf;

use crate::{
    error::{QuantionError, check_finite, check_not_negative, check_positive},
    utilities::{
        calculate_eic::TimeUnit,
        fit_peak::PeakShape,
        structs::FromTo,
    },
};

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
            return Err(QuantionError::invalid(function, "find_peaks.max_iterations", "must be greater than 0, got 0"));
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
            return Err(QuantionError::invalid(function, "points", "must be greater than 0, got 0"));
        }
        Ok(())
    }
}

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
            return Err(QuantionError::invalid(function, "max_iterations", "must be greater than 0, got 0"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalculateEicOptions {
    pub from: f64,
    pub to: f64,
    pub ppm_tolerance: f64,
    pub mz_tolerance: f64,
    pub time_unit: TimeUnit,
}

impl Default for CalculateEicOptions {
    fn default() -> Self {
        Self {
            from: 0.0,
            to: f64::INFINITY,
            ppm_tolerance: 20.0,
            mz_tolerance: 0.005,
            time_unit: TimeUnit::MINUTES,
        }
    }
}

impl CalculateEicOptions {
    pub(crate) fn check(&self, function: &'static str, prefix: &str) -> Result<(), QuantionError> {
        check_finite(function, &format!("{prefix}from"), self.from)?;
        if self.to.is_nan() || self.to <= self.from {
            return Err(QuantionError::invalid(
                function,
                format!("{prefix}to"),
                format!("must be greater than from ({}), got {}", self.from, self.to),
            ));
        }
        check_not_negative(function, &format!("{prefix}ppm_tolerance"), self.ppm_tolerance)?;
        check_not_negative(function, &format!("{prefix}mz_tolerance"), self.mz_tolerance)?;
        if self.ppm_tolerance == 0.0 && self.mz_tolerance == 0.0 {
            return Err(QuantionError::invalid(
                function,
                format!("{prefix}ppm_tolerance"),
                "and mz_tolerance cannot both be 0",
            ));
        }
        Ok(())
    }

    pub(crate) fn to_minutes(&self, value: f64) -> f64 {
        self.time_unit.to_minutes(value)
    }

    pub(crate) fn get_window(&self) -> FromTo {
        FromTo {
            from: self.to_minutes(self.from),
            to: self.to_minutes(self.to),
        }
    }

    pub(crate) fn in_minutes(self) -> CalculateEicOptions {
        let window = self.get_window();
        CalculateEicOptions {
            from: window.from,
            to: window.to,
            time_unit: TimeUnit::MINUTES,
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridOptions {
    pub start: f64,
    pub end: f64,
    pub step_size: f64,
}

impl Default for GridOptions {
    fn default() -> Self {
        Self {
            start: 40.0,
            end: 1000.0,
            step_size: 0.005,
        }
    }
}

impl GridOptions {
    pub(crate) fn check(&self, function: &'static str) -> Result<(), QuantionError> {
        check_positive(function, "grid.start", self.start)?;
        if !self.end.is_finite() || self.end <= self.start {
            return Err(QuantionError::invalid(
                function,
                "grid.end",
                format!("must be greater than grid.start ({}), got {}", self.start, self.end),
            ));
        }
        check_positive(function, "grid.step_size", self.step_size)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroupingOptions {
    pub ppm_tolerance: f64,
    pub mz_tolerance: f64,
    pub rt_tolerance: f64,
    pub min_frequency: f64,
}

impl Default for GroupingOptions {
    fn default() -> Self {
        Self {
            ppm_tolerance: 5.0,
            mz_tolerance: 0.0025,
            rt_tolerance: 0.05,
            min_frequency: 0.0,
        }
    }
}

impl GroupingOptions {
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    pub(crate) fn check(&self, function: &'static str) -> Result<(), QuantionError> {
        check_not_negative(function, "grouping.ppm_tolerance", self.ppm_tolerance)?;
        check_not_negative(function, "grouping.mz_tolerance", self.mz_tolerance)?;
        check_positive(function, "grouping.rt_tolerance", self.rt_tolerance)?;
        if !(0.0..=1.0).contains(&self.min_frequency) {
            return Err(QuantionError::invalid(
                function,
                "grouping.min_frequency",
                format!("must be between 0 and 1, got {}", self.min_frequency),
            ));
        }
        Ok(())
    }
}

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

#[derive(Clone, Debug, PartialEq)]
pub struct FindFeaturesOptions {
    pub eic: CalculateEicOptions,
    pub grid: GridOptions,
    pub find_peaks: FindPeaksOptions,
    pub threads: usize,
}

impl Default for FindFeaturesOptions {
    fn default() -> Self {
        Self {
            eic: CalculateEicOptions::default(),
            grid: GridOptions::default(),
            find_peaks: FindPeaksOptions::default(),
            threads: 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchFeaturesOptions {
    pub eic: CalculateEicOptions,
    pub grid: GridOptions,
    pub grouping: GroupingOptions,
    pub find_peaks: FindPeaksOptions,
    pub threads: usize,
}

impl Default for MatchFeaturesOptions {
    fn default() -> Self {
        Self {
            eic: CalculateEicOptions::default(),
            grid: GridOptions::default(),
            grouping: GroupingOptions::default(),
            find_peaks: FindPeaksOptions::default(),
            threads: 1,
        }
    }
}

pub(crate) fn check_threads(function: &'static str, threads: usize) -> Result<(), QuantionError> {
    if threads == 0 {
        return Err(QuantionError::invalid(function, "threads", "must be 1 or more, got 0"));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GetScansOptions {
    pub level: u8,
    pub time_unit: TimeUnit,
}

impl Default for GetScansOptions {
    fn default() -> Self {
        Self {
            level: 1,
            time_unit: TimeUnit::MINUTES,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct ParseIonOptions {
    pub max_cache_size: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum IonOutput {
    Path(PathBuf),
    Url(String),
}

impl From<&str> for IonOutput {
    fn from(text: &str) -> Self {
        if crate::utilities::parse_ion::is_url(text) {
            IonOutput::Url(text.to_string())
        } else {
            IonOutput::Path(PathBuf::from(text))
        }
    }
}

impl From<String> for IonOutput {
    fn from(text: String) -> Self {
        IonOutput::from(text.as_str())
    }
}

impl From<PathBuf> for IonOutput {
    fn from(path: PathBuf) -> Self {
        IonOutput::Path(path)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WriteIonOptions {
    pub out: Option<IonOutput>,
    pub overwrite: bool,
    pub level: u8,
    pub f32_compress: bool,
    pub headers: Vec<Header>,
}

impl Default for WriteIonOptions {
    fn default() -> Self {
        Self {
            out: None,
            overwrite: false,
            level: 12,
            f32_compress: false,
            headers: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct WriteMzmlOptions {
    pub out: Option<PathBuf>,
    pub overwrite: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct GetIonImageOptions {
    pub tolerance: f64,
    pub level: u8,
    pub on_progress: Option<fn(u64, u64, u64)>,
}

impl Default for GetIonImageOptions {
    fn default() -> Self {
        Self {
            tolerance: 0.125,
            level: 1,
            on_progress: None,
        }
    }
}
