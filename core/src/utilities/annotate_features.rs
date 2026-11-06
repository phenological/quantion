use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fmt,
};

use ionic::{
    ByteRangesOptions, IonReader,
    source::{ByteRange, merge_ranges},
};

use crate::{
    annotations::AnnotateFeaturesOptions,
    files::{CalculateEicOptions, ScanParamKind},
    sample_file::SampleFile,
    targets::FindTargetsOptions,
    utilities::{
        calculate_eic::{FastError, TimeUnit, lower_bound, upper_bound},
        clean_spectrum::{Spectrum, clean_spectrum},
        find_features::Feature,
        find_targets::{find_targets, plan_targets_ranges},
        get_cosine::get_cosine,
        get_scans::{
            CentroidScan, find_wanted_scans, get_scans, plan_scan_ranges, whole_scan_mz_range,
        },
        ion::ScanSource,
        spectrum_model::{SpectrumModel, SpectrumModelError},
        structs::{FromTo, Position, ScanQuery, Target},
    },
};

const MS2_LEVEL: u8 = 2;
const POSITIVE: u8 = 1;
const EIC_HALF_WINDOW: f64 = 0.5;
const ABOVE_PRECURSOR_LIMIT: f64 = 1.0;
const MZ_ARRAY: u32 = 1_000_514;
const INTENSITY_ARRAY: u32 = 1_000_515;

#[derive(Clone, Debug, PartialEq)]
pub struct FeatureMatch {
    pub mz: f64,
    pub rt: f64,
    pub rank: Option<usize>,
    pub frequency: f64,
    pub ms2_frequency: f64,
    pub score: Option<f64>,
    pub model_score: Option<f64>,
    pub cosine: Option<f64>,
    pub file: Option<usize>,
    pub scan_index: Option<usize>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub(crate) enum AnnotationError {
    ReadFailed(FastError),
    Model(SpectrumModelError),
    MissingVector {
        spectrum: usize,
        accession: u32,
    },
    VectorSize {
        spectrum: usize,
        found: usize,
        expected: usize,
    },
    MissingCompound {
        spectrum: usize,
        param: String,
    },
}

impl fmt::Display for AnnotationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadFailed(error) => write!(formatter, "{error}"),
            Self::Model(error) => write!(formatter, "{error}"),
            Self::MissingVector {
                spectrum,
                accession,
            } => write!(formatter, "spectrum {spectrum} has no array MS:{accession}"),
            Self::VectorSize {
                spectrum,
                found,
                expected,
            } => {
                write!(
                    formatter,
                    "spectrum {spectrum} has a vector of {found} values; the model gives {expected}"
                )
            }
            Self::MissingCompound { spectrum, param } => {
                write!(formatter, "spectrum {spectrum} has no userParam {param}")
            }
        }
    }
}

impl From<FastError> for AnnotationError {
    fn from(error: FastError) -> Self {
        Self::ReadFailed(error)
    }
}

impl From<ionic::IonError> for AnnotationError {
    fn from(error: ionic::IonError) -> Self {
        Self::ReadFailed(FastError::from(error))
    }
}

impl From<SpectrumModelError> for AnnotationError {
    fn from(error: SpectrumModelError) -> Self {
        Self::Model(error)
    }
}

pub(crate) fn annotate_features(
    files: &mut [&mut SampleFile],
    features: &[Feature],
    library: &mut IonReader,
    model: &SpectrumModel,
    options: &AnnotateFeaturesOptions,
) -> Result<Vec<FeatureMatch>, AnnotationError> {
    let file_count = files.len() as f64;
    let mut peak_counts = vec![0usize; features.len()];
    let mut scan_counts = vec![0usize; features.len()];
    let mut feature_scans: Vec<FeatureScan> = Vec::new();
    for (position, file) in files.iter_mut().enumerate() {
        for (feature, result) in read_file_results(file, position, features, options)?
            .into_iter()
            .enumerate()
        {
            match result {
                FileResult::NoPeak => {}
                FileResult::PeakOnly => peak_counts[feature] += 1,
                FileResult::Scan(feature_scan) => {
                    peak_counts[feature] += 1;
                    scan_counts[feature] += 1;
                    feature_scans.push(feature_scan);
                }
            }
        }
    }
    feature_scans.sort_by_key(|feature_scan| (feature_scan.feature, feature_scan.file));
    let spectra: Vec<Spectrum> = feature_scans
        .iter()
        .map(|feature_scan| feature_scan.spectrum.clone())
        .collect();
    let vectors = model.get_vectors(&spectra)?;
    drop(spectra);
    let index = LibraryIndex::read(library);
    let mut results = Vec::with_capacity(features.len());
    let mut next_scan = 0;
    for (position, feature) in features.iter().enumerate() {
        let mut scorer = FeatureScorer {
            library,
            index: &index,
            model,
            options,
            candidates: HashMap::new(),
            matches: Vec::new(),
            positions: HashMap::new(),
        };
        while next_scan < feature_scans.len() && feature_scans[next_scan].feature == position {
            scorer.add_scan(&feature_scans[next_scan], &vectors[next_scan])?;
            next_scan += 1;
        }
        let frequency = peak_counts[position] as f64 / file_count;
        let ms2_frequency = scan_counts[position] as f64 / file_count;
        results.extend(scorer.get_ranked(feature, frequency, ms2_frequency));
    }
    Ok(results)
}

