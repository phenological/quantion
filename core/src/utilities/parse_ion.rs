use std::path::{Path, PathBuf};

use ionic::{IonError, IonReader, IonResult, ReadOptions};

/// What `parse_ion` can open. Build it with `.into()` from a path, a URL
/// string, or bytes, and `parse_ion` picks the matching reader.
#[derive(Debug, Clone)]
pub enum IonInput {
    Path(PathBuf),
    Url(String),
    Bytes(Vec<u8>),
}

impl IonInput {
    /// A string that starts with `http://` or `https://` is a URL. Any other
    /// string is a file path.
    pub fn from_text(text: &str) -> Self {
        if is_url(text) {
            IonInput::Url(text.to_string())
        } else {
            IonInput::Path(PathBuf::from(text))
        }
    }
}

impl From<&str> for IonInput {
    fn from(text: &str) -> Self {
        IonInput::from_text(text)
    }
}

impl From<String> for IonInput {
    fn from(text: String) -> Self {
        if is_url(&text) {
            IonInput::Url(text)
        } else {
            IonInput::Path(PathBuf::from(text))
        }
    }
}

impl From<&String> for IonInput {
    fn from(text: &String) -> Self {
        IonInput::from_text(text)
    }
}

impl From<&Path> for IonInput {
    fn from(path: &Path) -> Self {
        IonInput::Path(path.to_path_buf())
    }
}

impl From<PathBuf> for IonInput {
    fn from(path: PathBuf) -> Self {
        IonInput::Path(path)
    }
}

impl From<&PathBuf> for IonInput {
    fn from(path: &PathBuf) -> Self {
        IonInput::Path(path.clone())
    }
}

impl From<Vec<u8>> for IonInput {
    fn from(bytes: Vec<u8>) -> Self {
        IonInput::Bytes(bytes)
    }
}

impl From<&[u8]> for IonInput {
    fn from(bytes: &[u8]) -> Self {
        IonInput::Bytes(bytes.to_vec())
    }
}

impl From<&Vec<u8>> for IonInput {
    fn from(bytes: &Vec<u8>) -> Self {
        IonInput::Bytes(bytes.clone())
    }
}

/// Open an ion file from a path, a URL, or bytes.
///
/// - Path: memory-maps the file, like `parse_ion_path`.
/// - Bytes: reads from memory, like `parse_bin`.
/// - URL: fetches only the byte ranges it needs with HTTP `Range` requests.
///   Needs the `http` feature, and the server must answer `206 Partial Content`.
pub fn parse_ion(input: impl Into<IonInput>, options: &ReadOptions) -> IonResult<IonReader> {
    match input.into() {
        IonInput::Path(path) => open_path(&path, options),
        IonInput::Url(url) => open_url(&url, options),
        IonInput::Bytes(bytes) => IonReader::from_bytes(&bytes, options),
    }
}

fn is_url(text: &str) -> bool {
    let head = text.get(..8).unwrap_or(text).to_ascii_lowercase();
    head.starts_with("http://") || head.starts_with("https://")
}

#[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
fn open_path(path: &Path, options: &ReadOptions) -> IonResult<IonReader> {
    IonReader::open(path, options)
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn open_path(_path: &Path, _options: &ReadOptions) -> IonResult<IonReader> {
    Err(IonError::from(
        "parse_ion: file paths are not supported on wasm; pass bytes",
    ))
}

#[cfg(all(
    feature = "http",
    not(all(target_arch = "wasm32", not(target_os = "wasi")))
))]
fn open_url(url: &str, options: &ReadOptions) -> IonResult<IonReader> {
    use std::sync::Arc;

    use ionic::source::{ByteRange, CallbackSource, ReadBytes};

    let agent = ureq::Agent::new_with_defaults();
    let url = url.to_string();
    let source = CallbackSource::new(move |range: ByteRange| fetch_range(&agent, &url, range));
    IonReader::new(Arc::new(source) as Arc<dyn ReadBytes>, options)
}

#[cfg(all(
    feature = "http",
    not(all(target_arch = "wasm32", not(target_os = "wasi")))
))]
fn fetch_range(
    agent: &ureq::Agent,
    url: &str,
    range: ionic::source::ByteRange,
) -> IonResult<Vec<u8>> {
    if range.length == 0 {
        return Ok(Vec::new());
    }
    let last = range.offset + range.length - 1;
    let mut response = agent
        .get(url)
        .header("Range", format!("bytes={}-{}", range.offset, last))
        .call()
        .map_err(|error| IonError::from(format!("parse_ion: GET {url} failed: {error}")))?;
    if response.status().as_u16() != 206 {
        return Err(IonError::from(format!(
            "parse_ion: {url} answered {} instead of 206; the server must support Range requests",
            response.status()
        )));
    }
    response
        .body_mut()
        .with_config()
        // ureq fails a body whose size equals the limit, so allow one more
        // byte; `CallbackSource` rejects a body longer than the range.
        .limit(range.length + 1)
        .read_to_vec()
        .map_err(|error| IonError::from(format!("parse_ion: reading {url} failed: {error}")))
}

#[cfg(not(all(
    feature = "http",
    not(all(target_arch = "wasm32", not(target_os = "wasi")))
)))]
fn open_url(url: &str, _options: &ReadOptions) -> IonResult<IonReader> {
    Err(IonError::from(format!(
        "parse_ion: {url} is a URL; enable the `http` feature of quantion to open URLs"
    )))
}
