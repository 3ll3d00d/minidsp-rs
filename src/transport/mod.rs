//! Transport base traits for talking to devices

//! Wraps a Stream + Sink backend into a transport
use anyhow::Result;
use async_trait::async_trait;
use commands::Commands;
use futures::future::BoxFuture;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{broadcast, Mutex};
use tower::Service;

pub type Transport = Arc<
    Mutex<
        dyn Service<
                Commands,
                Response = Responses,
                Error = MiniDSPError,
                Future = BoxFuture<'static, Result<Responses, MiniDSPError>>,
            > + Send,
    >,
>;

#[cfg(feature = "hid")]
pub mod hid;

#[cfg(feature = "hid")]
use hidapi::HidError;

use crate::commands::{self, Responses};

use self::multiplexer::Multiplexer;
pub mod frame_codec;
pub mod multiplexer;
pub mod net;

#[derive(Error, Debug)]
pub enum MiniDSPError {
    #[error("An HID error has occurred: {0}")]
    #[cfg(feature = "hid")]
    HIDError(#[from] HidError),

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("A malformed packet was received: {0}")]
    MalformedResponse(String),

    #[error("This source was not recognized. Supported types are: 'toslink', 'usb', 'analog'")]
    InvalidSource,

    #[error("There are too many coeffiients in this filter")]
    TooManyCoefficients,

    #[error("Parse error")]
    ParseError(#[from] commands::ParseError),

    #[error("Malformed filter data")]
    MalformedFilterData,

    #[error("Transport error")]
    TransportError(#[from] broadcast::error::RecvError),

    #[error("Transport error: {0}")]
    TransportFailure(String),

    #[error("Transport has closed")]
    TransportClosed,

    #[error("Multiple concurrent commands were sent")]
    ConcurencyError,

    #[error("Internal error")]
    InternalError(#[from] anyhow::Error),
}

#[async_trait]
pub trait Openable {
    async fn open(&self) -> Result<Arc<Multiplexer>, MiniDSPError>;
}

// pub struct Transport {
//     /// The sending side of a broadcast channel used for received messages
//     receiver_tx: Arc<SyncMutex<Option<broadcast::Sender<Bytes>>>>,

//     /// Inner struct wrapping the device handle, ensuring only one sender exists simultaneously
//     /// The Arc is used to be able to hold a lock guard as 'static
//     inner: Arc<Mutex<Sender>>,
// }

// impl Transport {
//     pub fn new(rx: BoxStream, tx: BoxSink<anyhow::Error>) -> Self {
//         let (recv_send, _) = broadcast::channel::<Bytes>(10);
//         let transport = Transport {
//             receiver_tx: Arc::new(SyncMutex::new(Some(recv_send.clone()))),
//             inner: Arc::new(Mutex::new(Sender::new(tx))),
//         };

//         let receiver_tx = transport.receiver_tx.clone();
//         tokio::spawn(async move {
//             let _ = Transport::recv_loop(recv_send, rx).await;
//             let mut tx = receiver_tx.lock().unwrap();
//             // Set `receiver_tx` to None to mark this as closed
//             tx.take();
//         });

//         transport
//     }

//     async fn recv_loop(sender: broadcast::Sender<Bytes>, mut stream: BoxStream) -> Result<()> {
//         loop {
//             let data = stream
//                 .as_mut()
//                 .next()
//                 .await
//                 .ok_or(MiniDSPError::TransportClosed)?;
//             trace!("recv: {:02x?}", data.as_ref());
//             sender.send(data)?;
//         }
//     }

//     pub fn subscribe(&self) -> Result<broadcast::Receiver<Bytes>, MiniDSPError> {
//         let receiver = self.receiver_tx.lock().unwrap();
//         match receiver.as_ref() {
//             Some(tx) => Ok(tx.subscribe()),
//             None => Err(MiniDSPError::TransportClosed),
//         }
//     }

//     pub async fn send_lock(&'_ self) -> OwnedMutexGuard<Sender> {
//         self.inner.clone().lock_owned().await
//     }
// }

// pub struct Sender {
//     write: BoxSink<anyhow::Error>,
// }

// impl Sender {
//     fn new(device: BoxSink<anyhow::Error>) -> Self {
//         Sender { write: device }
//     }

//     pub async fn send(&mut self, frame: Bytes) -> Result<(), MiniDSPError> {
//         trace!("send: {:02x?}", frame.as_ref());
//         Ok(self
//             .write
//             .send(frame)
//             .await
//             .map_err(|e| MiniDSPError::TransportFailure(e.to_string()))?)
//     }
// }
