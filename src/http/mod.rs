//This is the protocol layer for HTTP/1.1 this layer knows nothing about sockets or routing;
//it turns bytes into a Request and a Response into bytes

mod parser;
mod request;
mod response;
mod status;
mod url;

pub use parser::{parse_request, ParseError};
pub use request::Request;
pub use response::Response;
pub use status::reason;