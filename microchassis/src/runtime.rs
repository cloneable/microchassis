use crate::error::{RuntimeError, ShutdownError};
use futures::future::BoxFuture;
use opentelemetry::{runtime::Tokio, sdk::trace::TraceRuntime};
use opentelemetry_sdk::runtime::Runtime as OTelRuntime;
use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

#[derive(Clone)]
pub struct Runtime {
    inner: Arc<Inner>,
}

impl Runtime {
    pub fn new() -> Result<Self, RuntimeError> {
        Ok(Runtime { inner: Inner::new()? })
    }

    pub fn shutdown(self) -> Result<(), ShutdownError> {
        // TODO: signal task+thread
        let inner = self.inner;
        match Arc::try_unwrap(inner) {
            Ok(inner) => inner.shutdown(),
            Err(_inner) => Ok(()), //panic!("runtime singleton"),
        }
    }
}

struct Inner {
    otel_runtime: Tokio,
    tokio_runtime: tokio::runtime::Runtime,
    handle: Mutex<Option<JoinHandle<Result<(), RuntimeError>>>>,
}

impl Inner {
    pub fn new() -> Result<Arc<Self>, RuntimeError> {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;

        let inner =
            Arc::new(Inner { otel_runtime: Tokio, tokio_runtime: rt, handle: Mutex::new(None) });
        let inner_thread = Arc::clone(&inner);
        let inner_task = Arc::clone(&inner);

        let handle = thread::Builder::new().name("microchassis".to_owned()).spawn(move || {
            inner_thread.tokio_runtime.block_on(async move {
                tracing::debug!("runtime starting");
                let ret = inner_task.run().await;
                tracing::debug!(ret = ?ret, "runtime terminated");
                ret
            })
        })?;

        *inner.handle.lock().expect("mutex locked") = Some(handle);

        Ok(inner)
    }

    #[allow(clippy::unused_async)]
    async fn run(&self) -> Result<(), RuntimeError> {
        // TODO: run signal handler
        // TODO: run http server
        Ok(())
    }

    pub fn shutdown(self) -> Result<(), ShutdownError> {
        let handle = self.handle.lock().expect("mutex lock").take();
        handle.expect("runtime handle").join().expect("thread join").map_err(Into::into)
    }
}

#[allow(clippy::empty_drop)]
impl Drop for Inner {
    fn drop(&mut self) {
        // TODO: terminate
    }
}

impl TraceRuntime for Runtime {
    type Receiver = <Tokio as TraceRuntime>::Receiver;
    type Sender = <Tokio as TraceRuntime>::Sender;

    #[inline]
    fn batch_message_channel(&self, capacity: usize) -> (Self::Sender, Self::Receiver) {
        self.inner.otel_runtime.batch_message_channel(capacity)
    }
}

impl OTelRuntime for Runtime {
    type Interval = <Tokio as OTelRuntime>::Interval;
    type Delay = <Tokio as OTelRuntime>::Delay;

    #[inline]
    fn interval(&self, duration: Duration) -> Self::Interval {
        self.inner.otel_runtime.interval(duration)
    }

    #[inline]
    fn delay(&self, duration: Duration) -> Self::Delay {
        self.inner.otel_runtime.delay(duration)
    }

    #[inline]
    fn spawn(&self, future: BoxFuture<'static, ()>) {
        // Run task in our sidecar thread and tokio runtime.
        let _handle = self.inner.tokio_runtime.spawn(future);
    }
}
