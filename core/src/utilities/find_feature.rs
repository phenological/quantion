#[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
use rayon::{ThreadPoolBuilder, prelude::*};

use ionic::{IonReader, source::ByteRange};

use crate::utilities::{
    calculate_eic::{
        EicOptions, FastError, ScanQuery, get_eic_for_mz, get_scans, lower_bound, plan_scan_ranges,
        upper_bound,
    },
    find_features::max_intensity_centroid,
    find_peaks::FindPeaksOptions,
    get_peak::get_peak,
    ion::ScanSource,
    structs::{DataXY, FromTo, Peak, Roi, Target},
};

#[derive(Clone, Debug, Default)]
pub struct TargetedFeature {
    pub id: String,
    pub mz: f64,
    pub rt: f64,
    pub peak: Peak,
}

#[derive(Clone, Debug)]
pub struct FindFeatureOptions {
    pub seed_eic_options: Option<EicOptions>,
    pub final_eic_options: Option<EicOptions>,
    pub peak_options: Option<FindPeaksOptions>,
    pub threads: usize,
}

impl Default for FindFeatureOptions {
    fn default() -> Self {
        Self {
            seed_eic_options: Some(EicOptions {
                ppm_tolerance: 10.0,
                mz_tolerance: 0.003,
                ..Default::default()
            }),
            final_eic_options: Some(EicOptions {
                ppm_tolerance: 20.0,
                mz_tolerance: 0.005,
                ..Default::default()
            }),
            peak_options: Some(FindPeaksOptions::default()),
            threads: 1,
        }
    }
}

fn feature_time_window(targets: &[Target]) -> Option<FromTo> {
    let mut rt_min = f64::MAX;
    let mut rt_max = f64::MIN;
    for target in targets {
        let (rt, range) = (target.rt, target.range);
        if rt.is_finite() && range.is_finite() && range > 0.0 {
            rt_min = rt_min.min(rt - range);
            rt_max = rt_max.max(rt + range);
        }
    }

    if !rt_min.is_finite() || !rt_max.is_finite() || rt_max <= rt_min {
        return None;
    }
    Some(FromTo {
        from: rt_min,
        to: rt_max,
    })
}

pub fn plan_feature_ranges(
    ion: &mut IonReader,
    targets: &[Target],
) -> Result<Vec<ByteRange>, FastError> {
    let Some(time_window) = feature_time_window(targets) else {
        return Ok(Vec::new());
    };
    plan_scan_ranges(
        ion,
        ScanQuery::RtRange(time_window),
        EicOptions::default().time_unit,
        1,
    )
}

pub fn find_feature(
    source: &mut impl ScanSource,
    targets: &[Target],
    opts: FindFeatureOptions,
) -> Vec<Option<TargetedFeature>> {
    if targets.is_empty() {
        return Vec::new();
    }

    let threads = opts.threads;
    let scan_opts = opts.seed_eic_options.unwrap_or(EicOptions {
        ppm_tolerance: 10.0,
        mz_tolerance: 0.003,
        ..Default::default()
    });
    let eic_opts = opts.final_eic_options.unwrap_or(EicOptions {
        ppm_tolerance: 20.0,
        mz_tolerance: 0.005,
        ..Default::default()
    });
    let fp_opts = opts.peak_options.unwrap_or_default();

    let Some(time_window) = feature_time_window(targets) else {
        return vec![None; targets.len()];
    };

    let (rts, scans) = get_scans(
        source,
        ScanQuery::RtRange(time_window),
        eic_opts.time_unit,
        1,
    );

    if scans.is_empty() || rts.is_empty() {
        return vec![None; targets.len()];
    }

    let process = |target: &Target| -> Option<TargetedFeature> {
        let (rt, mz, range) = (target.rt, target.mz, target.range);
        if !rt.is_finite() || !mz.is_finite() || range <= 0.0 {
            return None;
        }

        let local_from = rt - range;
        let local_to = rt + range;
        let start = lower_bound(&rts, local_from);
        let end = upper_bound(&rts, local_to).min(rts.len());
        if end <= start {
            return None;
        }

        let local_rts = &rts[start..end];
        let local_scans = &scans[start..end];

        let refined_mz = max_intensity_centroid(local_scans, local_rts, rt, mz, scan_opts)?;
        let y = get_eic_for_mz(local_scans, local_rts.len(), refined_mz, eic_opts);
        let data = DataXY {
            x: local_rts.to_vec(),
            y,
        };

        let picked = get_peak(
            &data,
            &Roi {
                rt: rt,
                range: range,
            },
            fp_opts.clone(),
        );
        if picked.intensity <= 0.0 {
            return None;
        }

        Some(TargetedFeature {
            id: target.id.clone(),
            mz: refined_mz,
            rt: picked.rt,
            peak: picked,
        })
    };

    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        return targets.iter().map(process).collect();
    }

    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        if threads <= 1 {
            return targets.iter().map(process).collect();
        }
        let pool = ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("thread pool");
        pool.install(|| targets.par_iter().map(process).collect::<Vec<_>>())
    }
}
