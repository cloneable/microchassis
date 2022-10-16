use crate::error::{InitError, ShutdownError};
use std::time::Duration;
use tokio::{
    signal::{
        self,
        unix::{signal, SignalKind},
    },
    sync::broadcast::{self, Receiver},
};
use tracing as log;

pub(crate) fn init() -> Result<ShutdownSignal, InitError> {
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sighup = signal(SignalKind::hangup())?;
    let (signal_write_chan, signal_read_chan) = broadcast::channel(1);

    tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;
                // TODO: remove timeout
                _ = tokio::time::sleep(Duration::from_secs(600)) => {
                    log::debug!("shutdown due to timeout");
                    break;
                }
                _ = sigterm.recv() => {
                    log::debug!("shutdown due to SIGTERM");
                    break;
                }
                _ = sighup.recv() => {
                    log::debug!("shutdown due to SIGHUP");
                    break;
                }
                _ = signal::ctrl_c() => {
                    log::debug!("shutdown due to Ctrl-C");
                    break;
                }
            }
        }
        if let Err(err) = signal_write_chan.send(()) {
            log::error!("signal_write_chan.send(): {:?}", err);
        }
    });
    Ok(ShutdownSignal(signal_read_chan))
}

pub(crate) fn shutdown() -> Result<(), ShutdownError> {
    Ok(())
}

// TOOD: rewrite as Future
pub struct ShutdownSignal(Receiver<()>);

impl ShutdownSignal {
    pub async fn recv(mut self) {
        match self.0.recv().await {
            Ok(_) => {}
            Err(_err) => {
                // TODO report error
            }
        }
    }
}
