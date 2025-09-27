//! Basic server example demonstrating typestate builder with closures
//!
//! This example shows how to create a bandwidth server using the typestate builder
//! with closure-based event handling as requested.

use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error};
use bandwydth_server::{BandwidthServer, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting bandwidth server example with typestate builder and closures");

    // Generate self-signed certificate for demo
    let cert = generate_demo_cert()?;
    let key = generate_demo_key()?;

    // Create server using typestate builder with closure-based event handlers
    let server = BandwidthServer::builder()
        .bind_address("127.0.0.1:4433")?
        .with_certificate(cert)?
        .with_private_key(key)?
        .broadcast_interval_ms(1000)?  // 1 second intervals
        .client_timeout_ms(30000)?     // 30 second timeout
        
        // Closure for client connection events
        .on_client_connected(|client_info| {
            info!("🔗 Client connected: {} from {}", 
                client_info.id, 
                client_info.addr
            );
        })
        
        // Closure for client disconnection events  
        .on_client_disconnected(|client_info| {
            let duration = client_info.connection_duration()
                .map(|d| format!("{:.1}s", d.as_secs_f64()))
                .unwrap_or_else(|_| "unknown".to_string());
            
            info!("❌ Client disconnected: {} (connected for {})", 
                client_info.id, 
                duration
            );
        })
        
        // Closure for bandwidth measurement events
        .on_bandwidth_measured(|bandwidth_data| {
            info!("📊 Bandwidth: {:.1} Mbps down, {:.1} Mbps up, class: {:?}, quality: {:.0}%", 
                bandwidth_data.download_mbps,
                bandwidth_data.upload_mbps,
                bandwidth_data.bandwidth_class(),
                bandwidth_data.quality_score * 100.0
            );
        })
        
        // Closure for server error events
        .on_server_error(|error| {
            warn!("⚠️  Server error: {}", error);
        })
        
        .finalize_handlers()
        .build()?;

    info!("✅ Server created successfully using typestate builder");
    info!("🚀 Starting REAL QUIC bandwidth server with lib_bandwydth integration");
    info!("   Server will monitor actual network bandwidth and broadcast to QUIC clients");
    info!("   Press Ctrl+C to stop");

    // Run the server with real QUIC transport and bandwidth monitoring
    match server.run().await {
        Ok(()) => info!("Server shut down gracefully"),
        Err(e) => error!("Server error: {}", e),
    }

    Ok(())
}

/// Generate a demo self-signed certificate for testing
fn generate_demo_cert() -> Result<Vec<u8>> {
    let cert_pem = r#"-----BEGIN CERTIFICATE-----
MIIBhzCCASwwDQYJKoZIhvcNAQELBQAwOjELMAkGA1UEBhMCVVMxCzAJBgNVBAgM
AkNBMRAwDgYDVQQHDAdCZXJrZWxleTEMMAoGA1UECgwDRGVtbzAeFw0yNDExMzAy
MzE0MTBaFw0yNTExMzAyMzE0MTBaMDoxCzAJBgNVBAYTAlVTMQswCQYDVQQIDApD
YWxpZm9ybmlhMRAwDgYDVQQHDAdCZXJrZWxleTEMMAoGA1UECgwDRGVtbzBNMAkG
BSsOAwIaBQADQAAwPQI4AL3sOUbIkRfH4+K2sMXP6pSmF3kRj2nQIm1FjLdCiH3X
+s0tMFzTJ3jqUgR3RUnV2RbJJQyJxkwNPRXhYB8CAwEAATANBgkqhkiG9w0BAQsF
AAA4AQAAKKBzxHBfC7dJKjvr3pZuH5o8KLjD4A6JnQhxK8BgdN4L6vVwLaB+2MjO
8xwTFk8i1XM5YQnJYJQBTXEJ5d1p6QKF7sN+6wgHw9zF4+8xLz7bVgYj6s=
-----END CERTIFICATE-----"#;
    
    Ok(cert_pem.as_bytes().to_vec())
}

/// Generate a demo private key for testing
fn generate_demo_key() -> Result<Vec<u8>> {
    let key_pem = r#"-----BEGIN PRIVATE KEY-----
MIIBVAIBADANBgkqhkiG9w0BAQEFAASCAT4wggE6AgEAAjEAs4kOKzKAWNsW9J5r
X3V2zqSKzJ5G7mxYkpBKx6nN1ZH4BW7zL8b1mD3L5qL7U8n1RpPdAgMBAAECMB4y
+JXG7Y8Fb5Uq6RxQ2+qH6jGK4wYz1mKl8d7FgZ6kJqJKL9ZV2Hd8p1fS6J8wGT3y
QIBAKbG6Y5U8nF1K8YcqG6qK9L9QFHJgUuQKLgPG8b2N6l1Zv3B7W7ZJ5KqNz7N7
xY8ZGJzF4k1nD3H2gB5+zQIBAJQF8n1L6MqG3l8D2b2d3JzJ4f2HjQN7L7yR1t8K
LhOyVqP2m4l8YzGqB3JG6hF6ZS6qY8Z1L1wYKLlQZGrKF6QCQDc1v2b4U6YgK3y8
S4j8f7l2KzY2mZG5ZzGtKyJ1j4t8hLyU7Y4pF3YdQ2KqB8L6n1F8LyR3gYSqT8Y
zF7JQgG1UYwJAeHv5kTg5yDjl1SZfN2B7p4L8d8TyF6Zb7HbG5Yk3oTnZJ2H4B6
c1Wq8j5n2VqRg3Y8FzfO6k8p1F7KLQJBALrV2+7J6B8QyFz6L8HdY5c1p2z7h5Fz
kG3LjYpJjN8J7+Q5tG9U6zKJ1n4r5s4fF8z7qL2bYjF6n4J7GqQz6NQ1K=
-----END PRIVATE KEY-----"#;
    
    Ok(key_pem.as_bytes().to_vec())
}