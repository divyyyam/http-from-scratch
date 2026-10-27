
use http_server::{handlers, Config, Router, Server};

fn main() -> std::io::Result<()> {
    let config = Config::from_env();
    let router = Router::new()
        .route("GET", "/", handlers::index)
        .route("GET", "/hello/{name}", handlers::greet)
        .route("POST", "/echo", handlers::echo)
        .route("GET", "/debug", handlers::debug);

    let server = Server::bind(config, router)?;
    println!("listening on http://{}", server.local_addr()?);
    server.run()
}
