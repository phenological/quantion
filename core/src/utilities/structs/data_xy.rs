use serde::Deserialize;

/// **DataXY** · Results
///
/// A chromatogram (trace), with x as the retention times and y as the intensities. calculate_eic returns it, and find_peaks and get_peak take it.
///
/// Fields:
/// - `x`: Retention times, in minutes.
/// - `y`: Intensities, one per retention time.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#DataXY)
#[derive(Clone, Debug, Default, Deserialize)]
pub struct DataXY {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

impl DataXY {
    pub fn empty() -> DataXY {
        Self {
            x: Vec::new(),
            y: Vec::new(),
        }
    }
}
