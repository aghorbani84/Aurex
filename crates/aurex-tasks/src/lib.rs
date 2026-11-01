//! Bounded blocking-I/O executor with awaitable results, backpressure and cooperative cancellation.
//! No filesystem work is polled on the UI thread. Workers survive panicking jobs.
use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc,
    },
    task::{Context, Poll, Waker},
    thread,
};

#[derive(Clone, Default)]
pub struct Cancellation(Arc<AtomicBool>);
impl Cancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
#[derive(Clone, Default)]
pub struct Progress {
    bytes: Arc<AtomicU64>,
    files: Arc<AtomicU64>,
}
impl Progress {
    pub fn add_bytes(&self, bytes: u64) {
        self.bytes.fetch_add(bytes, Ordering::Relaxed);
    }
    pub fn finish_file(&self) {
        self.files.fetch_add(1, Ordering::Relaxed);
    }
    pub fn snapshot(&self) -> (u64, u64) {
        (
            self.bytes.load(Ordering::Relaxed),
            self.files.load(Ordering::Relaxed),
        )
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleError {
    Full,
    Closed,
    Panicked,
}
impl std::fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "task executor: {self:?}")
    }
}
impl std::error::Error for ScheduleError {}

type Job = Box<dyn FnOnce() + Send>;
#[derive(Clone)]
pub struct Scheduler {
    sender: mpsc::SyncSender<Job>,
}
struct State<T> {
    result: Option<std::result::Result<T, ScheduleError>>,
    waker: Option<Waker>,
}
struct Shared<T> {
    state: Mutex<State<T>>,
    ready: Condvar,
}
pub struct Task<T> {
    shared: Arc<Shared<T>>,
}
impl Scheduler {
    pub fn new(workers: usize, queue: usize) -> Self {
        assert!((1..=32).contains(&workers), "worker limit must be 1..=32");
        assert!((1..=4096).contains(&queue), "queue limit must be 1..=4096");
        let (sender, receiver) = mpsc::sync_channel::<Job>(queue);
        let receiver = Arc::new(Mutex::new(receiver));
        for n in 0..workers {
            let receiver = receiver.clone();
            thread::Builder::new()
                .name(format!("aurex-io-{n}"))
                .spawn(move || {
                    loop {
                        let job = receiver.lock().expect("receiver lock poisoned").recv();
                        match job {
                            Ok(job) => job(),
                            Err(_) => break,
                        }
                    }
                })
                .expect("cannot start I/O worker");
        }
        Self { sender }
    }
    pub fn submit<T: Send + 'static>(
        &self,
        work: impl FnOnce() -> T + Send + 'static,
    ) -> Result<Task<T>, ScheduleError> {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                result: None,
                waker: None,
            }),
            ready: Condvar::new(),
        });
        let output = shared.clone();
        let job = Box::new(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work))
                .map_err(|_| ScheduleError::Panicked);
            let waker = {
                let mut state = output.state.lock().expect("task lock poisoned");
                state.result = Some(result);
                state.waker.take()
            };
            output.ready.notify_all();
            if let Some(waker) = waker {
                waker.wake();
            }
        });
        self.sender.try_send(job).map_err(|e| match e {
            mpsc::TrySendError::Full(_) => ScheduleError::Full,
            mpsc::TrySendError::Disconnected(_) => ScheduleError::Closed,
        })?;
        Ok(Task { shared })
    }
}
impl<T> Task<T> {
    /// For CLI/tests only; UI callers must `.await`.
    pub fn wait(self) -> Result<T, ScheduleError> {
        let mut state = self.shared.state.lock().expect("task lock poisoned");
        loop {
            if let Some(result) = state.result.take() {
                return result;
            }
            state = self.shared.ready.wait(state).expect("task lock poisoned");
        }
    }
}
impl<T> Future for Task<T> {
    type Output = Result<T, ScheduleError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.shared.state.lock().expect("task lock poisoned");
        match state.result.take() {
            Some(result) => Poll::Ready(result),
            None => {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runs_and_recovers_from_panic() {
        let pool = Scheduler::new(2, 4);
        assert_eq!(pool.submit(|| 42).unwrap().wait(), Ok(42));
        assert_eq!(
            pool.submit(|| panic!("controlled test")).unwrap().wait(),
            Err(ScheduleError::Panicked)
        );
        assert_eq!(pool.submit(|| 7).unwrap().wait(), Ok(7));
    }
    #[test]
    fn bounded_queue_rejects_excess_work() {
        let pool = Scheduler::new(1, 1);
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let a = pool
            .submit(move || {
                started_tx.send(()).unwrap();
                release_rx.recv().unwrap();
            })
            .unwrap();
        started_rx.recv().unwrap();
        let b = pool.submit(|| 2).unwrap();
        assert!(matches!(pool.submit(|| 3), Err(ScheduleError::Full)));
        release_tx.send(()).unwrap();
        a.wait().unwrap();
        assert_eq!(b.wait(), Ok(2));
    }
    #[test]
    fn cancellation_and_progress_are_shared() {
        let token = Cancellation::default();
        let other = token.clone();
        other.cancel();
        assert!(token.is_cancelled());
        let progress = Progress::default();
        progress.clone().add_bytes(15);
        progress.finish_file();
        assert_eq!(progress.snapshot(), (15, 1));
    }
    #[test]
    fn future_wakes_without_blocking() {
        use std::task::{Wake, Waker};
        struct WakeThread(thread::Thread);
        impl Wake for WakeThread {
            fn wake(self: Arc<Self>) {
                self.0.unpark();
            }
        }
        let pool = Scheduler::new(1, 2);
        let (release, gate) = mpsc::channel();
        let mut task = Box::pin(
            pool.submit(move || {
                gate.recv().unwrap();
                9
            })
            .unwrap(),
        );
        let waker = Waker::from(Arc::new(WakeThread(thread::current())));
        let mut cx = Context::from_waker(&waker);
        assert!(task.as_mut().poll(&mut cx).is_pending());
        release.send(()).unwrap();
        loop {
            match task.as_mut().poll(&mut cx) {
                Poll::Ready(value) => {
                    assert_eq!(value, Ok(9));
                    break;
                }
                Poll::Pending => thread::park(),
            }
        }
    }
}
