# HTTP server from raw TCP

This is a small HTTP/1.0 and HTTP/1.1 server written with Rust's standard
library. It does not use an HTTP framework or parser. The point of the project
is to show where HTTP begins: as bytes read from a TCP connection.

## From TCP to HTTP

`TcpListener` binds to `127.0.0.1:8080` and accepts incoming connections. Each
`TcpStream` is handed to a worker thread and wrapped in a `BufReader`. From
there, the server reads the request in three parts:

1. The request line, such as `GET /hello/sam HTTP/1.1`.
2. Header lines, ending at the first empty line.
3. The body, when a valid `Content-Length` is present.

Those bytes become a `Request` containing the method, path, query parameters,
headers, and body. The router passes it to a handler, which returns a
`Response`. Finally, the response is turned back into the HTTP wire format and
written to the same TCP stream:

```text
HTTP/1.1 200 OK\r\n
Content-Type: text/plain; charset=utf-8\r\n
Content-Length: 12\r\n
Connection: keep-alive\r\n
\r\n
Hello, sam!
```

One connection can carry more than one request. The same `BufReader` is kept
alive so any bytes already buffered for the next request are not lost. The
connection closes when the client asks for it, HTTP/1.0 defaults to it, or a
parse error occurs.

The parser also puts limits on header and body size, requires `Host` for
HTTP/1.1, and rejects transfer encoding because request bodies currently use
`Content-Length` only.

## Run it

```sh
cargo run
```

Then send a normal request:

```sh
curl 'http://127.0.0.1:8080/hello/sam?greeting=Hi'
```

Or send the HTTP bytes yourself:

```sh
printf 'GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n' \
  | nc 127.0.0.1 8080
```

The main pieces are in `src/server/connection.rs` (TCP connection lifecycle),
`src/http/parser.rs` (bytes to `Request`), and `src/http/response.rs`
(`Response` to bytes).
