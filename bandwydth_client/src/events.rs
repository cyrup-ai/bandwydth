//! Event handler type definitions for bandwidth monitoring client
//!
//! Zero-allocation, thread-safe event handling system using Arc-wrapped closures
//! optimized for client-side bandwidth monitoring and server communication

use std::sync::Arc;
use crate::types::BandwidthEvent;
use crate::error::BandwidthError;

/// Handler for bandwidth event reception
/// 
/// Called when bandwidth events are received from the server.
/// Provides real-time bandwidth measurements including download/upload speeds,
/// timestamps, and bandwidth classifications.
pub type BandwidthEventHandler = Arc<dyn Fn(&BandwidthEvent) + Send + Sync>;

/// Handler for successful connection establishment
///
/// Called when the client successfully connects to the bandwidth server.
/// Useful for initialization, logging, and UI updates.
pub type ConnectedHandler = Arc<dyn Fn() + Send + Sync>;

/// Handler for connection termination
///
/// Called when the client disconnects from the server, either gracefully
/// or due to network issues. Provides opportunity for cleanup and reconnection logic.
pub type DisconnectedHandler = Arc<dyn Fn() + Send + Sync>;

/// Handler for client error events
///
/// Called when errors occur during client operation that don't require
/// immediate termination. Provides detailed error information for logging,
/// monitoring, and recovery strategies.
pub type ClientErrorHandler = Arc<dyn Fn(&BandwidthError) + Send + Sync>;

/// Collection of all event handlers for bandwidth client
///
/// Provides a convenient way to group and pass all event handlers
/// together while maintaining zero-allocation performance.
#[derive(Clone)]
pub struct EventHandlers {
    pub bandwidth_event: Option<BandwidthEventHandler>,
    pub connected: Option<ConnectedHandler>, 
    pub disconnected: Option<DisconnectedHandler>,
    pub client_error: Option<ClientErrorHandler>,
}

impl EventHandlers {
    /// Create new event handlers collection with all handlers empty
    #[inline]
    pub const fn new() -> Self {
        Self {
            bandwidth_event: None,
            connected: None,
            disconnected: None,
            client_error: None,
        }
    }
    
    /// Invoke bandwidth event handler if present
    #[inline]
    pub fn on_bandwidth_event(&self, event: &BandwidthEvent) {
        if let Some(ref handler) = self.bandwidth_event {
            handler(event);
        }
    }
    
    /// Invoke connected handler if present
    #[inline]
    pub fn on_connected(&self) {
        if let Some(ref handler) = self.connected {
            handler();
        }
    }
    
    /// Invoke disconnected handler if present
    #[inline]
    pub fn on_disconnected(&self) {
        if let Some(ref handler) = self.disconnected {
            handler();
        }
    }
    
    /// Invoke client error handler if present
    #[inline]
    pub fn on_client_error(&self, error: &BandwidthError) {
        if let Some(ref handler) = self.client_error {
            handler(error);
        }
    }
}

impl Default for EventHandlers {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience trait for converting closures to event handlers
///
/// Allows ergonomic conversion from various closure types while maintaining
/// the Arc<dyn Fn() + Send + Sync> requirement for thread safety.
pub trait IntoEventHandler<T> {
    type Handler;
    fn into_handler(self) -> Self::Handler;
}

impl<F> IntoEventHandler<BandwidthEvent> for F
where
    F: Fn(&BandwidthEvent) + Send + Sync + 'static,
{
    type Handler = BandwidthEventHandler;
    
    #[inline]
    fn into_handler(self) -> Self::Handler {
        Arc::new(self)
    }
}

impl<F> IntoEventHandler<()> for F
where
    F: Fn() + Send + Sync + 'static,
{
    type Handler = ConnectedHandler;
    
    #[inline]
    fn into_handler(self) -> Self::Handler {
        Arc::new(self)
    }
}

impl<F> IntoEventHandler<BandwidthError> for F
where
    F: Fn(&BandwidthError) + Send + Sync + 'static,
{
    type Handler = ClientErrorHandler;
    
    #[inline]
    fn into_handler(self) -> Self::Handler {
        Arc::new(self)
    }
}

/// Additional convenience types for common event handler patterns
pub mod handlers {
    use super::*;
    use tracing::{info, warn, error, debug};
    
    /// Create a logging bandwidth event handler
    /// 
    /// Returns a handler that logs all received bandwidth events
    /// at INFO level with structured formatting.
    pub fn logging_bandwidth_handler() -> BandwidthEventHandler {
        Arc::new(|event: &BandwidthEvent| {
            info!(
                "Bandwidth: {} Mbps down, {} Mbps up, class: {}, timestamp: {}",
                (event.download_bps as f64) / 1_000_000.0,
                (event.upload_bps as f64) / 1_000_000.0,
                event.class,
                event.timestamp
            );
        })
    }
    
    /// Create a logging connection handler
    /// 
    /// Returns a handler that logs connection establishment
    /// at INFO level for monitoring purposes.
    pub fn logging_connected_handler() -> ConnectedHandler {
        Arc::new(|| {
            info!("Successfully connected to bandwidth server");
        })
    }
    
    /// Create a logging disconnection handler
    /// 
    /// Returns a handler that logs disconnection events
    /// at WARN level for monitoring purposes.
    pub fn logging_disconnected_handler() -> DisconnectedHandler {
        Arc::new(|| {
            warn!("Disconnected from bandwidth server");
        })
    }
    
    /// Create a logging error handler
    /// 
    /// Returns a handler that logs errors at ERROR level
    /// with full error context for debugging.
    pub fn logging_error_handler() -> ClientErrorHandler {
        Arc::new(|error: &BandwidthError| {
            error!("Client error: {}", error);
            
            if error.is_recoverable() {
                debug!("Error is recoverable, continuing operation");
            } else if error.requires_reconnection() {
                debug!("Error requires reconnection");
            } else if error.is_permanent() {
                debug!("Permanent error, operation cannot continue");
            }
        })
    }
}