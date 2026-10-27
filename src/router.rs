use crate::http::{Request, Response};

pub type Handler = fn(&Request, &[&str]) -> Response;

#[derive(Default)]
pub struct Router {
    routes: Vec<(&'static str, Vec<&'static str>, Handler)>,
}

impl Router {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn route(mut self, method: &'static str, pattern: &'static str, h: Handler) -> Self {
        self.routes.push((method, segments(pattern), h));
        self
    }

    pub fn handle(&self, req: &Request) -> Response {
        let segs = segments(&req.path);
        let mut allowed = Vec::new();
        for (method, pattern, handler) in &self.routes {
            if let Some(params) = match_path(pattern, &segs) {
                // HEAD is served by the GET handler; the connection drops the body.
                if *method == req.method || (*method == "GET" && req.method == "HEAD") {
                    return handler(req, &params);
                }
                allowed.push(*method);
            }
        }
        if allowed.is_empty() {
            Response::text(404, "not found\n")
        } else {
            Response::text(405, "method not allowed\n").header("Allow", &allowed.join(", "))
        }
    }
}

fn segments(path: &str) -> Vec<&str> {
    path.split('/').filter(|s| !s.is_empty()).collect()
}

fn match_path<'a>(pattern: &[&str], segs: &[&'a str]) -> Option<Vec<&'a str>> {
    if pattern.len() != segs.len() {
        return None;
    }
    let mut params = Vec::new();
    for (p, s) in pattern.iter().zip(segs) {
        if p.starts_with('{') && p.ends_with('}') {
            params.push(*s);
        } else if p != s {
            return None;
        }
    }
    Some(params)
}

