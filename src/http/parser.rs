use std::collections::HashMap;
use std::io::{self, BufRead, Read};

use super::{Request, Response, url::split_target};

use crate::config::Limits;

#[derive(Debug)]
pub enum ParseError {
    Closed,  //client closed cleanly between requests
    Timeout, //client was idle
    Bad(&'static str),
    HeadersTooLarge,
    BodyTooLarge,
    Unsupported(&'static str),
    BadVersion,
    Io(io::Error),
}



impl ParseError{
    pub fn to_response(&self) -> Option<Response>{
        match self{
            ParseError::Closed | ParseError:: Io(_) | ParseError::Timeout => None,
            ParseError::Bad(m) => Some(Response::text(400,&format!("bad request: {m}\n"))),
            ParseError::HeadersTooLarge => Some(Response::text(431,"headers too large\n")),
            ParseError::BodyTooLarge => Some(Response::text(413,"body too large\n")),
            ParseError::Unsupported(m) => Some(Response::text(501,&format!("not implemented:{m}\n"))),
            ParseError::BadVersion => Some(Response::text(505,"only HTTP/1.0 and HTTP/1.1\n")),
        }
    }
}

impl From<io::Error> for ParseError{
    fn from(e:io::Error) -> Self{
        match e.kind(){
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => ParseError::Timeout,
            _=>ParseError::Io(e),
        }
    }
}


fn read_line(reader: &mut impl BufRead, budget: &mut usize) -> Result<String, ParseError> {
    let mut buf = Vec::new();
    let n = reader.take(*budget as u64 + 1).read_until(b'\n', &mut buf)?;
    if n == 0 {
        return Err(ParseError::Closed);
    }
    if n > *budget {
        return Err(ParseError::HeadersTooLarge);
    }
    *budget -= n;
    if !buf.ends_with(b"\n") {
        return Err(ParseError::Bad("unterminated line"));
    }
    buf.pop();
    if buf.ends_with(b"\r") {
        buf.pop();
    }
    String::from_utf8(buf).map_err(|_| ParseError::Bad("non-utf8 header"))
}

pub fn parse_request(reader: &mut impl BufRead, limits: &Limits) -> Result<Request, ParseError> {
    let mut budget = limits.max_header_bytes;

    // 1. Request line
    let mut line = read_line(reader, &mut budget)?;
    while line.is_empty() {
        line = read_line(reader, &mut budget)?;
    }
    let mut parts = line.split(' ');
    let (method, target, version) = match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(m), Some(t), Some(v), None) if !m.is_empty() && t.starts_with('/') => (m, t, v),
        _ => return Err(ParseError::Bad("malformed request line")),
    };
    if version != "HTTP/1.1" && version != "HTTP/1.0" {
        return Err(ParseError::BadVersion);
    }
    let (path, query) = split_target(target);

    // 2. Headers
    let mut headers: HashMap<String, String> = HashMap::new();
    loop {
        let line = match read_line(reader, &mut budget) {
            Err(ParseError::Closed) => return Err(ParseError::Bad("eof in headers")),
            r => r?,
        };
        if line.is_empty() {
            break;
        }
        let (name, value) = line.split_once(':').ok_or(ParseError::Bad("header without colon"))?;
        if name.is_empty() || name.contains(char::is_whitespace) {
            return Err(ParseError::Bad("invalid header name"));
        }
        let name = name.to_ascii_lowercase();
        let value = value.trim();
        if name == "content-length" && headers.contains_key("content-length") {
            return Err(ParseError::Bad("duplicate content-length"));
        }
        headers
            .entry(name)
            .and_modify(|v| {
                v.push_str(", ");
                v.push_str(value);
            })
            .or_insert_with(|| value.to_string());
    }

    if version == "HTTP/1.1" && !headers.contains_key("host") {
        return Err(ParseError::Bad("missing Host header"));
    }

    // 3. Body (Content-Length only)
    if headers.contains_key("transfer-encoding") {
        return Err(ParseError::Unsupported("transfer-encoding"));
    }
    let body = match headers.get("content-length") {
        None => Vec::new(),
        Some(v) => {
            let len: usize = v.parse().map_err(|_| ParseError::Bad("invalid content-length"))?;
            if len > limits.max_body_bytes {
                return Err(ParseError::BodyTooLarge);
            }
            let mut body = vec![0u8; len];
            reader.read_exact(&mut body).map_err(|e| match e.kind() {
                io::ErrorKind::UnexpectedEof => ParseError::Bad("body shorter than content-length"),
                _ => ParseError::from(e),
            })?;
            body
        }
    };

    Ok(Request {
        method: method.to_string(),
        path,
        query,
        version: version.to_string(),
        headers,
        body,
    })
}