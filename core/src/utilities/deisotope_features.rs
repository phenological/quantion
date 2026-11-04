use crate::Feature;

/// Mass of 13C minus 12C (Da), from https://en.wikipedia.org/wiki/Carbon-13
const ISOTOPE_DIFF: f64 = 1.003354835;

/// 13C abundance / 12C abundance = 1.07 / 98.93
const CARBON_13_RATIO: f64 = 0.0108157;

/// **DeisotopeFeaturesOptions** · Options
///
/// How deisotope_features links isotope peaks.
///
/// Fields:
/// - `rt_tolerance`: Same compound when rt is within this many minutes of M. Default `0.02`.
/// - `mz_tolerance`: Isotope peak when m/z is within this many Da of the expected position. Default `0.005`.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#DeisotopeFeaturesOptions)
pub struct DeisotopeFeaturesOptions {
    pub rt_tolerance: f64,
    pub mz_tolerance: f64
}

impl Default for DeisotopeFeaturesOptions {
    fn default() -> Self {
        Self {
            rt_tolerance: 0.02,
            mz_tolerance: 0.005
        }
    }
}


/// **deisotope_features** · Untargeted
///
/// Remove 13C isotope peaks from a feature list, keeping the monoisotopic peak (M) of each compound. A feature is an isotope when it sits 1.00335/charge above the previous peak (charge = 1 or 2), has the same rt as M, and its intensity is not too high: M+1 at most `M × mz × charge / 12 × 0.0108157`.
///
/// - `features`: The features to clean, from find_features or match_features.
/// - `options`: Optional settings: `rt_tolerance` = 0.02, `mz_tolerance` = 0.005. See [DeisotopeFeaturesOptions](https://phenological.github.io/quantion/?language=rust#DeisotopeFeaturesOptions).
///
/// Returns: The features that are not isotopes, sorted by m/z.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#deisotope_features)
pub fn deisotope_features(mut features: Vec<Feature>, options: DeisotopeFeaturesOptions) -> Vec<Feature> {
    let DeisotopeFeaturesOptions {rt_tolerance, mz_tolerance} = options;
    features.sort_unstable_by(|a, b| a.mz.total_cmp(&b.mz));
    let mut is_isotope: Vec<bool> = vec![false; features.len()];
    for idx in 0..features.len() {
        if is_isotope[idx] {
            continue;
        }
        for charge in 1..=2 {
            let mut current = idx;
            loop {
                let target = features[current].mz + ISOTOPE_DIFF / charge as f64;
                let start = features.partition_point(|feature| feature.mz < target - mz_tolerance);
                let mut best: Option<usize> = None;
                for j in start..features.len() {
                    if features[j].mz > target + mz_tolerance {
                        break;
                    } else if is_isotope[j] {
                        continue;
                    } else if (features[j].rt - features[idx].rt).abs() > rt_tolerance {
                        continue;
                    }
                    let height_ok = if current == idx {
                        let max_carbons = features[idx].mz * charge as f64 / 12.0;
                        features[j].intensity / features[idx].intensity <= max_carbons * CARBON_13_RATIO
                    } else {
                        features[j].intensity < features[current].intensity
                    };
                    if !height_ok {
                        continue;
                    }
                    best = match best {
                        Some(b) if (features[b].mz - target).abs() <= (features[j].mz - target).abs() => Some(b),
                        _ => Some(j),
                    };
                }
                match best {
                    Some(b) => {
                        is_isotope[b] = true;
                        current = b;
                    }
                    None => break,
                }
            }
            if current != idx {
                break;
            }
        }
    }
    features
        .into_iter()
        .zip(is_isotope)
        .filter(|(_, isotope)| !isotope)
        .map(|(feature, _)| feature)
        .collect()
}