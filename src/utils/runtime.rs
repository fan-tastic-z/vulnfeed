use std::future::Future;
use std::panic::resume_unwind;
use std::sync::Arc;
use std::task::ready;
use std::time::Duration;
use std::time::Instant;

pub fn make_runtime(runtime_name: &str, thread_name: &str, worker_threads: usize) -> Runtime {
    log::debug!(
        "creating runtime with runtime_name: {runtime_name}, thread_name: {thread_name}, work_threads: {worker_threads}"
    );

    Builder::new(runtime_name, thread_name)
        .worker_threads(worker_threads)
        .build()
        .expect("failed to create runtime")
}

#[derive(Debug, Clone)]
pub struct Runtime {
    name: String,
    runtime: Arc<tokio::runtime::Runtime>,
}

impl Runtime {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        JoinHandle::new(self.runtime.spawn(future))
    }

    pub fn spawn_blocking<F, R>(&self, func: F) -> JoinHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        JoinHandle::new(self.runtime.spawn_blocking(func))
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }
}

impl fastimer::Spawn for Runtime {
    fn spawn<F: Future<Output = ()> + Send + 'static>(&self, future: F) {
        Runtime::spawn(self, future);
    }
}

#[derive(Debug)]
struct Timer;

pub fn timer() -> impl fastimer::MakeDelay {
    Timer
}

impl fastimer::MakeDelay for Timer {
    type Delay = tokio::time::Sleep;

    fn delay_util(&self, at: Instant) -> Self::Delay {
        tokio::time::sleep_until(tokio::time::Instant::from_std(at))
    }

    fn delay(&self, duration: Duration) -> Self::Delay {
        tokio::time::sleep(duration)
    }
}

#[pin_project::pin_project]
#[derive(Debug)]
pub struct JoinHandle<R> {
    #[pin]
    inner: tokio::task::JoinHandle<R>,
}

impl<R> JoinHandle<R> {
    fn new(inner: tokio::task::JoinHandle<R>) -> Self {
        Self { inner }
    }
}

impl<R> Future for JoinHandle<R> {
    type Output = R;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        let val = ready!(this.inner.poll(cx));
        match val {
            Ok(val) => std::task::Poll::Ready(val),
            Err(err) => {
                if err.is_panic() {
                    resume_unwind(err.into_panic())
                } else {
                    unreachable!()
                }
            }
        }
    }
}

pub struct Builder {
    runtime_name: String,
    thread_name: String,
    builder: tokio::runtime::Builder,
}

impl Builder {
    pub fn new(runtime_name: impl Into<String>, thread_name: impl Into<String>) -> Self {
        Self {
            runtime_name: runtime_name.into(),
            thread_name: thread_name.into(),
            builder: tokio::runtime::Builder::new_multi_thread(),
        }
    }

    pub fn worker_threads(&mut self, val: usize) -> &mut Self {
        self.builder.worker_threads(val);
        self
    }

    pub fn max_blocking_threads(&mut self, val: usize) -> &mut Self {
        self.builder.max_blocking_threads(val);
        self
    }

    pub fn thread_keep_alive(&mut self, duration: Duration) -> &mut Self {
        self.builder.thread_keep_alive(duration);
        self
    }

    pub fn runtime_name(&mut self, val: impl Into<String>) -> &mut Self {
        self.runtime_name = val.into();
        self
    }

    pub fn thread_name(&mut self, val: impl Into<String>) -> &mut Self {
        self.thread_name = val.into();
        self
    }

    pub fn build(&mut self) -> std::io::Result<Runtime> {
        let name = self.runtime_name.clone();
        let runtime = self
            .builder
            .enable_all()
            .thread_name(self.thread_name.clone())
            .build()
            .map(Arc::new)?;
        Ok(Runtime { name, runtime })
    }
}
