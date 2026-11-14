use crate::utilities::structs::FromTo;

/// **ScanQuery** · Inputs
///
/// Which scans get_scans returns, by retention time, by precursor m/z, or by both. Ranges give every scan inside them, and one value gives the closest scan.
///
/// Fields:
/// - `rt`: Retention time, as a range, as one value for the closest scan, or nothing.
/// - `mz`: Precursor m/z, as a range, as one value for the closest scan, or nothing.
///
/// Example:
/// ```ignore
/// let query = ScanQuery { rt: Position::Range(FromTo { from: 2.3, to: 2.4 }), mz: Position::Closest(300.0) };
/// ```
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#ScanQuery)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScanQuery {
    pub rt: Position,
    pub mz: Position,
}

/// **Position** · Inputs
///
/// In Rust, where rt or m/z must be in a ScanQuery. Python, JavaScript and R use a number, a {from, to} range, or nothing instead.
///
/// Fields:
/// - `Any`: No filter.
/// - `Range(FromTo)`: Every scan whose value is in the range.
/// - `Closest(f64)`: Only the scan whose value is closest to this one.
///
/// Example:
/// ```ignore
/// let window = Position::Range(FromTo { from: 2.3, to: 2.4 });
/// ```
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#Position)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Position {
    #[default]
    Any,
    Range(FromTo),
    Closest(f64),
}