fn read_file_results(
    file: &mut SampleFile,
    position: usize,
    features: &[Feature],
    options: &AnnotateFeaturesOptions,
) -> Result<Vec<FileResult>, AnnotationError> {
    let mut results = Vec::with_capacity(features.len());
    for (feature_position, feature) in features.iter().enumerate() {
        let Some(apex) = find_apex(file, feature, options)? else {
            results.push(FileResult::NoPeak);
            continue;
        };
        let found = find_closest_scan(file, feature, apex, options);
        results.push(match found {
            Some(scan) => FileResult::Scan(FeatureScan {
                feature: feature_position,
                file: position,
                scan_index: scan.index,
                precursor: scan.metadata.selected_ion_mz,
                spectrum: clean_spectrum(
                    Spectrum {
                        mz: scan.mz.to_vec(),
                        intensity: scan.intensity.to_vec(),
                    },
                    options.min_relative_intensity,
                ),
            }),
            None => FileResult::PeakOnly,
        });
    }
    Ok(results)
}

struct FeatureScan {
    feature: usize,
    file: usize,
    scan_index: usize,
    precursor: f64,
    spectrum: Spectrum,
}

enum FileResult {
    NoPeak,
    PeakOnly,
    Scan(FeatureScan),
}

fn find_apex(
    file: &mut SampleFile,
    feature: &Feature,
    options: &AnnotateFeaturesOptions,
) -> Result<Option<f64>, FastError> {
    let target = get_target(feature, options);
    let peaks = find_targets(
        &mut file.reader(),
        &[target],
        get_eic_window(feature),
        FindTargetsOptions::default(),
    )?;
    Ok(peaks
        .first()
        .filter(|peak| peak.intensity > 0.0)
        .map(|peak| peak.rt))
}

fn find_closest_scan(
    file: &mut SampleFile,
    feature: &Feature,
    apex: f64,
    options: &AnnotateFeaturesOptions,
) -> Option<CentroidScan> {
    let query = ScanQuery {
        rt: Position::Closest(apex),
        mz: Position::Range(options.get_precursor_window(feature.mz)),
    };
    let scans = get_scans(file, query, TimeUnit::MINUTES, MS2_LEVEL);
    scans.into_iter().next().filter(|scan| {
        scan.metadata.polarity == POSITIVE && (scan.rt - apex).abs() <= options.rt_range
    })
}

struct FeatureScorer<'a> {
    library: &'a mut IonReader,
    index: &'a LibraryIndex,
    model: &'a SpectrumModel,
    options: &'a AnnotateFeaturesOptions,
    candidates: HashMap<usize, Candidate>,
    matches: Vec<CompoundMatch>,
    positions: HashMap<String, usize>,
}

