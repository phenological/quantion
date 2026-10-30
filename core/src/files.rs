use std::{fmt::Display, fs, path::Path};

use ionic::{
    ReadOptions, WriteOptions,
    mzml::{bin_to_mzml, parse_mzml as read_mzml},
};

use crate::{
    error::{ErrorKind, QuantionError, check_finite, check_not_negative, check_positive},
    options::{CalculateEicOptions, GetScansOptions, Header, GetIonImageOptions, IonOutput, ParseIonOptions, WriteIonOptions, WriteMzmlOptions},
    sample_file::{FileSource, MzmlInput, SampleFile, is_url_path},
    utilities::{
        calculate_eic::{CentroidScan, FastError, ScanQuery, calculate_eic as read_eic, get_scans as read_scans},
        ion::write_mzml_to_ion,
        ion_image::{IonImage, image_session_begin},
        parse_ion::{IonInput, parse_ion as open_ion},
        structs::DataXY,
    },
};

const MAX_COMPRESSION_LEVEL: u8 = 22;
const IMAGE_SCANS_PER_STEP: usize = 256;
const RESERVED_HEADERS: [&str; 3] = ["host", "content-length", "transfer-encoding"];

/// **Scan** · Results
///
/// One scan from get_scans: its retention time, centroided m/z and intensity values, and its details.
///
/// Fields:
/// - `rt`: Retention time (minutes).
/// - `mz`: m/z values.
/// - `intensity`: Intensities, same length as mz.
/// - `level`: MS level.
/// - `precursor_mz`: Precursor m/z (MS2 and higher).
/// - `base_peak_mz`: Base peak m/z.
/// - `base_peak_intensity`: Base peak intensity.
/// - `total_ion_current`: Total ion current.
/// - `polarity`: Polarity: 1 positive, 2 negative, 0 unknown.
/// - `x`: Imaging x position. 0 in non-imaging files.
/// - `y`: Imaging y position. 0 in non-imaging files.
/// - `z`: Imaging z position. 0 in non-imaging files.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#Scan)
#[derive(Clone, Debug, PartialEq)]
pub struct Scan {
    pub rt: f64,
    pub mz: Vec<f64>,
    pub intensity: Vec<f64>,
    pub level: u8,
    pub precursor_mz: f64,
    pub base_peak_mz: f64,
    pub base_peak_intensity: f64,
    pub total_ion_current: f64,
    pub polarity: u8,
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl From<CentroidScan> for Scan {
    fn from(scan: CentroidScan) -> Self {
        let details = scan.metadata;
        Self {
            rt: scan.rt,
            mz: scan.mz.to_vec(),
            intensity: scan.intensity.to_vec(),
            level: details.ms_level,
            precursor_mz: details.selected_ion_mz,
            base_peak_mz: details.base_peak_mz,
            base_peak_intensity: details.base_peak_int,
            total_ion_current: details.total_ion_current,
            polarity: details.polarity,
            x: details.position_x,
            y: details.position_y,
            z: details.position_z,
        }
    }
}

pub(crate) fn read_error(function: &'static str, error: impl Display) -> QuantionError {
    QuantionError::new(ErrorKind::ParseFailed, function, format!("the file could not be read: {error}"))
}

pub(crate) fn from_fast_error(function: &'static str, error: FastError) -> QuantionError {
    match error {
        FastError::InvalidRequest => QuantionError::new(ErrorKind::InvalidArgument, function, "the request is not valid"),
        other => read_error(function, other),
    }
}

/// **parse_ion** · Files
///
/// Open an Ionic file (.ion) from a path, a URL or bytes. Paths and URLs are read lazily: only the bytes a query needs are fetched.
///
/// **Warning:** The sample is freed when it goes out of scope, so nothing else is needed in normal use. When processing many samples, drop each one after using it, with drop(file), so its memory is freed right away instead of piling up.
///
/// - `input`: Where the file is: a path, an http(s) URL, or raw bytes.
/// - `options`: Optional settings: `max_cache_size` = 0. See [ParseIonOptions](https://phenological.github.io/quantion/?language=rust#ParseIonOptions).
///
/// Returns: The opened sample.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#parse_ion)
pub fn parse_ion(input: impl Into<IonInput>, options: ParseIonOptions) -> Result<SampleFile, QuantionError> {
    const FUNCTION: &str = "parse_ion";
    let input = input.into();
    if let IonInput::Path(path) = &input
        && !path.exists()
    {
        return Err(QuantionError::new(
            ErrorKind::FileNotFound,
            FUNCTION,
            format!("{} does not exist", path.display()),
        ));
    }
    let is_remote = matches!(input, IonInput::Url(_));
    let read_options = ReadOptions {
        max_cached_bytes: options.max_cache_size,
        ..Default::default()
    };
    let reader = open_ion(input, &read_options).map_err(|error| read_error(FUNCTION, error))?;
    let source = if is_remote {
        FileSource::Remote(Box::new(reader))
    } else {
        FileSource::Lazy(Box::new(reader))
    };
    Ok(SampleFile::new(source))
}

/// **parse_mzml** · Files
///
/// Open an mzML file from a path or bytes. It is always loaded fully into memory. URLs are refused.
///
/// **Warning:** The sample is freed when it goes out of scope, so nothing else is needed in normal use. When processing many samples, drop each one after using it, with drop(file), so its memory is freed right away instead of piling up.
///
/// - `input`: Where the file is: a path or raw bytes.
///
/// Returns: The loaded sample.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#parse_mzml)
pub fn parse_mzml(input: impl Into<MzmlInput>) -> Result<SampleFile, QuantionError> {
    const FUNCTION: &str = "parse_mzml";
    let bytes = match input.into() {
        MzmlInput::Path(path) if is_url_path(&path) => return Err(QuantionError::remote_file(FUNCTION)),
        MzmlInput::Path(path) => fs::read(&path).map_err(|_| {
            QuantionError::new(ErrorKind::FileNotFound, FUNCTION, format!("{} could not be opened", path.display()))
        })?,
        MzmlInput::Bytes(bytes) => bytes,
    };
    let mzml = read_mzml(&bytes).map_err(|error| read_error(FUNCTION, error))?;
    Ok(SampleFile::new(FileSource::Full(Box::new(mzml))))
}

/// **write_ion** · Files
///
/// Write a sample as an Ionic file (.ion). With no out, it returns the bytes. A path writes a file; a URL uploads it in one PUT request.
///
/// - `file`: The sample to write.
/// - `options`: Optional settings: `out` = None, `overwrite` = false, `level` = 12, `f32_compress` = false, `headers` = empty. See [WriteIonOptions](https://phenological.github.io/quantion/?language=rust#WriteIonOptions).
///
/// Returns: The bytes when out is left out; nothing when it is written or uploaded.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#write_ion)
pub fn write_ion(file: &mut SampleFile, options: WriteIonOptions) -> Result<Option<Vec<u8>>, QuantionError> {
    const FUNCTION: &str = "write_ion";
    let bytes = encode_ion(file, &options)?;
    match options.out {
        None => Ok(Some(bytes)),
        Some(IonOutput::Path(path)) => {
            write_file(FUNCTION, &path, &bytes)?;
            Ok(None)
        }
        Some(IonOutput::Url(url)) => {
            upload(FUNCTION, &url, &options.headers, bytes)?;
            Ok(None)
        }
    }
}

pub(crate) fn encode_ion(file: &mut SampleFile, options: &WriteIonOptions) -> Result<Vec<u8>, QuantionError> {
    const FUNCTION: &str = "write_ion";
    if options.level > MAX_COMPRESSION_LEVEL {
        return Err(QuantionError::invalid(
            FUNCTION,
            "level",
            format!("must be between 0 and {MAX_COMPRESSION_LEVEL}, got {}", options.level),
        ));
    }
    let is_upload = matches!(options.out, Some(IonOutput::Url(_)));
    if !options.headers.is_empty() && !is_upload {
        return Err(QuantionError::invalid(FUNCTION, "headers", "can only be used when out is a URL"));
    }
    check_headers(FUNCTION, &options.headers)?;
    if let Some(IonOutput::Path(path)) = &options.out {
        check_can_write(FUNCTION, path, options.overwrite)?;
    }
    file.with_mzml(FUNCTION, |mzml| {
        let mut bytes = Vec::new();
        let write_options = WriteOptions {
            compression_level: options.level,
            force_f32: options.f32_compress,
            ..Default::default()
        };
        write_mzml_to_ion(mzml, write_options, &mut bytes)
            .map_err(|error| QuantionError::internal(FUNCTION, format!("the .ion file could not be written: {error}")))?;
        Ok(bytes)
    })
}

/// **write_mzml** · Files
///
/// Write a sample as mzML. With no out, it returns the bytes. Only a path is allowed, no URL.
///
/// - `file`: The sample to write.
/// - `options`: Optional settings: `out` = None, `overwrite` = false. See [WriteMzmlOptions](https://phenological.github.io/quantion/?language=rust#WriteMzmlOptions).
///
/// Returns: The bytes when out is left out; nothing when it is written.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#write_mzml)
pub fn write_mzml(file: &mut SampleFile, options: WriteMzmlOptions) -> Result<Option<Vec<u8>>, QuantionError> {
    const FUNCTION: &str = "write_mzml";
    if let Some(path) = &options.out {
        if is_url_path(path) {
            return Err(QuantionError::invalid(FUNCTION, "out", "must be a path; mzML cannot be written to a URL"));
        }
        check_can_write(FUNCTION, path, options.overwrite)?;
    }
    let bytes = file.with_mzml(FUNCTION, |mzml| {
        bin_to_mzml(mzml).map_err(|error| QuantionError::internal(FUNCTION, format!("the mzML could not be written: {error}")))
    })?;
    match options.out {
        None => Ok(Some(bytes)),
        Some(path) => {
            write_file(FUNCTION, &path, &bytes)?;
            Ok(None)
        }
    }
}

fn check_headers(function: &'static str, headers: &[Header]) -> Result<(), QuantionError> {
    for (index, header) in headers.iter().enumerate() {
        let name = header.name.trim();
        if name.is_empty() {
            return Err(QuantionError::invalid(function, format!("headers[{index}].name"), "must not be empty"));
        }
        let has_line_break = |text: &str| text.contains('\r') || text.contains('\n');
        if has_line_break(&header.name) || has_line_break(&header.value) {
            return Err(QuantionError::invalid(function, format!("headers[{index}]"), "must not contain a line break"));
        }
        if RESERVED_HEADERS.contains(&name.to_ascii_lowercase().as_str()) {
            return Err(QuantionError::invalid(
                function,
                format!("headers[{index}].name"),
                format!("must not be {name}; quantion sets it"),
            ));
        }
    }
    Ok(())
}

fn check_can_write(function: &'static str, path: &Path, overwrite: bool) -> Result<(), QuantionError> {
    if path.exists() && !overwrite {
        return Err(QuantionError {
            kind: ErrorKind::FileExists,
            function,
            argument: Some("out".to_string()),
            message: format!("{} already exists; set overwrite to replace it", path.display()),
        });
    }
    Ok(())
}

fn write_file(function: &'static str, path: &Path, bytes: &[u8]) -> Result<(), QuantionError> {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".part");
    let could_not_write = |error: std::io::Error| {
        QuantionError::invalid(function, "out", format!("could not be written ({}): {error}", path.display()))
    };
    fs::write(&temporary, bytes).map_err(could_not_write)?;
    fs::rename(&temporary, path).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        could_not_write(error)
    })
}

