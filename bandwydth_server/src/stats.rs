use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::{debug, info, warn, error};

use crate::types::{BandwidthClass, BandwidthEvent};
use crate::server::get_global_connection_registry;

/// Lock-free bandwidth statistics using atomic operations
#[derive(Debug, Default)]
pub struct AtomicBandwidthStats {
    download_bps: AtomicU64,
    upload_bps: AtomicU64,
    last_update: AtomicU64,
    sample_count: AtomicUsize,
}

impl AtomicBandwidthStats {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn update(&self, download_bps: u64, upload_bps: u64) {
        self.download_bps.store(download_bps, Ordering::Relaxed);
        self.upload_bps.store(upload_bps, Ordering::Relaxed);
        self.last_update.store(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_secs()),
            Ordering::Relaxed,
        );
        self.sample_count.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn get(&self) -> (u64, u64) {
        (
            self.download_bps.load(Ordering::Relaxed),
            self.upload_bps.load(Ordering::Relaxed),
        )
    }
}


/// Monitors bandwidth and broadcasts events to clients.
#[derive(Debug)]
pub struct BandwidthMonitor {
    stats: Arc<AtomicBandwidthStats>,
    clients: Arc<RwLock<Vec<mpsc::Sender<BandwidthEvent>>>>,
}

impl BandwidthMonitor {
    /// Create a new monitor.
    pub fn new(stats: Arc<AtomicBandwidthStats>, clients: Arc<RwLock<Vec<mpsc::Sender<BandwidthEvent>>>>) -> Self {
        Self { stats, clients }
    }

    /// Start the monitoring loop.
    pub async fn run(&self, broadcast_interval_ms: u64) {
        let mut interval = interval(Duration::from_millis(broadcast_interval_ms));
        info!("Bandwidth monitor started.");

        loop {
            interval.tick().await;
            let (download_bps, upload_bps) = self.stats.get();

            let event = BandwidthEvent {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_or(0, |d| d.as_secs()),
                download_bps,
                upload_bps,
                class: BandwidthClass::from_bps(download_bps).as_str().to_string(),
            };

            debug!("Broadcasting bandwidth event: {:?}", event);
            self.broadcast(event).await;
        }
    }

    /// Broadcast an event to all connected clients - public method for server usage
    pub async fn broadcast_event(&self, event: BandwidthEvent) {
        self.broadcast(event).await;
    }

    /// Broadcast raw serialized data to all connected QUIC clients
    /// 
    /// This method sends protocol messages directly over QUIC streams using
    /// the connection registry for real-time bandwidth data distribution.
    pub async fn broadcast_raw(&self, serialized_data: &[u8]) {
        if let Some(registry) = get_global_connection_registry() {
            let handles = registry.get_all_handles().await;
            let client_count = handles.len();
            
            if client_count == 0 {
                debug!("No QUIC clients connected for broadcast");
                return;
            }
            
            let mut successful_sends = 0u32;
            let mut failed_sends = 0u32;
            
            // Broadcast to all connected QUIC clients
            for (index, handle) in handles.iter().enumerate() {
                match handle.send_stream_data(serialized_data, false) {
                    Ok(()) => {
                        successful_sends += 1;
                        debug!("Successfully broadcast {} bytes to QUIC client {}", 
                               serialized_data.len(), index);
                    }
                    Err(e) => {
                        failed_sends += 1;
                        warn!("Failed to broadcast to QUIC client {}: {}", index, e);
                    }
                }
            }
            
            // Log broadcast summary
            if failed_sends == 0 {
                info!("📡 Successfully broadcast {} bytes to {} QUIC clients", 
                      serialized_data.len(), successful_sends);
            } else {
                warn!("📡 Broadcast completed: {} successful, {} failed out of {} total clients",
                      successful_sends, failed_sends, client_count);
            }
        } else {
            error!("QUIC connection registry not available for broadcast");
        }
    }

    /// Broadcast an event to all connected clients.
    async fn broadcast(&self, event: BandwidthEvent) {
        let mut clients = self.clients.write().await;
        let mut disconnected_indices = Vec::new();

        for (i, client_tx) in clients.iter().enumerate() {
            if client_tx.send(event.clone()).await.is_err() {
                disconnected_indices.push(i);
            }
        }

        // Remove disconnected clients
        for i in disconnected_indices.iter().rev() {
            clients.remove(*i);
        }
    }
}
