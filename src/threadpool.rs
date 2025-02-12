use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
    is_running: bool,
}

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let num_workers = match size {
            0 => num_cpus::get(),
            _ => size,
        };

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(num_workers);
        for _ in 0..num_workers {
            workers.push(Worker::new(Arc::clone(&receiver)));
        }
        ThreadPool {
            workers,
            sender,
            is_running: true,
        }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if !self.is_running {
            panic!("ThreadPool isn't running.\nUsed ThreadPool::join() and then ThreadPool::execute().");
        }
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
    pub fn join(mut self) {
        self.is_running = false;
        drop(self)
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if !self.is_running {
            return;
        }
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        for worker in &mut self.workers {
            if let Some(thread) = worker.handle.take() {
                thread.join().unwrap();
            }
        }
    }
}
struct Worker {
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => {
                    job();
                }
                Message::Terminate => {
                    break;
                }
            }
        });
        Worker {
            handle: Some(thread),
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let pool = ThreadPool::new(4);
        let result: u8 = 0;
        let a: u8 = 1;
        let b: u8 = 3;
        pool.execute(move || {
            let result = a + b;
            assert_eq!(result, 4);
        });

        assert_eq!(result, 0);
    }

    #[test]
    fn join() {
        let mut pool = ThreadPool::new(4);
        let result: u8 = 0;
        let a: u8 = 1;
        let b: u8 = 3;
        pool.execute(move || {
            let result = a + b;
            assert_eq!(result, 4);
        });
        pool.join();

        assert_eq!(result, 0);
    }
}
