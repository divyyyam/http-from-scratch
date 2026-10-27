pub fn reason(code: u16) -> &'static str {
    match code {
        200 => "OK",
        201 => "CREATED",
        400 => "Bad Request",
        404 => "Not found",
        405 => "Method not allowed",
        408 => "Request Timeout",
        413 => "Payload too large",
        431 => "Request header fields too large",
        500 => "Internal server error",
        501 => "Not Implemented",
        505 => "HTTP version not supported",
        _ => "Unknown",
    }
}
