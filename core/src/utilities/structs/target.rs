/// **Target** · Inputs
///
/// One compound to look for, with its m/z and expected retention time. range limits the search in find_targets, and a missing or 0 range searches the whole EIC. predict_rts and calibrate do not use range.
///
/// Fields:
/// - `id`: Your name for the compound. It is not used in the search. Optional.
/// - `mz`: The m/z of the ion to look for (Da), not the neutral mass of the compound. For example, for the protonated molecule \[M+H\]+ in positive mode, use M + 1.00728, and for the deprotonated molecule \[M−H\]− in negative mode, use M − 1.00728, where M is the monoisotopic mass.
/// - `rt`: Expected retention time, in minutes by default (or in the time unit set in the EIC options).
/// - `range`: Half-width of the search window around rt, in the same unit as rt. Missing or 0 searches the whole EIC. Optional.
///
/// Example:
/// ```ignore
/// let target = Target { id: Some("compound_1".into()), mz: 260.103, rt: 2.383, range: Some(0.05) };
/// ```
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#Target)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Target {
    pub id: Option<String>,
    pub mz: f64,
    pub rt: f64,
    pub range: Option<f64>,
}

impl Target {
    pub(crate) fn get_id(&self) -> &str {
        self.id.as_deref().unwrap_or("")
    }

    pub(crate) fn get_range(&self) -> f64 {
        self.range.unwrap_or(0.0)
    }
}
