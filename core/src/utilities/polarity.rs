pub(crate) const UNKNOWN_POLARITY: u8 = 0;
pub(crate) const POSITIVE_POLARITY: u8 = 1;
pub(crate) const NEGATIVE_POLARITY: u8 = 2;

/// **Polarity** · Enums
///
/// Ion mode of a scan, the same codes as the polarity of a scan: 1 for positive and 2 for negative.
///
/// Fields:
/// - `POSITIVE`: Positive ion mode, the code 1.
/// - `NEGATIVE`: Negative ion mode, the code 2.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#Polarity)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Polarity {
    POSITIVE,
    NEGATIVE,
}

impl Polarity {
    pub fn code(self) -> u8 {
        match self {
            Self::POSITIVE => POSITIVE_POLARITY,
            Self::NEGATIVE => NEGATIVE_POLARITY,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::POSITIVE => "positive",
            Self::NEGATIVE => "negative",
        }
    }

    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            POSITIVE_POLARITY => Some(Self::POSITIVE),
            NEGATIVE_POLARITY => Some(Self::NEGATIVE),
            _ => None,
        }
    }

    pub fn allows(self, polarity: u8) -> bool {
        polarity == UNKNOWN_POLARITY || polarity == self.code()
    }
}

pub(crate) fn get_polarity_name(polarity: u8) -> &'static str {
    match Polarity::from_code(polarity) {
        Some(found) => found.name(),
        None => "unknown",
    }
}

pub(crate) fn is_opposite_polarity(left: u8, right: u8) -> bool {
    left != UNKNOWN_POLARITY && right != UNKNOWN_POLARITY && left != right
}
