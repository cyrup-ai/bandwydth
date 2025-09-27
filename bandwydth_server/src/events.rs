//! Event handler type definitions for bandwidth monitoring server
//!
//! Zero-allocation, thread-safe event handling system using Arc-wrapped closures

use std::sync::Arc;
use crate::types::{ClientInfo, BandwidthData};
use crate::error::BandwidthError;

/// Handler for client connection events
/// 
/// Called when a new client establishes a QUIC connection to the server.
/// The handler receives client information including unique ID, socket address,
/// and connection timestamp.
pub type ClientConnectedHandler = Arc<dyn Fn(&ClientInfo) + Send + Sync>;

/// Handler for client disconnection events
///
/// Called when a client closes their QUIC connection or times out.
/// Provides client information for cleanup and logging purposes.
pub type ClientDisconnectedHandler = Arc<dyn Fn(&ClientInfo) + Send + Sync>;

/// Handler for bandwidth measurement events
///
/// Called periodically when new bandwidth measurements are collected.
/// Receives comprehensive bandwidth data including download/upload speeds,
/// measurement timestamp, and quality metrics.
pub type BandwidthMeasuredHandler = Arc<dyn Fn(&BandwidthData) + Send + Sync>;

/// Handler for server error events
///
/// Called when non-fatal errors occur during server operation.
/// Provides detailed error information for logging, monitoring, and recovery.
/// Fatal errors that require shutdown are handled separately.
pub type ServerErrorHandler = Arc<dyn Fn(&BandwidthError) + Send + Sync>;

/// Collection of all event handlers for bandwidth server
///
/// Provides a convenient way to group and pass all event handlers
/// together while maintaining zero-allocation performance.
#[derive(Clone)]
pub struct EventHandlers {
    pub client_connected: Option<ClientConnectedHandler>,
    pub client_disconnected: Option<ClientDisconnectedHandler>, 
    pub bandwidth_measured: Option<BandwidthMeasuredHandler>,
    pub server_error: Option<ServerErrorHandler>,
}

impl EventHandlers {
    /// Create new event handlers collection with all handlers empty
    #[inline]
    pub const fn new() -> Self {
        Self {
            client_connected: None,
            client_disconnected: None,
            bandwidth_measured: None,
            server_error: None,
        }
    }
    
    /// Invoke client connected handler if present
    #[inline]
    pub fn on_client_connected(&self, client_info: &ClientInfo) {
        if let Some(ref handler) = self.client_connected {
            handler(client_info);
        }
    }
    
    /// Invoke client disconnected handler if present
    #[inline]
    pub fn on_client_disconnected(&self, client_info: &ClientInfo) {
        if let Some(ref handler) = self.client_disconnected {
            handler(client_info);
        }
    }
    
    /// Invoke bandwidth measured handler if present
    #[inline]
    pub fn on_bandwidth_measured(&self, bandwidth_data: &BandwidthData) {
        if let Some(ref handler) = self.bandwidth_measured {
            handler(bandwidth_data);
        }
    }
    
    /// Invoke server error handler if present
    #[inline]
    pub fn on_server_error(&self, error: &BandwidthError) {
        if let Some(ref handler) = self.server_error {
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

impl<F> IntoEventHandler<ClientInfo> for F
where
    F: Fn(&ClientInfo) + Send + Sync + 'static,
{
    type Handler = ClientConnectedHandler;
    
    #[inline]
    fn into_handler(self) -> Self::Handler {
        Arc::new(self)
    }
}

impl<F> IntoEventHandler<BandwidthData> for F  
where
    F: Fn(&BandwidthData) + Send + Sync + 'static,
{
    type Handler = BandwidthMeasuredHandler;
    
    #[inline]
    fn into_handler(self) -> Self::Handler {
        Arc::new(self)
    }
}

impl<F> IntoEventHandler<BandwidthError> for F
where
    F: Fn(&BandwidthError) + Send + Sync + 'static,
{
    type Handler = ServerErrorHandler;
    
    #[inline]
    fn into_handler(self) -> Self::Handler {
        Arc::new(self)
    }
}