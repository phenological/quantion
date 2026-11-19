use std::path::{Path, PathBuf};

use ionic::{IonError, IonReader, IonResult, ReadOptions};

use crate::Header;
#[cfg(all(
    feature = "http",
    not(all(target_arch = "wasm32", not(target_os = "wasi")))
))]
use crate::utilities::http::HttpClient;

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
/// - Bytes: reads from memory, like `parse_ion_bytes`.
/// - URL: fetches only the byte ranges it needs with HTTP `Range` requests.
///   Needs the `http` feature, and the server must answer `206 Partial Content`.
pub fn parse_ion(input: impl Into<IonInput>, options: &ReadOptions) -> IonResult<IonReader> {
    match input.into() {
        IonInput::Path(path) => open_path(&path, options),
        IonInput::Url(url) => open_url(&url, options),
        IonInput::Bytes(bytes) => IonReader::from_bytes(&bytes, options),
    }
}

pub(crate) fn parse_ion_with_prefetch(
    input: IonInput,
    options: &ReadOptions,
    #[cfg_attr(
        not(all(
            feature = "http",
            not(all(target_arch = "wasm32", not(target_os = "wasi")))
        )),
        allow(unused_variables)
    )]
    headers: Vec<Header>,
) -> IonResult<(IonReader, Option<std::sync::Arc<dyn PrefetchRanges>>)> {
    match input {
        #[cfg(all(
            feature = "http",
            not(all(target_arch = "wasm32", not(target_os = "wasi")))
        ))]
        IonInput::Url(url) => {
            open_remote_ion(&url, options, headers).map(|(reader, cache)| (reader, Some(cache)))
        }
        other => parse_ion(other, options).map(|reader| (reader, None)),
    }
}

pub(crate) fn is_url(text: &str) -> bool {
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
    open_remote_ion(url, options, Vec::new()).map(|(reader, _)| reader)
}

pub(crate) trait PrefetchRanges: Send + Sync {
    fn prefetch(&self, ranges: Vec<ionic::source::ByteRange>) -> IonResult<()>;
}

#[cfg(all(
    feature = "http",
    not(all(target_arch = "wasm32", not(target_os = "wasi")))
))]
pub(crate) fn open_remote_ion(
    url: &str,
    options: &ReadOptions,
    headers: Vec<Header>,
) -> IonResult<(IonReader, std::sync::Arc<dyn PrefetchRanges>)> {
    use std::sync::Arc;

    use ionic::source::{CallbackSource, ReadBytes};

    let cache = Arc::new(RangeCache::new(url, headers));
    let reading = cache.clone();
    let source = CallbackSource::new(move |range| reading.read(range));
    let reader = IonReader::new(Arc::new(source) as Arc<dyn ReadBytes>, options)?;
    Ok((reader, cache as Arc<dyn PrefetchRanges>))
}

#[cfg(all(
    feature = "http",
    not(all(target_arch = "wasm32", not(target_os = "wasi")))
))]
pub(crate) struct RangeCache {
    client: HttpClient,
    url: String,
    held: std::sync::Mutex<Vec<(ionic::source::ByteRange, std::sync::Arc<[u8]>)>>,
    pub(crate) request_count: std::sync::atomic::AtomicUsize,
}

#[cfg(all(
    feature = "http",
    not(all(target_arch = "wasm32", not(target_os = "wasi")))
))]
impl RangeCache {
    fn new(url: &str, headers: Vec<Header>) -> Self {
        Self {
            client: HttpClient::new(headers),
            url: url.to_string(),
            held: std::sync::Mutex::new(Vec::new()),
            request_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    fn fetch(&self, range: ionic::source::ByteRange) -> IonResult<Vec<u8>> {
        self.request_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.client.get_range(&self.url, range)
    }

    fn read(&self, range: ionic::source::ByteRange) -> IonResult<Vec<u8>> {
        if range.length == 0 {
            return Ok(Vec::new());
        }
        match self.read_held(range) {
            Some(bytes) => Ok(bytes),
            None => self.fetch(range),
        }
    }

    fn read_held(&self, range: ionic::source::ByteRange) -> Option<Vec<u8>> {
        let held = self.held.lock().ok()?;
        held.iter().find_map(|(stored, bytes)| {
            let start = range.offset.checked_sub(stored.offset)?;
            let end = start.checked_add(range.length)?;
            (end <= stored.length).then(|| bytes[start as usize..end as usize].to_vec())
        })
    }
}

#[cfg(all(
    feature = "http",
    not(all(target_arch = "wasm32", not(target_os = "wasi")))
))]
impl PrefetchRanges for RangeCache {
    fn prefetch(&self, mut ranges: Vec<ionic::source::ByteRange>) -> IonResult<()> {
        ionic::source::merge_ranges(&mut ranges, LARGEST_PREFETCH_GAP);
        let mut fetched = Vec::with_capacity(ranges.len());
        for range in ranges {
            let bytes = match self.read_held(range) {
                Some(bytes) => bytes,
                None => self.fetch(range)?,
            };
            fetched.push((range, std::sync::Arc::from(bytes)));
        }
        let mut held = self
            .held
            .lock()
            .map_err(|_| IonError::from("parse_ion: the range cache is unusable"))?;
        *held = fetched;
        Ok(())
    }
}

#[cfg(all(
    feature = "http",
    not(all(target_arch = "wasm32", not(target_os = "wasi")))
))]
const LARGEST_PREFETCH_GAP: u64 = 131072;

#[cfg(not(all(
    feature = "http",
    not(all(target_arch = "wasm32", not(target_os = "wasi")))
)))]
fn open_url(url: &str, _options: &ReadOptions) -> IonResult<IonReader> {
    Err(IonError::from(format!(
        "parse_ion: {url} is a URL; enable the `http` feature of quantion to open URLs"
    )))
}
