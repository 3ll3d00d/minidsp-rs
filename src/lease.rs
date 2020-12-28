use crate::{Gain, MiniDSP, Source};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

pub async fn lease_source(minidsp: Arc<Mutex<MiniDSP>>, source: Source) -> Result<SourceLease> {
    {
        let minidsp = minidsp.lock().await;
        minidsp.set_source(source).await?;
    }

    let (tx, rx) = oneshot::channel::<()>();
    {
        let minidsp = minidsp.clone();
        tokio::spawn(async move {
            let _ = rx.await;

            let minidsp = minidsp.lock_owned().await;
            if let Err(e) = minidsp.set_source(Source::Toslink).await {
                eprintln!("Failed to set source back: {:?}", e)
            }
            if let Err(e) = minidsp.set_master_volume(Gain(-40.)).await {
                eprintln!("Failed to set volume back: {:?}", e)
            }
        });
    }

    Ok(SourceLease { tx })
}

pub struct SourceLease {
    #[allow(dead_code)]
    tx: oneshot::Sender<()>,
}
