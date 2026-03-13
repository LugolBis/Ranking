use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use crossbeam_queue::SegQueue;

use crate::errors::{RefErr, ThreadPoolErr};

/// An abstraction of a thread which can interact with `ThreadPool` to execute `Job`.
#[derive(Debug)]
pub struct Worker {
    /// Random ID to distinguish different workers.
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
    error: Option<String>,
}

/// Represent a job that a `Work` need to do.
pub enum Job {
    Task(Box<dyn FnOnce() -> Result<(), RefErr> + Send + 'static>),
    Shutdown,
}

/// An abstraction to manage threads with ease.
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

impl Worker {
    /// This method create and start a Worker who will execute the `Job`s pushed in the `ThreadPool` until `ThreadPool.shutdown()` is called.
    fn new(
        id: usize,
        job_queue: Arc<SegQueue<Job>>,
        job_signal: Arc<(Mutex<bool>, Condvar)>,
        running: Arc<AtomicBool>,
    ) -> Worker {
        let error_shared = Arc::new(Mutex::new(None));
        let error_c = Arc::clone(&error_shared);

        let thread = thread::spawn(move || {
            while running.load(Ordering::Relaxed) || !job_queue.is_empty() {
                match job_queue.pop() {
                    Some(Job::Task(task)) => {
                        if let Err(e) = task() {
                            if let Ok(mut mutex) = error_c.lock() {
                                *mutex = Some(e.to_string());
                            }
                        }
                    }
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

        let error: Option<String>;
        if let Ok(mut lock) = error_shared.lock() {
            error = lock.take();
        } else {
            error = None;
        }

        Worker {
            id,
            thread: Some(thread),
            error,
        }
    }
}

impl ThreadPool {
    /// Create new thread pool with `size` workers (threads).
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

    /// Took a closure in input to be executed by a `Worker`.
    pub fn execute<F>(&self, func: F) -> Result<(), ThreadPoolErr>
    where
        F: FnOnce() -> Result<(), RefErr> + Send + 'static,
    {
        // Create a new Job::Task by wrapping the closure 'func'
        let job = Job::Task(Box::new(func));

        // Push this job to the queue
        self.job_queue.push(job);

        // Signal there is a new job is available
        let (lock, cvar) = &*self.job_signal;
        if let Ok(mut job_available) = lock.lock() {
            *job_available = true;
            cvar.notify_all();
            Ok(())
        } else {
            Err(ThreadPoolErr::JobSignal(
                "Failed to acquire the job signal lock".into(),
            ))
        }
    }

    /// Shutdown the thread pool after a given timeout.
    pub fn shutdown(&mut self, timeout: Duration) -> Result<(), ThreadPoolErr> {
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
                ThreadPoolErr::JobSignal(
                    "Warning: Couldn't acquire lock to notify workers. They will exit on their next timeout check.".into()
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
                    return Err(ThreadPoolErr::ShutdownTimeout);
                }

                // 6 : Wait for the worker to finish.
                if thread.join().is_err() {
                    return Err(ThreadPoolErr::ThreadJoin(format!(
                        "Worker {} failed to join",
                        worker.id
                    )));
                }

                // 7 : Check the task was successfully executed.
                if let Some(e) = &worker.error {
                    return Err(ThreadPoolErr::ThreadExec(format!(
                        "Worker {} encounter the following error during execution : {:?}",
                        worker.id, e
                    )));
                }
            }
        }
        // 8 : Final timeout check.
        if start.elapsed() > timeout {
            Err(ThreadPoolErr::ShutdownTimeout)
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