impl FeatureScorer<'_> {
    fn add_scan(
        &mut self,
        feature_scan: &FeatureScan,
        vector: &[f64],
    ) -> Result<(), AnnotationError> {
        let mut spectrum = feature_scan.spectrum.clone();
        spectrum.sort_by_mz();
        spectrum.keep_below(feature_scan.precursor + ABOVE_PRECURSOR_LIMIT);
        let found = self
            .index
            .find(self.options.get_precursor_window(feature_scan.precursor));
        for library_spectrum in found {
            if !self.candidates.contains_key(&library_spectrum.index) {
                let candidate = read_candidate(self.library, &library_spectrum, self.model)?;
                self.candidates.insert(library_spectrum.index, candidate);
            }
            let candidate = &self.candidates[&library_spectrum.index];
            if candidate.vector.len() != vector.len() {
                return Err(AnnotationError::VectorSize {
                    spectrum: library_spectrum.index,
                    found: candidate.vector.len(),
                    expected: vector.len(),
                });
            }
            let model_score: f64 = candidate
                .vector
                .iter()
                .zip(vector)
                .map(|(left, right)| left * right)
                .sum();
            let cosine = get_cosine(&spectrum, &candidate.spectrum);
            let found = CompoundMatch {
                compound: candidate.compound.clone(),
                score: (model_score + cosine) / 2.0,
                model_score,
                cosine,
                file: feature_scan.file,
                scan_index: feature_scan.scan_index,
                library_index: library_spectrum.index,
            };
            self.keep_best(found);
        }
        Ok(())
    }

    fn keep_best(&mut self, found: CompoundMatch) {
        if !found.score.is_finite() {
            return;
        }
        match self.positions.get(&found.compound) {
            Some(position) if found.score > self.matches[*position].score => {
                self.matches[*position] = found
            }
            Some(_) => {}
            None => {
                self.positions
                    .insert(found.compound.clone(), self.matches.len());
                self.matches.push(found);
            }
        }
    }

    fn get_ranked(
        mut self,
        feature: &Feature,
        frequency: f64,
        ms2_frequency: f64,
    ) -> Vec<FeatureMatch> {
        self.matches
            .sort_by(|left, right| right.score.total_cmp(&left.score));
        self.matches.truncate(self.options.matches);
        if self.matches.is_empty() {
            return vec![get_empty_match(feature, frequency, ms2_frequency)];
        }
        self.matches
            .into_iter()
            .enumerate()
            .map(|(position, found)| FeatureMatch {
                mz: feature.mz,
                rt: feature.rt,
                rank: Some(position + 1),
                frequency,
                ms2_frequency,
                score: Some(found.score),
                model_score: Some(found.model_score),
                cosine: Some(found.cosine),
                file: Some(found.file),
                scan_index: Some(found.scan_index),
                metadata: self
                    .candidates
                    .get(&found.library_index)
                    .map(|candidate| candidate.user_params.clone())
                    .unwrap_or_default(),
            })
            .collect()
    }
}

struct CompoundMatch {
    compound: String,
    score: f64,
    model_score: f64,
    cosine: f64,
    file: usize,
    scan_index: usize,
    library_index: usize,
}

fn read_candidate(
    library: &mut IonReader,
    spectrum: &LibrarySpectrum,
    model: &SpectrumModel,
) -> Result<Candidate, AnnotationError> {
    let mut vector = Vec::new();
    if !library.load_array(spectrum.index, model.vector_accession, &mut vector) {
        return Err(AnnotationError::MissingVector {
            spectrum: spectrum.index,
            accession: model.vector_accession,
        });
    }
    let mut peaks = Spectrum::default();
    if !library.load_scan(spectrum.index, &mut peaks.mz, &mut peaks.intensity) {
        peaks = Spectrum::default();
    }
    peaks.sort_by_mz();
    peaks.keep_below(spectrum.precursor + ABOVE_PRECURSOR_LIMIT);
    let mut user_params = BTreeMap::new();
    for row in library.load_metadata(spectrum.index)? {
        if row.kind == ScanParamKind::USER {
            user_params
                .entry(row.name)
                .or_insert_with(|| row.value.unwrap_or_default());
        }
    }
    let compound = user_params
        .get(&model.compound_param)
        .cloned()
        .ok_or_else(|| AnnotationError::MissingCompound {
            spectrum: spectrum.index,
            param: model.compound_param.clone(),
        })?;
    Ok(Candidate {
        compound,
        vector,
        spectrum: peaks,
        user_params,
    })
}

struct Candidate {
    compound: String,
    vector: Vec<f64>,
    spectrum: Spectrum,
    user_params: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug)]
struct LibrarySpectrum {
    index: usize,
    precursor: f64,
}

struct LibraryIndex {
    precursors: Vec<f64>,
    indexes: Vec<usize>,
}

