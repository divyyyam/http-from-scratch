use std::io::{self, Write};

use super::status::reason;

pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status: u16) -> Self {
        Response { status, headers: Vec::new(), body: Vec::new() }
    }

    pub fn header(mut self, k: &str, v: &str) -> Self {
        self.headers.push((k.to_string(), v.to_string()));
        self
    }

    pub fn body(mut self, b: impl Into<Vec<u8>>) -> Self {
        self.body = b.into();
        self
    }

    pub fn text(status: u16, s: &str) -> Self {
        Response::new(status).header("Content-Type", "text/plain; charset=utf-8").body(s)
    }

    pub fn write_to(&self, w: &mut impl Write, keep_alive: bool, send_body: bool) -> io::Result<()> {
        let mut head = format!("HTTP/1.1 {} {}\r\n", self.status, reason(self.status));
        for (k, v) in &self.headers {
            head.push_str(&format!("{k}: {v}\r\n"));
        }
        head.push_str(&format!("Content-Length: {}\r\n", self.body.len()));
        head.push_str(if keep_alive { "Connection: keep-alive\r\n" } else { "Connection: close\r\n" });
        head.push_str("\r\n");
        w.write_all(head.as_bytes())?;
        if send_body {
            w.write_all(&self.body)?;
        }
        w.flush()
    }
}