#[cfg(all(feature = "http", not(all(target_arch = "wasm32", not(target_os = "wasi")))))]
fn upload(function: &'static str, url: &str, headers: &[Header], bytes: Vec<u8>) -> Result<(), QuantionError> {
    let agent = ureq::Agent::new_with_defaults();
    let mut request = agent.put(url);
    for header in headers {
        request = request.header(header.name.trim(), header.value.as_str());
    }
    match request.send(&bytes[..]) {
        Ok(_) => Ok(()),
        Err(ureq::Error::StatusCode(status)) => Err(QuantionError::new(
            ErrorKind::UploadFailed,
            function,
            format!("PUT returned {status}"),
        )),
        Err(error) => Err(QuantionError::new(ErrorKind::UploadFailed, function, format!("PUT failed: {error}"))),
    }
}

#[cfg(not(all(feature = "http", not(all(target_arch = "wasm32", not(target_os = "wasi"))))))]
fn upload(function: &'static str, _url: &str, _headers: &[Header], _bytes: Vec<u8>) -> Result<(), QuantionError> {
    Err(QuantionError::invalid(function, "out", "cannot be a URL: this build of quantion has no HTTP support"))
}

/// **get_scans** · Files
///
/// Read scans by retention time or precursor m/z. A range returns every scan in it; one number returns the closest scan, as a list of one.
///
/// - `file`: The sample to read, from parse_ion or parse_mzml.
/// - `query`: Which scans, by retention time: a range or one value. Each ScanQuery has `RtRange(FromTo)`, `Rt(f64)`, `MzRange(FromTo)`, `Mz(f64)`. See [ScanQuery](https://phenological.github.io/quantion/?language=rust#ScanQuery).
///   Example: `let query = ScanQuery::RtRange(FromTo { from: 1.0, to: 2.5 });`
/// - `query`: Or by precursor m/z: a range or one value. Give rt or mz, not both. Each ScanQuery has `RtRange(FromTo)`, `Rt(f64)`, `MzRange(FromTo)`, `Mz(f64)`. See [ScanQuery](https://phenological.github.io/quantion/?language=rust#ScanQuery).
///   Example: `let query = ScanQuery::RtRange(FromTo { from: 1.0, to: 2.5 });`
/// - `options`: Optional settings: `level` = 1, `time_unit` = MINUTES. See [GetScansOptions](https://phenological.github.io/quantion/?language=rust#GetScansOptions).
///
/// Returns: The scans found, in rt order.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#get_scans)
pub fn get_scans(file: &mut SampleFile, query: ScanQuery, options: GetScansOptions) -> Result<Vec<Scan>, QuantionError> {
    const FUNCTION: &str = "get_scans";
    match query {
        ScanQuery::Rt(rt) => check_finite(FUNCTION, "rt", rt)?,
        ScanQuery::Mz(mz) => check_positive(FUNCTION, "mz", mz)?,
        ScanQuery::RtRange(range) | ScanQuery::MzRange(range) => {
            check_finite(FUNCTION, "from", range.from)?;
            if range.to.is_nan() || range.to < range.from {
                return Err(QuantionError::invalid(
                    FUNCTION,
                    "to",
                    format!("must be at least from ({}), got {}", range.from, range.to),
                ));
            }
        }
    }
    let (_, scans) = match &mut file.source {
        FileSource::Full(mzml) => read_scans(mzml.as_mut(), query, options.time_unit, options.level),
        FileSource::Lazy(ion) | FileSource::Remote(ion) => read_scans(ion.as_mut(), query, options.time_unit, options.level),
    };
    Ok(scans.into_iter().map(Scan::from).collect())
}

