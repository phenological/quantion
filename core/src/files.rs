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

pub fn calculate_eic(file: &mut SampleFile, mz: f64, options: CalculateEicOptions) -> Result<DataXY, QuantionError> {
    const FUNCTION: &str = "calculate_eic";
    check_positive(FUNCTION, "mz", mz)?;
    options.check(FUNCTION, "")?;
    read_eic(&mut file.reader(), mz, options.get_window(), options.in_minutes()).map_err(|error| from_fast_error(FUNCTION, error))
}

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
