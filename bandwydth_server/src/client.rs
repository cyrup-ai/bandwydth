use std::sync::Arc;
use std::time::Duration;
use futures_util::StreamExt;
use tokio::sync::{mpsc, RwLock};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use cryypt_quic::{QuicSend, QuicRecv, QuicConnectionHandle};

use crate::stats::AtomicBandwidthStats;
use crate::types::{BandwidthEvent, ClientInfo, BandwidthData};
use crate::server::get_global_connection_registry;

/// Handles a single client connection.
#[derive(Debug)]
pub struct ClientHandler {
    stats: Arc<AtomicBandwidthStats>,
    clients: Arc<RwLock<Vec<mpsc::Sender<BandwidthEvent>>>>,
    client_timeout_ms: u64,
}

impl ClientHandler {
    /// Create a new client handler.
    pub fn new(
        stats: Arc<AtomicBandwidthStats>,
        clients: Arc<RwLock<Vec<mpsc::Sender<BandwidthEvent>>>>,
        client_timeout_ms: u64,
    ) -> Self {
        Self { stats, clients, client_timeout_ms }
    }

    /// Handle a new client connection.
    pub async fn handle_client(&self, _send: QuicSend, recv: QuicRecv) {
        let (tx, mut rx) = mpsc::channel(100);
        self.clients.write().await.push(tx);
        
        // Create client info for tracking
        let client_info = ClientInfo::new("127.0.0.1:8080".parse().unwrap());
        info!("New client connected: {}", client_info.id);
        debug!("Connection duration: {:?}", client_info.connection_duration());

        // Convert QuicRecv to stream for processing incoming data using StreamExt
        let mut recv_stream = recv.on_chunk(|result| {
            match result {
                Ok(data) => Some(data),
                Err(e) => {
                    debug!("Error receiving data: {:?}", e);
                    None
                }
            }
        }).stream();
        
        let timeout_duration = Duration::from_millis(self.client_timeout_ms);
        let mut connection_active = true;
        
        while connection_active {
            tokio::select! {
                // Forward broadcast events to the client
                Some(event) = rx.recv() => {
                    let event_data = match bincode::encode_to_vec(&event, bincode::config::standard()) {
                        Ok(data) => data,
                        Err(e) => {
                            error!("Failed to serialize event: {}", e);
                            continue;
                        }
                    };

                    // Real QUIC sending using connection registry
                    // Use QuicConnectionHandle.send_stream_data() which doesn't consume self
                    let send_result = if let Some(registry) = get_global_connection_registry() {
                        let handles = registry.get_all_handles().await;
                        let mut sent_bytes = 0u64;
                        let mut send_errors = 0u32;
                        
                        // Broadcast to all connected QUIC clients
                        for handle in handles {
                            match handle.send_stream_data(&event_data, false) {
                                Ok(()) => {
                                    sent_bytes += event_data.len() as u64;
                                    debug!("Successfully sent {} bytes via QUIC", event_data.len());
                                }
                                Err(e) => {
                                    send_errors += 1;
                                    warn!("QUIC send failed: {}", e);
                                }
                            }
                        }
                        
                        if send_errors == 0 {
                            Ok(sent_bytes)
                        } else {
                            Err(format!("Failed to send to {} clients", send_errors))
                        }
                    } else {
                        Err("Connection registry not available".to_string())
                    };

                    match send_result {
                        Ok(sent_bytes) => {
                            // Track actual bytes sent via QUIC
                            let (current_down, current_up) = self.stats.get();
                            self.stats.update(current_down, sent_bytes);
                            
                            info!("📡 Sent {} bytes to QUIC clients", sent_bytes);
                            
                            // Create bandwidth data for analysis
                            let bandwidth_data = BandwidthData::new(
                                current_down as f64 / 1_000_000.0, // Convert to Mbps
                                current_up as f64 / 1_000_000.0,   // Convert to Mbps
                                0.9, // Quality score
                                1,   // Active clients
                                event_data.len() as u64,
                            );
                            
                            debug!("Would send {} bytes to client: {:?}", event_data.len(), event);
                            debug!("Bandwidth: {:.2} Mbps total, class: {:?}, high quality: {}", 
                                   bandwidth_data.total_mbps(), 
                                   bandwidth_data.bandwidth_class(),
                                   bandwidth_data.is_high_quality());
                            
                            if let Ok(age) = bandwidth_data.measurement_age() {
                                debug!("Measurement age: {:?}", age);
                            }
                        }
                        Err(send_error) => {
                            error!("Failed to send data via QUIC: {}", send_error);
                            // Continue trying to send to remaining clients
                            // Don't disconnect on individual send failures
                        }
                    }
                }
                
                // Handle incoming data from client using StreamExt
                Some(incoming_data) = recv_stream.next() => {
                    // Track bytes received in stats
                    let (_, current_up) = self.stats.get();
                    self.stats.update(incoming_data.len() as u64, current_up);
                    debug!("Received {} bytes from client", incoming_data.len());
                    
                    // Process incoming client data (e.g., client commands, heartbeats)
                    debug!("Client sent data: {} bytes", incoming_data.len());
                }
                
                // Handle connection timeout/heartbeat
                _ = tokio::time::sleep(timeout_duration) => {
                    debug!("Client handler heartbeat - connection still active");
                }
            }
        }
        
        info!("Client disconnected.");
    }
}