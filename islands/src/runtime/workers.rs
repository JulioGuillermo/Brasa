//! A few threads for slow work (decoding pictures, reading files, running commands), so the GTK thread
//! never waits. A job sends what it produced back as an Event.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

const WORKER_THREADS: usize = 2;

type Job = Box<dyn FnOnce() + Send + 'static>;

static QUEUE: OnceLock<Mutex<Sender<Job>>> = OnceLock::new();

pub fn start() {
    let (sender, receiver) = mpsc::channel::<Job>();
    let receiver = Arc::new(Mutex::new(receiver));
    for number in 0..WORKER_THREADS {
        let receiver = Arc::clone(&receiver);
        thread::Builder::new()
            .name(format!("worker-{number}"))
            .spawn(move || work(&receiver))
            .expect("a worker thread");
    }
    QUEUE.set(Mutex::new(sender)).ok();
}

pub fn run(job: impl FnOnce() + Send + 'static) {
    let queue = QUEUE.get().expect("workers::start() before workers::run()");
    let _ = queue.lock().unwrap().send(Box::new(job));
}

fn work(receiver: &Mutex<Receiver<Job>>) {
    loop {
        let job = receiver.lock().unwrap().recv();
        match job {
            Ok(job) => job(),
            Err(_) => return,
        }
    }
}
