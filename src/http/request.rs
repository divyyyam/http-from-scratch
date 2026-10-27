use std::collections::HashMap;

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .get(&name.to_ascii_lowercase())
            .map(String::as_str)
    }
    ///HTTP/1.1 defaults to keep-alive; HTTP/1.0 defaults to close;
    pub fn wants_keep_alive(&self) -> bool {
        match self.header("connection").map(|v| v.to_ascii_lowercase()) {
            Some(v) if v == "close" => false,
            Some(v) if v == "keep-alive" => true,
            _ => self.version == "HTTP/1.1",
        }
    }
}
