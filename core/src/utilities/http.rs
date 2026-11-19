use ionic::{IonError, IonResult, source::ByteRange};

use crate::Header;

pub(crate) struct HttpClient {
    agent: ureq::Agent,
    headers: Vec<Header>,
}

impl HttpClient {
    pub(crate) fn new(headers: Vec<Header>) -> Self {
        Self {
            agent: ureq::Agent::new_with_defaults(),
            headers,
        }
    }

    fn with_headers<B>(&self, request: ureq::RequestBuilder<B>) -> ureq::RequestBuilder<B> {
        self.headers.iter().fold(request, |request, header| {
            request.header(header.name.trim(), header.value.as_str())
        })
    }

    pub(crate) fn put(&self, url: &str, bytes: &[u8]) -> Result<(), ureq::Error> {
        self.with_headers(self.agent.put(url))
            .send(bytes)
            .map(|_| ())
    }

    pub(crate) fn get_range(&self, url: &str, range: ByteRange) -> IonResult<Vec<u8>> {
        if range.length == 0 {
            return Ok(Vec::new());
        }
        let last = range.offset + range.length - 1;
        let mut response = self
            .with_headers(self.agent.get(url))
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
            .limit(range.length + 1)
            .read_to_vec()
            .map_err(|error| IonError::from(format!("parse_ion: reading {url} failed: {error}")))
    }
}
