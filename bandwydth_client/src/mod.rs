use anyhow::Result;
use bandwydth_server::BandwidthEvent;
use cryypt::Cryypt;
use futures_core::Stream;
use futures_util::StreamExt;
use tracing::{error, info};

/// Super easy bandwidth monitoring client
pub struct BandwidthClient {
    server_addr: String,
}

impl BandwidthClient {
    /// Create a new bandwidth client
    pub fn new(server_addr: impl Into<String>) -> Self {
        Self {
            server_addr: server_addr.into(),
        }
    }

    /// Connect and stream bandwidth events - it's that easy!
    pub async fn stream(&self) -> Result<impl Stream<Item = BandwidthEvent>> {
        info!("Connecting to bandwidth server at {}", self.server_addr);

        // Connect using unified cryypt QUIC API
        let client = Cryypt::quic()
            .client()
            .with_server_name("bandwidth.progresshub")
            .connect(&self.server_addr)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;

        // Open stream
        let (_send, recv) = client
            .open_bi()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to open stream: {}", e))?;

        // Return event stream
        let stream = async_stream::stream! {
            let mut buffer = Vec::with_capacity(1024);
            
            loop {
                // Read from QUIC stream
                match recv.read(&mut buffer).await {
                    Ok(0) => {
                        info!("Bandwidth server closed connection");
                        break;
                    }
                    Ok(n) => {
                        // Deserialize event
                        match serde_json::from_slice::<BandwidthEvent>(&buffer[..n]) {
                            Ok(event) => yield event,
                            Err(e) => {
                                error!("Failed to deserialize event: {}", e);
                            }
                        }
                        buffer.clear();
                    }
                    Err(e) => {
                        error!("Failed to read from server: {}", e);
                        break;
                    }
                }
            }
        };

        Ok(stream)
    }
}

/// Convenience function for one-liner usage
pub async fn stream_bandwidth(addr: &str) -> Result<impl Stream<Item = BandwidthEvent>> {
    BandwidthClient::new(addr).stream().await
}