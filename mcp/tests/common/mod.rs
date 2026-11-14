#![allow(dead_code)]

use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use serde_json::{Value, json};

pub const PEAK_MZ: f64 = 300.1;
pub const PEAK_RT: f64 = 2.0;
pub const PEAK_HEIGHT: f64 = 100_000.0;
const PEAK_WIDTH: f64 = 0.05;
const SCAN_COUNT: usize = 201;
const FIRST_RT: f64 = 1.0;
const SCAN_STEP: f64 = 0.01;

pub struct TestFolder {
    pub path: PathBuf,
}

impl TestFolder {
    pub fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("quantion-mcp-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("test folder");
        let path = path.canonicalize().expect("test folder path");
        Self { path }
    }

    pub fn write_sample(&self, name: &str) -> PathBuf {
        let path = self.path.join(name);
        fs::write(&path, get_synthetic_mzml()).expect("synthetic mzML");
        path
    }
}

impl Drop for TestFolder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub struct Server {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    next_id: u64,
}

impl Server {
    pub fn start(folder: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_quantion-mcp"))
            .arg("--allow")
            .arg(folder)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("server starts");
        let input = child.stdin.take().expect("server input");
        let output = BufReader::new(child.stdout.take().expect("server output"));
        let mut server = Self {
            child,
            input,
            output,
            next_id: 1,
        };
        let reply = server.request("initialize", json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "test", "version": "1" } }));
        assert_eq!(reply["result"]["protocolVersion"], "2025-06-18");
        server.send_line(
            &json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }).to_string(),
        );
        server
    }

    pub fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send_line(
            &json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }).to_string(),
        );
        let reply = self.read_reply();
        assert_eq!(reply["id"], id, "reply to the right request");
        reply
    }

    pub fn call_tool(&mut self, name: &str, arguments: Value) -> (Value, bool) {
        let reply = self.request(
            "tools/call",
            json!({ "name": name, "arguments": arguments }),
        );
        let result = &reply["result"];
        let text = result["content"][0]["text"].as_str().expect("text content");
        let output: Value = serde_json::from_str(text).expect("tool output is JSON");
        (output, result["isError"].as_bool().expect("isError"))
    }

    pub fn send_line(&mut self, line: &str) {
        writeln!(self.input, "{line}").expect("send to server");
        self.input.flush().expect("flush to server");
    }

    pub fn send_bytes(&mut self, bytes: &[u8]) {
        self.input.write_all(bytes).expect("send to server");
        self.input.flush().expect("flush to server");
    }

    pub fn read_reply(&mut self) -> Value {
        let mut line = String::new();
        self.output.read_line(&mut line).expect("read from server");
        serde_json::from_str(&line).unwrap_or_else(|_| panic!("stdout line is not JSON: {line}"))
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn get_synthetic_mzml() -> String {
    let mut spectra = String::new();
    for index in 0..SCAN_COUNT {
        let rt = FIRST_RT + index as f64 * SCAN_STEP;
        let peak = PEAK_HEIGHT * (-0.5 * ((rt - PEAK_RT) / PEAK_WIDTH).powi(2)).exp();
        let mz = [150.05, PEAK_MZ, 450.2];
        let intensity = [1000.0 + (index % 7) as f64 * 10.0, 50.0 + peak, 2000.0];
        spectra.push_str(&get_spectrum(index, rt, &mz, &intensity));
    }
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<mzML xmlns="http://psi.hupo.org/ms/mzml" version="1.1.0">
  <cvList count="2">
    <cv id="MS" fullName="Proteomics Standards Initiative Mass Spectrometry Ontology" URI="https://raw.githubusercontent.com/HUPO-PSI/psi-ms-CV/master/psi-ms.obo"/>
    <cv id="UO" fullName="Unit Ontology" URI="https://raw.githubusercontent.com/bio-ontology-research-group/unit-ontology/master/unit.obo"/>
  </cvList>
  <run id="synthetic">
    <spectrumList count="{SCAN_COUNT}">
{spectra}    </spectrumList>
  </run>
</mzML>
"#
    )
}

fn get_spectrum(index: usize, rt: f64, mz: &[f64], intensity: &[f64]) -> String {
    format!(
        r#"      <spectrum index="{index}" id="scan={scan}" defaultArrayLength="{length}">
        <cvParam cvRef="MS" accession="MS:1000511" name="ms level" value="1"/>
        <cvParam cvRef="MS" accession="MS:1000130" name="positive scan" value=""/>
        <scanList count="1">
          <scan>
            <cvParam cvRef="MS" accession="MS:1000016" name="scan start time" value="{rt}" unitCvRef="UO" unitAccession="UO:0000031" unitName="minute"/>
          </scan>
        </scanList>
        <binaryDataArrayList count="2">
{mz_array}{intensity_array}        </binaryDataArrayList>
      </spectrum>
"#,
        scan = index + 1,
        length = mz.len(),
        mz_array = get_binary_array(mz, "MS:1000514", "m/z array"),
        intensity_array = get_binary_array(intensity, "MS:1000515", "intensity array"),
    )
}

fn get_binary_array(values: &[f64], accession: &str, name: &str) -> String {
    let bytes: Vec<u8> = values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect();
    let encoded = encode_base64(&bytes);
    format!(
        r#"          <binaryDataArray encodedLength="{length}">
            <cvParam cvRef="MS" accession="MS:1000523" name="64-bit float" value=""/>
            <cvParam cvRef="MS" accession="MS:1000576" name="no compression" value=""/>
            <cvParam cvRef="MS" accession="{accession}" name="{name}" value=""/>
            <binary>{encoded}</binary>
          </binaryDataArray>
"#,
        length = encoded.len(),
    )
}

fn encode_base64(bytes: &[u8]) -> String {
    const LETTERS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut text = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for group in bytes.chunks(3) {
        let first = group[0] as u32;
        let second = group.get(1).copied().unwrap_or(0) as u32;
        let third = group.get(2).copied().unwrap_or(0) as u32;
        let number = (first << 16) | (second << 8) | third;
        text.push(LETTERS[(number >> 18) as usize & 63] as char);
        text.push(LETTERS[(number >> 12) as usize & 63] as char);
        text.push(if group.len() > 1 {
            LETTERS[(number >> 6) as usize & 63] as char
        } else {
            '='
        });
        text.push(if group.len() > 2 {
            LETTERS[number as usize & 63] as char
        } else {
            '='
        });
    }
    text
}
