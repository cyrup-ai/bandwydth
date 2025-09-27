//! Basic client example demonstrating typestate builder with closures
//!
//! This example shows how to create a bandwidth client using the typestate builder
//! with closure-based event handling as requested.

use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error};
use bandwydth_client::{BandwidthClient, ConnectionState, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting bandwidth client example with typestate builder and closures");

    // Create client using typestate builder with closure-based event handlers
    let client = BandwidthClient::builder()
        .server_address("127.0.0.1:4433")?
        .connection_timeout_ms(5000)?      // 5 second connection timeout
        .response_timeout_ms(10000)?       // 10 second response timeout
        .auto_reconnect(true)              // Enable auto-reconnection
        .reconnect_delay_ms(2000)?         // 2 second delay between reconnects
        .max_reconnect_attempts(3)         // Try 3 times then give up
        
        // Closure for bandwidth events received from server
        .on_bandwidth_event(|bandwidth_event| {
            info!("📊 Received bandwidth: {:.1} Mbps down, {:.1} Mbps up, class: {}", 
                (bandwidth_event.download_bps as f64) / 1_000_000.0,
                (bandwidth_event.upload_bps as f64) / 1_000_000.0,
                bandwidth_event.class
            );
        })
        
        // Closure for successful connection
        .on_connected(|| {
            info!("🔗 Successfully connected to bandwidth server!");
        })
        
        // Closure for disconnection
        .on_disconnected(|| {
            info!("❌ Disconnected from bandwidth server");
        })
        
        // Closure for client errors
        .on_error(|error| {
            warn!("⚠️  Client error: {}", error);
        })
        
        .finalize_handlers()
        .build()?;

    info!("✅ Client created successfully using typestate builder");
    info!("🔄 Connection state: {:?}", client.state());
    info!("🚀 Connecting to REAL QUIC bandwidth server - will receive actual bandwidth data");

    // Connect to the server (this starts the connection process)
    let client_handle = tokio::spawn(async move {
        match client.connect().await {
            Ok(()) => info!("Client connection completed"),
            Err(e) => error!("Client connection failed: {}", e),
        }
    });

    // Let the client run for 30 seconds to receive real bandwidth data
    info!("⏰ Running for 30 seconds to receive real bandwidth measurements from server...");
    sleep(Duration::from_secs(30)).await;

    info!("🛑 Shutting down client example");
    client_handle.abort();
    
    // Give a moment for cleanup
    sleep(Duration::from_millis(100)).await;
    
    info!("✅ Client example completed");
    Ok(())
}