impl LibraryIndex {
    fn read(library: &mut IonReader) -> Self {
        let mut spectra = Vec::new();
        library.for_each_summary(&mut |index, summary| {
            if summary.ms_level == MS2_LEVEL && summary.selected_ion_mz.is_finite() {
                spectra.push(LibrarySpectrum {
                    index,
                    precursor: summary.selected_ion_mz,
                });
            }
        });
        spectra.sort_by(|left, right| {
            left.precursor
                .total_cmp(&right.precursor)
                .then(left.index.cmp(&right.index))
        });
        Self {
            precursors: spectra.iter().map(|spectrum| spectrum.precursor).collect(),
            indexes: spectra.iter().map(|spectrum| spectrum.index).collect(),
        }
    }

    fn find(&self, window: FromTo) -> Vec<LibrarySpectrum> {
        let first = lower_bound(&self.precursors, window.from);
        let last = upper_bound(&self.precursors, window.to).max(first);
        (first..last)
            .map(|position| LibrarySpectrum {
                index: self.indexes[position],
                precursor: self.precursors[position],
            })
            .collect()
    }
}

fn get_empty_match(feature: &Feature, frequency: f64, ms2_frequency: f64) -> FeatureMatch {
    FeatureMatch {
        mz: feature.mz,
        rt: feature.rt,
        rank: None,
        frequency,
        ms2_frequency,
        score: None,
        model_score: None,
        cosine: None,
        file: None,
        scan_index: None,
        metadata: BTreeMap::new(),
    }
}

pub(crate) fn plan_sample_ranges(
    ion: &mut IonReader,
    features: &[Feature],
    options: &AnnotateFeaturesOptions,
) -> Result<Vec<ByteRange>, FastError> {
    ion.require_bounds()?;
    let mut ranges = Vec::new();
    for feature in features {
        let window = get_eic_window(feature);
        let target = get_target(feature, options);
        ranges.extend(plan_targets_ranges(
            ion,
            &[target],
            window.from,
            window.to,
            CalculateEicOptions::default(),
        )?);
        let query = get_ms2_query(feature, options);
        ranges.extend(plan_scan_ranges(
            ion,
            query,
            TimeUnit::MINUTES,
            MS2_LEVEL,
            &ByteRangesOptions::default(),
        )?);
    }
    merge_ranges(&mut ranges, 0);
    Ok(ranges)
}

pub(crate) fn plan_library_ranges(
    library: &mut IonReader,
    files: &mut [&mut SampleFile],
    features: &[Feature],
    options: &AnnotateFeaturesOptions,
    vector_accession: u32,
) -> Result<Vec<ByteRange>, FastError> {
    library.require_bounds()?;
    let index = LibraryIndex::read(library);
    let mut wanted = BTreeSet::new();
    for file in files.iter_mut() {
        for feature in features {
            let scans = find_wanted_scans(
                *file,
                get_ms2_query(feature, options),
                TimeUnit::MINUTES,
                MS2_LEVEL,
            )
            .scans;
            for (_, scan) in scans {
                let candidates = index.find(options.get_precursor_window(scan.selected_ion_mz));
                wanted.extend(candidates.iter().map(|spectrum| spectrum.index));
            }
        }
    }
    let arrays = ByteRangesOptions {
        arrays: vec![MZ_ARRAY, INTENSITY_ARRAY, vector_accession],
    };
    let mut ranges = Vec::new();
    for spectrum in wanted {
        ranges.extend(library.byte_ranges(spectrum, whole_scan_mz_range(), &arrays)?);
        ranges.extend(library.spectrum_metadata_byte_ranges(spectrum)?);
    }
    merge_ranges(&mut ranges, 0);
    Ok(ranges)
}

fn get_target(feature: &Feature, options: &AnnotateFeaturesOptions) -> Target {
    Target {
        id: None,
        mz: feature.mz,
        rt: feature.rt,
        range: Some(options.rt_range),
    }
}

fn get_eic_window(feature: &Feature) -> FromTo {
    FromTo {
        from: feature.rt - EIC_HALF_WINDOW,
        to: feature.rt + EIC_HALF_WINDOW,
    }
}

fn get_ms2_query(feature: &Feature, options: &AnnotateFeaturesOptions) -> ScanQuery {
    let reach = options.rt_range + options.rt_range.min(EIC_HALF_WINDOW);
    ScanQuery {
        rt: Position::Range(FromTo {
            from: feature.rt - reach,
            to: feature.rt + reach,
        }),
        mz: Position::Range(options.get_precursor_window(feature.mz)),
    }
}
