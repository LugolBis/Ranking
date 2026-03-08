use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use crossbeam_queue::SegQueue;
use mylog::error;

#[derive(Debug)]
pub struct Worker {
    /// Random ID to distinguish different workers.
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

pub enum Job {
    Task(Box<dyn FnOnce() -> Result<(), Box<dyn std::error::Error>> + Send + 'static>),
    Shutdown,
}

#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    /// The shared queue for distributing jobs to workers.
    job_queue: Arc<SegQueue<Job>>,
    /// Notifier for workers when new jobs are available.
    job_signal: Arc<(Mutex<bool>, Condvar)>,
    /// Used to stop the workers.
    running: Arc<AtomicBool>,
}

#[derive(Debug)]
pub enum ThreadPoolError {
    ShutdownTimeout,
    ThreadJoinError(String),
}

impl Worker {
    fn new(
        id: usize,
        job_queue: Arc<SegQueue<Job>>,
        job_signal: Arc<(Mutex<bool>, Condvar)>,
        running: Arc<AtomicBool>,
    ) -> Worker {
        let thread = thread::spawn(move || {
            while running.load(Ordering::Relaxed) || !job_queue.is_empty() {
                match job_queue.pop() {
                    Some(Job::Task(task)) => if let Err(_) = task() {},
                    Some(Job::Shutdown) => break,
                    None => {
                        if !running.load(Ordering::Relaxed) {
                            break;
                        }
                        let (lock, cvar) = &*job_signal;
                        let mut job_available = lock.lock().unwrap();
                        while !*job_available && running.load(Ordering::Relaxed) {
                            job_available = cvar
                                .wait_timeout(job_available, Duration::from_millis(100))
                                .unwrap()
                                .0;
                        }
                        *job_available = false;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let job_queue = Arc::new(SegQueue::new());
        let job_signal = Arc::new((Mutex::new(false), Condvar::new()));
        let mut workers = Vec::with_capacity(size);
        let running = Arc::new(AtomicBool::new(true));

        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&job_queue),
                Arc::clone(&job_signal),
                Arc::clone(&running),
            ));
        }

        ThreadPool {
            workers,
            job_queue,
            job_signal,
            running,
        }
    }

    pub fn execute<F>(&self, func: F) -> Result<(), ThreadPoolError>
    where
        F: FnOnce() -> Result<(), Box<dyn std::error::Error>> + Send + 'static,
    {
        // Create a new Job::Task by wrapping the closure 'func'
        let job = Job::Task(Box::new(func));

        // Push this job to the queue
        self.job_queue.push(job);

        // Signal there is a new job is available
        let (lock, cvar) = &*self.job_signal;
        let mut job_available = lock.lock().unwrap();
        *job_available = true;
        cvar.notify_all();
        Ok(())
    }

    pub fn shutdown(&mut self, timeout: Duration) -> Result<(), ThreadPoolError> {
        let start = Instant::now();
        // 1 : Signal all workers to stop.
        for _ in 0..self.workers.len() {
            self.job_queue.push(Job::Shutdown);
        }

        self.running.store(false, Ordering::SeqCst);

        // 2 : Wake up all waiting threads.
        let (lock, cvar) = &*self.job_signal;
        match lock.try_lock() {
            Ok(mut job_available) => {
                *job_available = true;
                cvar.notify_all();
            }
            Err(_) => {
                error!(
                    "Warning: Couldn't acquire lock to notify workers. They will exit on their next timeout check."
                );
            }
        }

        // 3 : Wait for all workers to finish.
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                // 4 : Calculate remaining time.
                let remaining = timeout
                    .checked_sub(start.elapsed())
                    .unwrap_or(Duration::ZERO);

                // 5 : Check if we've exceeded the timeout.
                if remaining.is_zero() {
                    return Err(ThreadPoolError::ShutdownTimeout);
                }

                // 6 : Wait for the worker to finish.
                if thread.join().is_err() {
                    return Err(ThreadPoolError::ThreadJoinError(format!(
                        "Worker {} failed to join",
                        worker.id
                    )));
                }
            }
        }
        // 7 : Final timeout check.
        if start.elapsed() > timeout {
            Err(ThreadPoolError::ShutdownTimeout)
        } else {
            Ok(())
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if !self.workers.is_empty() {
            let _ = self.shutdown(Duration::from_secs(2));
        }
    }
}
