//fixed sze worker pool fed by an mpsc channel

use std::sync::{Arc, Mutex, mpsc};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    sender: Option<mpsc::Sender<Job>>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "pool needs atleast one worker");
        let (tx, rx) = mpsc::channel::<Job>();
        let rx = Arc::new(Mutex::new(rx));
        let workers = (0..size)
            .map(|id| {
                let rx = Arc::clone(&rx);
                thread::Builder::new()
                    .name(format!("worker-{id}"))
                    .spawn(move || {
                        loop {
                            let job = rx.lock().unwrap().recv();
                            match job {
                                Ok(job) => job(),
                                Err(_) => break,
                            }
                        }
                    })
                    .expect("spawn worker")
            })
            .collect();
        ThreadPool {
            sender: Some(tx),
            workers,
        }
    }
    pub fn execute(&self, f: impl FnOnce() + Send + 'static) {
        self.sender
            .as_ref()
            .unwrap()
            .send(Box::new(f))
            .expect("pool closeed");
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for w in self.workers.drain(..) {
            let _ = w.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[test]
    fn runs_all_jobs_before_drop_returns() {
        let count = Arc::new(AtomicUsize::new(0));
        {
            let pool: ThreadPool = ThreadPool::new(4);
            for _ in 0..100 {
                let c = Arc::clone(&count);
                pool.execute(move || {
                    c.fetch_add(1, Ordering::SeqCst);
                })
            }
        }
        assert_eq!(count.load(Ordering::SeqCst), 100);
    }
}
