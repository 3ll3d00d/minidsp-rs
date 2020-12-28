use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use hidapi::HidError;
use std::ops::DerefMut;
use thiserror::Error;
use tokio::sync::{broadcast, OwnedMutexGuard};
pub mod hid;
pub mod net;

#[derive(Error, Debug)]
pub enum MiniDSPError {
    #[error("An HID error has occurred: {0}")]
    HIDError(#[from] HidError),

    #[error("A malformed packet was received")]
    MalformedResponse,
}

impl<T> Sender for OwnedMutexGuard<T>
where
    T: Sender,
{
    fn send(&mut self, frame: Bytes) -> Result<(), MiniDSPError> {
        // self.deref_mut().send() confuses clion
        T::send(self.deref_mut(), frame)
    }
}

/// Transport trait implemented by different backends
#[async_trait]
pub trait Transport: Send + Sync {
    // Subscribe to all received frames
    fn subscribe(&self) -> broadcast::Receiver<Bytes>;

    // Acquire an exclusive lock for sending frames on this device
    async fn send_lock(&self) -> Box<dyn Sender>;

    // Sends a single frame
    async fn send(&self, frame: Bytes) -> Result<(), MiniDSPError> {
        let mut tx = self.send_lock().await;
        tx.send(frame)
    }
}

pub trait Sender: Send + Sync {
    fn send(&mut self, frame: Bytes) -> Result<(), MiniDSPError>;
}
