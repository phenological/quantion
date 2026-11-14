use std::io::{self, BufRead, Read, Write};

use serde_json::Value;

const MAX_LINE_BYTES: u64 = 64 * 1024 * 1024;

pub enum Line {
    Text(String),
    Unreadable(&'static str),
}

pub fn read_line<R: BufRead>(input: &mut R) -> io::Result<Option<Line>> {
    let mut bytes = Vec::new();
    let count = Read::take(&mut *input, MAX_LINE_BYTES).read_until(b'\n', &mut bytes)?;
    if count == 0 {
        return Ok(None);
    }
    if !bytes.ends_with(b"\n") && count as u64 == MAX_LINE_BYTES {
        skip_rest_of_line(input)?;
        return Ok(Some(Line::Unreadable("the message is longer than 64 MB")));
    }
    while bytes
        .last()
        .is_some_and(|byte| *byte == b'\n' || *byte == b'\r')
    {
        bytes.pop();
    }
    match String::from_utf8(bytes) {
        Ok(text) => Ok(Some(Line::Text(text))),
        Err(_) => Ok(Some(Line::Unreadable("the message is not UTF-8 text"))),
    }
}

fn skip_rest_of_line<R: BufRead>(input: &mut R) -> io::Result<()> {
    loop {
        let buffer = input.fill_buf()?;
        if buffer.is_empty() {
            return Ok(());
        }
        match buffer.iter().position(|byte| *byte == b'\n') {
            Some(position) => {
                input.consume(position + 1);
                return Ok(());
            }
            None => {
                let length = buffer.len();
                input.consume(length);
            }
        }
    }
}

pub fn write_message(message: &Value) -> io::Result<()> {
    let mut output = io::stdout().lock();
    serde_json::to_writer(&mut output, message)?;
    output.write_all(b"\n")?;
    output.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_text_and_bad_bytes() {
        let mut input: &[u8] = b"{\"a\":1}\r\n\xff\xfe\n";
        assert!(
            matches!(read_line(&mut input).unwrap(), Some(Line::Text(text)) if text == "{\"a\":1}")
        );
        assert!(matches!(
            read_line(&mut input).unwrap(),
            Some(Line::Unreadable(_))
        ));
        assert!(read_line(&mut input).unwrap().is_none());
    }
}
