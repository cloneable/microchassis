extern crate alloc;

use crate::error::RuntimeError;
use alloc::sync::Arc;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};

pub struct Runtime {
    handle: Mutex<Option<JoinHandle<Result<(), RuntimeError>>>>,
}

impl Runtime {
    pub fn new() -> Arc<Self> {
        Arc::new(Runtime { handle: Mutex::new(None) })
    }

    pub fn start(self: &Arc<Self>) -> Result<(), RuntimeError> {
        let self_clone = Arc::clone(self);
        let _unused = self.handle.lock().expect("poisened mutex").insert(
            thread::Builder::new().name("microchassis".to_owned()).spawn(move || {
                tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(
                    async move {
                        tracing::debug!("runtime starting");
                        let ret = self_clone.run().await;
                        tracing::debug!(ret = ?ret, "runtime terminated");
                        ret
                    },
                )
            })?,
        );
        Ok(())
    }

    #[allow(clippy::unused_async)]
    async fn run(self: &Arc<Self>) -> Result<(), RuntimeError> {
        // TODO: run signal handler
        // TODO: run http server
        Ok(())
    }

    pub fn stop(self) -> Result<(), RuntimeError> {
        let handle = self.handle.lock().expect("poisened mutex").take();
        handle.expect("unstarted runtime").join().expect("thread error")
    }
}

#[allow(clippy::empty_drop)]
impl Drop for Runtime {
    fn drop(&mut self) {
        // TODO: terminate
    }
}
