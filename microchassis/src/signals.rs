use crate::error::{InitError, ShutdownError};
use std::sync::Arc;
use tokio::{
    signal::{
        self,
        unix::{signal, SignalKind},
    },
    sync::broadcast::{self, Receiver, Sender},
};
use tracing as log;

pub(crate) fn init() -> Result<ShutdownBroadcast, InitError> {
    // TODO: support other OSes
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;
    let (signal_write_chan, signal_read_chan) = broadcast::channel(1);

    let signal_write_chan = Arc::new(signal_write_chan);
    tokio::spawn({
        let signal_write_chan = signal_write_chan.clone();
        async move {
            // TODO: remove loop
            loop {
                tokio::select! {
                    biased;
                    _ = sigterm.recv() => {
                        log::debug!("shutdown due to SIGTERM");
                        break;
                    }
                    _ = sigint.recv() => {
                        log::debug!("shutdown due to SIGINT");
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
            drop(signal_read_chan);
        }
    });

    Ok(ShutdownBroadcast(signal_write_chan))
}

pub(crate) fn shutdown() -> Result<(), ShutdownError> {
    Ok(())
}

pub struct ShutdownBroadcast(Arc<Sender<()>>);

impl ShutdownBroadcast {
    pub fn subscribe(&self) -> ShutdownReceiver {
        ShutdownReceiver(self.0.subscribe())
    }
}

pub struct ShutdownReceiver(Receiver<()>);

impl ShutdownReceiver {
    pub async fn recv(mut self) {
        match self.0.recv().await {
            Ok(_) => {}
            Err(_err) => {
                // TODO report error
            }
        }
    }
}
