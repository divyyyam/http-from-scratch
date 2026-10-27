/// One Tcp connection : flow being parse -> route -> respond, repeated while keep-alive
use std::io::{self, BufReader};
use std::net::TcpStream;

use crate::config::Config;
use crate::http::{ParseError, parse_request};
use crate::router::Router;

pub fn handle(stream: TcpStream, router: &Router, config: &Config) -> io::Result<()> {
    stream.set_read_timeout(Some(config.read_timeout))?;
    stream.set_nodelay(true)?;
    let peer = stream.peer_addr()?;
    let mut writer = stream.try_clone()?;

    //one bufreader per connection: buffered bytes past current stream belong to the next
    //pipelined request
    let mut reader = BufReader::new(stream);
    loop {
        let req = match parse_request(&mut reader, &config.limits) {
            Ok(r) => r,
            Err(e) => {
                if let ParseError::Io(err) = &e {
                    eprintln!("{peer} io error:{err}");
                }
                if let Some(resp) = e.to_response() {
                    eprintln!("{peer}->{}({e:?})", resp.status);
                    let _ = resp.write_to(&mut writer, false, true);
                }
                return Ok(());
            }
        };
        let keep_alive: bool = req.wants_keep_alive();
        let resp = router.handle(&req);
        println!("{peer}{}{}->{}", req.method, req.path, resp.status);
        resp.write_to(&mut writer, keep_alive, req.method != "HEAD")?;
        if !keep_alive {
            return Ok(());
        }
    }
}
