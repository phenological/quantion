/// **TransitionQuery** · Inputs
///
/// Which stored chromatogram get_transition and get_transition_metadata read: one index, or one precursor (Q1) and product (Q3) pair.
///
/// Fields:
/// - `Index`: The chromatogram at this position in the file, counted from 0.
/// - `Transition`: The chromatogram whose precursor (Q1) and product (Q3) are both within 0.005 m/z.
///
/// Example:
/// ```ignore
/// let query = TransitionQuery::Transition { precursor: 89.04768, product: 89.04768 };
/// ```
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#TransitionQuery)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TransitionQuery {
    Index(usize),
    Transition { precursor: f64, product: f64 },
}
