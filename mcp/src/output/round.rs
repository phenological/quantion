#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Rt,
    Mz,
    Intensity,
    Ratio,
    Count,
    Text,
}

pub fn round_value(value: f64, unit: Unit) -> f64 {
    match unit {
        Unit::Rt => round_to_decimals(value, 4),
        Unit::Mz => round_to_decimals(value, 5),
        Unit::Intensity => round_to_significant(value, 4),
        Unit::Ratio => round_to_decimals(value, 3),
        Unit::Count | Unit::Text => value,
    }
}

pub fn round_to_decimals(value: f64, decimals: i32) -> f64 {
    if !value.is_finite() {
        return value;
    }
    let scale = 10_f64.powi(decimals);
    (value * scale).round() / scale
}

pub fn round_to_significant(value: f64, digits: i32) -> f64 {
    if !value.is_finite() || value == 0.0 {
        return value;
    }
    let magnitude = value.abs().log10().floor() as i32;
    let decimals = digits - 1 - magnitude;
    if decimals >= 0 {
        round_to_decimals(value, decimals)
    } else {
        let scale = 10_f64.powi(-decimals);
        (value / scale).round() * scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_by_unit() {
        assert_eq!(round_value(2.383_456_7, Unit::Rt), 2.3835);
        assert_eq!(round_value(195.087_654_3, Unit::Mz), 195.08765);
        assert_eq!(round_value(1_234_567.0, Unit::Intensity), 1_235_000.0);
        assert_eq!(round_value(0.012_345_6, Unit::Intensity), 0.01235);
        assert_eq!(round_value(0.0, Unit::Intensity), 0.0);
    }
}
