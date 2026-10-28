use ionic::mzml::structs::{
    BinaryDataArray, BinaryDataArrayList, Chromatogram, IsolationWindow, MzML, NumericArray,
};
#[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
use rayon::{ThreadPoolBuilder, prelude::*};

use crate::utilities::{
    calculate_baseline::BaselineOptions,
    find_peaks::{FindPeaksOptions, find_peaks},
    get_peak::get_peak,
    structs::{ChromTarget, DataXY, Roi},
};

const ACC_TIME_ARRAY: &str = "MS:1000595";
const ACC_INTENSITY_ARRAY: &str = "MS:1000515";
const ACC_ISOLATION_TARGET: &str = "MS:1000827";
const TRANSITION_TOLERANCE: f64 = 0.005;

#[derive(Clone, Debug)]
pub struct ChromPeakRow {
    pub id: String,
    pub target_rt: f64,
    pub peak_rt: f64,
    pub from_rt: f64,
    pub to_rt: f64,
    pub intensity: f64,
    pub area: f64,
    pub total_area: f64,
    pub timestamp: String,
}

impl ChromPeakRow {
    fn empty(id: String, target_rt: f64, timestamp: String) -> Self {
        Self {
            id,
            target_rt,
            peak_rt: 0.0,
            from_rt: 0.0,
            to_rt: 0.0,
            intensity: 0.0,
            area: 0.0,
            total_area: 0.0,
            timestamp,
        }
    }
}

pub fn get_peaks_from_chrom(
    mzml: &MzML,
    targets: &[ChromTarget],
    options: FindPeaksOptions,
) -> Option<Vec<ChromPeakRow>> {
    let run = &mzml.run;
    let chroms = run.chromatogram_list.as_ref()?.chromatograms.as_slice();
    let timestamp = run.start_time_stamp.clone().unwrap_or_default();

    let threads = options.threads;
    let opts = FindPeaksOptions {
        baseline: Some(BaselineOptions {
            lambda: Some(50.0),
            ..Default::default()
        }),
        ..options
    };

    let make_row = |target: &ChromTarget| {
        let timestamp = timestamp.clone();
        if target.range <= 0.0 || !target.rt.is_finite() {
            return ChromPeakRow::empty(target.id.clone(), target.rt, timestamp);
        }
        let Some(chrom) = find_chromatogram(chroms, target) else {
            return ChromPeakRow::empty(target.id.clone(), target.rt, timestamp);
        };
        let Some((time_values, intensity_values)) = chromatogram_xy(chrom) else {
            return ChromPeakRow::empty(target.id.clone(), target.rt, timestamp);
        };
        compute_one(target, time_values, intensity_values, timestamp, &opts)
    };

    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    {
        let _ = threads;
        return Some(targets.iter().map(make_row).collect());
    }

    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    {
        if threads <= 1 || targets.len() < 2 {
            return Some(targets.iter().map(make_row).collect());
        }
        let pool = ThreadPoolBuilder::new().num_threads(threads).build().ok()?;
        Some(pool.install(|| targets.par_iter().map(make_row).collect()))
    }
}

fn find_chromatogram<'a>(
    chroms: &'a [Chromatogram],
    target: &ChromTarget,
) -> Option<&'a Chromatogram> {
    chroms.iter().find(|chrom| {
        let precursor = chrom
            .precursor
            .as_ref()
            .and_then(|precursor| precursor.isolation_window.as_ref());
        let product = chrom
            .product
            .as_ref()
            .and_then(|product| product.isolation_window.as_ref());
        is_near(get_isolation_target(precursor), target.precursor)
            && is_near(get_isolation_target(product), target.product)
    })
}

fn get_isolation_target(window: Option<&IsolationWindow>) -> Option<f64> {
    window?
        .cv_params
        .iter()
        .find(|param| param.accession.as_deref() == Some(ACC_ISOLATION_TARGET))?
        .value
        .as_deref()?
        .parse()
        .ok()
}

