//Application code, handlers only see req and res never raw sockets

use crate::http::{Request,Response};

pub fn index(_:&Request,_:&[&str])->Response {
    Response::new(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .body("<h1>Hello from raw rust</h1>\n")
}
pub fn greet(req:&Request, params:&[&str]) -> Response{
    let greeting = req.query.get("greeting").map(String::as_str).unwrap_or("Hello");
    Response::text(200,&format!("{greeting},{}!\n",params[0]))
}

pub fn echo(req:&Request, _:&[&str]) -> Response{
    let ct = req.header("content-type").unwrap_or("application/octet-stream").to_string();
    Response::new(200).header("Content-Type",&ct).body(req.body.clone())
}

pub fn debug(req:&Request,_:&[&str]) -> Response{
    let mut s = format!("{}{}{}\n",req.method,req.path,req.version);
    let mut hs:Vec<_>= req.headers.iter().collect();
    hs.sort();
    for (k,v) in hs{
        s.push_str(&format!("{k}:{v}\n"));
    }
    Response::text(200,&s)
}
