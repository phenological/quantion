/// **ChromatogramQuery** · Inputs
///
/// In Rust, which stored chromatogram get_chromatogram reads. Python, JavaScript and R use an index number or a {precursor, product} pair instead.
///
/// Fields:
/// - `Index(usize)`: The chromatogram at this position in the file, counted from 0.
/// - `Transition { precursor, product }`: The chromatogram whose precursor (Q1) and product (Q3) are both within 0.005 m/z.
///
/// Example:
/// ```ignore
/// let query = ChromatogramQuery::Transition { precursor: 240.1, product: 112.0 };
/// ```
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#ChromatogramQuery)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChromatogramQuery {
    Index(usize),
    Transition { precursor: f64, product: f64 },
}
