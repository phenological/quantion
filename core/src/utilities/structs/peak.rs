use serde::Deserialize;

/// **Peak** · Results
///
/// One chromatographic peak, or an empty peak (every field is 0) when no peak is found.
///
/// Fields:
/// - `rt`: Apex retention time, in minutes.
/// - `from`: Start of the peak, in minutes.
/// - `to`: End of the peak, in minutes.
/// - `intensity`: Apex intensity: the peak height.
/// - `integral`: Area (integral) of the peak, calculated with the trapezoid rule on the chromatogram trace between from and to.
/// - `n_points`: Number of data points in the peak, from its start to its end.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#Peak)
#[derive(Clone, Copy, Deserialize)]
pub struct Peak {
    pub from: f64,
    pub to: f64,
    pub rt: f64,
    pub integral: f64,
    pub intensity: f64,
    pub n_points: usize,
    pub(crate) noise: f64,
    pub(crate) r2: f64,
}

impl std::fmt::Debug for Peak {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Peak")
            .field("from", &self.from)
            .field("to", &self.to)
            .field("rt", &self.rt)
            .field("integral", &self.integral)
            .field("intensity", &self.intensity)
            .field("n_points", &self.n_points)
            .finish()
    }
}

impl Default for Peak {
    fn default() -> Self {
        Self {
            from: 0.0,
            to: 0.0,
            rt: 0.0,
            integral: 0.0,
            intensity: 0.0,
            r2: 0.0,
            n_points: 0,
            noise: 0.0,
        }
    }
}
