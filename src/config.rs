use std::time::Duration;

#[derive(Debug,Clone,Copy)]
pub struct Limits{
    pub max_header_bytes:usize,
    pub max_body_bytes:usize,
}
impl Default for Limits{
    fn default() -> Self{
        Limits{
            max_header_bytes:8*1024, max_body_bytes: 1024*1024
        }
    }
}
#[derive(Debug,Clone)]
pub struct Config{
    pub addr:String,
    pub workers: usize,
    pub read_timeout:Duration,
    pub limits: Limits,
}

impl Default for Config{
    fn default() -> Self{
        Config{
            addr:"127.0.0.1:8080".into(),
            workers:8,
            read_timeout:Duration::from_secs(5),
            limits:Limits::default(),
        }
    }
}
impl Config{
    pub fn from_env() -> Self{
        let mut c = Config::default();
        if let Ok(v) = std::env::var("ADDR"){
            c.addr=v;
        }
        if let Some(v) = env_parse("WORKERS"){
            c.workers =v;
        }
        if let Some(v) = env_parse("READ_TIMEOUT-SECS"){
            c.read_timeout = Duration::from_secs(v);
        }
        if let Some(v) = env_parse("MAX_HEADER_BYTES"){
            c.limits.max_header_bytes = v; 
        }
        if let Some(v) = env_parse("MAX_BODY_BYTES"){
            c.limits.max_body_bytes = v;
        }
        c
    }
}
fn env_parse<T:std::str::FromStr>(key: &str) -> Option<T>{
    std::env::var(key).ok()?.parse().ok()
}
