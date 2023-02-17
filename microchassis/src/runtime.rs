use crate::error::RuntimeError;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

pub struct Runtime {
    handle: Mutex<Option<JoinHandle<Result<(), RuntimeError>>>>,
}

impl Runtime {
    pub fn new() -> Arc<Self> {
        Arc::new(Runtime { handle: Mutex::new(None) })
    }

    pub fn start(self: &Arc<Self>) -> Result<(), RuntimeError> {
        let self_clone = self.clone();
        let _unused = self.handle.lock().unwrap().insert(
            thread::Builder::new().name("microchassis".to_string()).spawn(move || {
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

impl Drop for Runtime {
    fn drop(&mut self) {
        // TODO: terminate
    }
}