/// **calculate_eic** · Files
///
/// Extract the ion chromatogram of one m/z.
///
/// - `file`: The sample to read, from parse_ion or parse_mzml.
/// - `mz`: The m/z to extract (Da).
/// - `options`: Optional settings: `from` = 0.0, `to` = f64::INFINITY, `ppm_tolerance` = 20.0, `mz_tolerance` = 0.005, `time_unit` = MINUTES. See [CalculateEicOptions](https://phenological.github.io/quantion/?language=rust#CalculateEicOptions).
///
/// Returns: The chromatogram: x = retention times, y = summed intensities.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#calculate_eic)
pub fn calculate_eic(file: &mut SampleFile, mz: f64, options: CalculateEicOptions) -> Result<DataXY, QuantionError> {
    const FUNCTION: &str = "calculate_eic";
    check_positive(FUNCTION, "mz", mz)?;
    options.check(FUNCTION, "")?;
    read_eic(&mut file.reader(), mz, options.get_window(), options.in_minutes()).map_err(|error| from_fast_error(FUNCTION, error))
}

/// **get_ion_image** · Files
///
/// Build an imaging map of one m/z: summed intensity at each x, y position.
///
/// **Warning:** Only Ionic files (.ion). The sample must come from parse_ion; a sample from parse_mzml gives an invalid_argument error.
///
/// - `file`: The sample to read. Must come from parse_ion.
/// - `mz`: The m/z to map (Da).
/// - `options`: Optional settings: `tolerance` = 0.125, `level` = 1, `on_progress` = None. See [GetIonImageOptions](https://phenological.github.io/quantion/?language=rust#GetIonImageOptions).
///
/// Returns: The image.
///
/// [Documentation](https://phenological.github.io/quantion/?language=rust#get_ion_image)
pub fn get_ion_image(file: &mut SampleFile, mz: f64, options: GetIonImageOptions) -> Result<IonImage, QuantionError> {
    const FUNCTION: &str = "get_ion_image";
    if !file.is_ion() {
        return Err(QuantionError::invalid(
            FUNCTION,
            "file",
            "must be an .ion file from parse_ion; an mzML sample cannot be imaged",
        ));
    }
    check_positive(FUNCTION, "mz", mz)?;
    check_not_negative(FUNCTION, "tolerance", options.tolerance)?;
    let mut reader = file.reader();
    let mut session = image_session_begin(&mut reader, mz, options.tolerance, options.level);
    let total = session.scan_count();
    let mut done = 0;
    while done < total {
        let count = IMAGE_SCANS_PER_STEP.min(total - done);
        session.fold(&mut reader, done, count);
        done += count;
        if let Some(on_progress) = options.on_progress {
            on_progress(done as u64, total as u64, 0);
        }
    }
    Ok(session.finish())
}
