//transport layer : bind, accept , hand each connection ot the pool


mod connection;
mod pool;

use std::io;
use std::net::{SocketAddr,TcpListener};
use std::sync::Arc;

use crate::{config::Config, router::Router};
use pool::ThreadPool;

pub struct Server{
    listener: TcpListener,
    config: Arc<Config>,
    router: Arc<Router>,
}

impl Server{
    pub fn bind(config:Config, router:Router) -> io::Result<Self>{
        let listener = TcpListener::bind(&config.addr)?;
        Ok(Server{
            listener,
            config: Arc::new(config),router:Arc::new(router)
        })
    }
    pub fn local_addr(&self)->io::Result<SocketAddr>{
        self.listener.local_addr()
    }
    pub fn run(self) -> io::Result<()>{
        let pool = ThreadPool::new(self.config.workers);
        for stream in self.listener.incoming(){
            match stream{
                Ok(stream)=>{
                    let router = Arc::clone(&self.router);
                    let config= Arc::clone(&self.config);
                    pool.execute(move ||{
                        if let Err(e) = connection::handle(stream,&router,&config){
                            eprintln!("connection error:{e}");
                        }
                    });
                }
                Err(e)=>eprintln!("accept error:{e}"),
            }
        }
        Ok(())
    }
}
