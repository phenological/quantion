use std::path::{Path, PathBuf};

use ionic::{ArrayKind, IonReader, ReadOptions};

use crate::utilities::parse_ion::{IonInput, parse_ion};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("test.ion")
}

/// Every chromatogram's time and intensity arrays, plus the spectrum count.
fn contents(mut ion: IonReader) -> (u64, Vec<Vec<f64>>) {
    let mut arrays = Vec::new();
    for index in 0..ion.chromatogram_count() as usize {
        for kind in [ArrayKind::Time, ArrayKind::Intensity] {
            arrays.push(
                ion.chromatogram_array(index, kind)
                    .expect("chromatogram array"),
            );
        }
    }
    (ion.spectrum_count(), arrays)
}

#[test]
fn picks_the_input_kind() {
    assert!(matches!(IonInput::from("a/b.ion"), IonInput::Path(_)));
    assert!(matches!(
        IonInput::from("https://x/b.ion"),
        IonInput::Url(_)
    ));
    assert!(matches!(IonInput::from("HTTP://x/b.ion"), IonInput::Url(_)));
    assert!(matches!(IonInput::from(vec![0u8; 4]), IonInput::Bytes(_)));
    assert!(matches!(IonInput::from(fixture()), IonInput::Path(_)));
}

#[test]
fn path_and_bytes_give_the_same_contents() {
    let options = ReadOptions::default();
    let from_path = contents(parse_ion(fixture(), &options).expect("open path"));
    let text = fixture().to_string_lossy().into_owned();
    let from_text = contents(parse_ion(text.as_str(), &options).expect("open text path"));
    let bytes = std::fs::read(fixture()).expect("read fixture");
    let from_bytes = contents(parse_ion(bytes, &options).expect("open bytes"));

    assert!(from_path.1.iter().any(|array| !array.is_empty()));
    assert_eq!(from_path, from_text);
    assert_eq!(from_path, from_bytes);
}

#[test]
fn a_missing_path_is_an_error() {
    assert!(parse_ion("does/not/exist.ion", &ReadOptions::default()).is_err());
}

#[cfg(not(feature = "http"))]
#[test]
fn a_url_needs_the_http_feature() {
    let error = parse_ion("https://example.com/a.ion", &ReadOptions::default())
        .err()
        .expect("url must fail without the http feature");
    assert!(format!("{error:?}").contains("http"));
}

#[cfg(feature = "http")]
mod http {
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        thread,
    };

    use super::*;

    #[derive(Default)]
    struct Counts {
        sent: AtomicUsize,
        requests: AtomicUsize,
        with_token: AtomicUsize,
    }

    const TOKEN_LINE: &str = "authorization: bearer test-token\r\n";

    /// Serves `bytes` with `206 Partial Content` for every `Range` request and
    /// counts the bytes sent, the requests, and the requests carrying the token.
    fn serve(bytes: Vec<u8>, counts: Arc<Counts>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let address = listener.local_addr().expect("address");
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut reader = BufReader::new(stream.try_clone().expect("clone"));
                loop {
                    let mut range = None;
                    let mut line = String::new();
                    let mut first = true;
                    loop {
                        line.clear();
                        if reader.read_line(&mut line).unwrap_or(0) == 0 {
                            return;
                        }
                        if line == "\r\n" && !first {
                            break;
                        }
                        first = false;
                        let lower = line.to_ascii_lowercase();
                        if lower == TOKEN_LINE {
                            counts.with_token.fetch_add(1, Ordering::Relaxed);
                        }
                        if let Some(value) = lower.strip_prefix("range: bytes=") {
                            let (a, b) = value.trim().split_once('-').expect("range");
                            range =
                                Some((a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap()));
                        }
                    }
                    let (start, end) = range.expect("request without Range");
                    let end = end.min(bytes.len() - 1);
                    let body = &bytes[start..=end];
                    counts.sent.fetch_add(body.len(), Ordering::Relaxed);
                    counts.requests.fetch_add(1, Ordering::Relaxed);
                    let head = format!(
                        "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {start}-{end}/{}\r\n\r\n",
                        body.len(),
                        bytes.len()
                    );
                    if stream.write_all(head.as_bytes()).is_err() || stream.write_all(body).is_err()
                    {
                        break;
                    }
                }
            }
        });
        format!("http://{address}/test.ion")
    }

    #[test]
    fn url_gives_the_same_contents_as_path() {
        let bytes = std::fs::read(fixture()).expect("read fixture");
        let size = bytes.len();
        let counts = Arc::new(Counts::default());
        let url = serve(bytes, counts.clone());
        let options = ReadOptions::default();

        let from_url = contents(parse_ion(url.as_str(), &options).expect("open url"));
        let from_path = contents(parse_ion(fixture(), &options).expect("open path"));

        assert_eq!(from_url, from_path);
        let sent = counts.sent.load(Ordering::Relaxed);
        println!("sent {sent} of {size} bytes");
        assert!(sent > 0);
        assert_eq!(counts.with_token.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn headers_reach_the_server_on_every_request() {
        use crate::{
            GetScansOptions, Header, ParseIonOptions, Position, ScanQuery, get_scans, parse_ion,
        };

        let api_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/api/api.ion");
        let bytes = std::fs::read(api_file).expect("read fixture");
        let counts = Arc::new(Counts::default());
        let url = serve(bytes, counts.clone());
        let options = ParseIonOptions {
            headers: vec![Header {
                name: "Authorization".to_string(),
                value: "Bearer test-token".to_string(),
            }],
            ..Default::default()
        };

        let mut file = parse_ion(url.as_str(), options).expect("open url");
        let opened = counts.requests.load(Ordering::Relaxed);
        let scans = get_scans(
            &mut file,
            ScanQuery {
                rt: Position::Closest(2.385),
                ..Default::default()
            },
            GetScansOptions::default(),
        )
        .expect("read scans");

        let requests = counts.requests.load(Ordering::Relaxed);
        assert!(!scans.is_empty());
        assert!(opened > 0);
        assert!(requests > opened);
        assert_eq!(counts.with_token.load(Ordering::Relaxed), requests);
    }
}