fn is_near(value: Option<f64>, wanted: f64) -> bool {
    value.is_some_and(|value| (value - wanted).abs() <= TRANSITION_TOLERANCE)
}

fn chromatogram_xy(ch: &Chromatogram) -> Option<(Vec<f64>, Vec<f32>)> {
    let bal = ch.binary_data_array_list.as_ref()?;

    let time_bda =
        find_bda_by_accession(bal, ACC_TIME_ARRAY).or_else(|| bal.binary_data_arrays.first());
    let inten_bda =
        find_bda_by_accession(bal, ACC_INTENSITY_ARRAY).or_else(|| bal.binary_data_arrays.get(1));

    let (time_bda, inten_bda) = (time_bda?, inten_bda?);

    let mut x = bda_to_f64(time_bda);
    let mut y = bda_to_f32(inten_bda);

    let n = x.len().min(y.len());
    x.truncate(n);
    y.truncate(n);

    if n >= 1 { Some((x, y)) } else { None }
}

#[inline]
fn find_bda_by_accession<'a>(
    bal: &'a BinaryDataArrayList,
    accession: &str,
) -> Option<&'a BinaryDataArray> {
    bal.binary_data_arrays.iter().find(|bda| {
        bda.cv_params
            .iter()
            .any(|cv| cv.accession.as_deref() == Some(accession))
    })
}

#[inline]
fn bda_to_f64(bda: &BinaryDataArray) -> Vec<f64> {
    let Some(bin) = bda.binary.as_ref() else {
        return Vec::new();
    };
    match bin {
        NumericArray::F64(v) => v.clone(),
        NumericArray::F32(v) => v.iter().map(|&x| x as f64).collect(),
        NumericArray::F16(v) => v.iter().map(|&x| x as f64).collect(),
        NumericArray::I64(v) => v.iter().map(|&x| x as f64).collect(),
        NumericArray::I32(v) => v.iter().map(|&x| x as f64).collect(),
        NumericArray::I16(v) => v.iter().map(|&x| x as f64).collect(),
    }
}

#[inline]
fn bda_to_f32(bda: &BinaryDataArray) -> Vec<f32> {
    let Some(bin) = bda.binary.as_ref() else {
        return Vec::new();
    };
    match bin {
        NumericArray::F16(v) => v.iter().map(|&x| x as f32).collect(),
        NumericArray::F32(v) => v.clone(),
        NumericArray::F64(v) => v.iter().map(|&x| x as f32).collect(),
        NumericArray::I64(v) => v.iter().map(|&x| x as f32).collect(),
        NumericArray::I32(v) => v.iter().map(|&x| x as f32).collect(),
        NumericArray::I16(v) => v.iter().map(|&x| x as f32).collect(),
    }
}

#[inline]
fn compute_one(
    target: &ChromTarget,
    time_values: Vec<f64>,
    intensity_values: Vec<f32>,
    timestamp: String,
    options: &FindPeaksOptions,
) -> ChromPeakRow {
    let mut peak_rt = 0.0_f64;
    let mut from_rt = 0.0_f64;
    let mut to_rt = 0.0_f64;
    let mut intensity = 0.0_f64;
    let mut area = 0.0_f64;
    let mut total_area = 0.0_f64;

    if time_values.len() >= 3 && time_values.len() == intensity_values.len() {
        let data = DataXY {
            x: time_values,
            y: intensity_values.iter().map(|&value| value as f64).collect(),
        };

        let roi = Roi {
            rt: target.rt,
            range: target.range,
        };
        let picked = get_peak(&data, &roi, options.clone());
        if picked.intensity > 0.0 {
            peak_rt = picked.rt;
            from_rt = picked.from;
            to_rt = picked.to;
            intensity = picked.intensity;
            area = picked.integral;
        }

        for peak in find_peaks(&data, options.clone()) {
            total_area += peak.integral;
        }
    }

    ChromPeakRow {
        id: target.id.clone(),
        target_rt: target.rt,
        peak_rt,
        from_rt,
        to_rt,
        intensity,
        area,
        total_area,
        timestamp,
    }
}
