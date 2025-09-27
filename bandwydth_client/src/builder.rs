//! Typestate builder for bandwidth monitoring client
//!
//! Zero-allocation, compile-time safe builder pattern ensuring proper configuration
//! before client instantiation. Each state transition validates configuration completeness.

use std::marker::PhantomData;
use std::net::SocketAddr;
use std::str::FromStr;

use crate::client::BandwidthClient;
use crate::error::{BandwidthError, ConnectionError, ConfigurationIssue, Result};
use crate::events::{EventHandlers, IntoEventHandler};
use crate::types::{BandwidthEvent, BandwidthClientConfig};

/// Initial builder state - no configuration set
pub struct Initial;

/// Server address has been configured
pub struct ServerSet;

/// Event handlers have been configured
pub struct HandlersSet;

/// Typestate builder for bandwidth monitoring client
/// 
/// Enforces compile-time configuration validation through progressive state transitions.
/// Each method returns the next state in the configuration sequence, preventing
/// incomplete client instantiation.
pub struct BandwidthClientBuilder<State> {
    server_addr: Option<String>,
    connection_timeout_ms: u64,
    response_timeout_ms: u64,
    auto_reconnect: bool,
    reconnect_delay_ms: u64,
    max_reconnect_attempts: u32,
    event_handlers: EventHandlers,
    _state: PhantomData<State>,
}

impl BandwidthClientBuilder<Initial> {
    /// Create a new bandwidth client builder
    #[inline]
    pub const fn new() -> Self {
        Self {
            server_addr: None,
            connection_timeout_ms: 5000,
            response_timeout_ms: 10000,
            auto_reconnect: true,
            reconnect_delay_ms: 1000,
            max_reconnect_attempts: 5,
            event_handlers: EventHandlers::new(),
            _state: PhantomData,
        }
    }

    /// Set the server address to connect to
    /// 
    /// Validates the address format and transitions to ServerSet state.
    /// Accepts formats like "127.0.0.1:4433", "[::1]:4433", "example.com:8080".
    pub fn server_address(self, addr: impl AsRef<str>) -> Result<BandwidthClientBuilder<ServerSet>> {
        let addr_str = addr.as_ref();
        
        // Validate address format by attempting to parse it
        let _socket_addr = SocketAddr::from_str(addr_str)
            .map_err(|_| BandwidthError::configuration("server_addr", ConfigurationIssue::Invalid))?;
        
        Ok(BandwidthClientBuilder {
            server_addr: Some(addr_str.to_string()),
            connection_timeout_ms: self.connection_timeout_ms,
            response_timeout_ms: self.response_timeout_ms,
            auto_reconnect: self.auto_reconnect,
            reconnect_delay_ms: self.reconnect_delay_ms,
            max_reconnect_attempts: self.max_reconnect_attempts,
            event_handlers: self.event_handlers,
            _state: PhantomData,
        })
    }
}

impl BandwidthClientBuilder<ServerSet> {
    /// Set connection timeout in milliseconds
    /// 
    /// Configures how long to wait for initial connection establishment.
    /// Minimum timeout is 1000ms to allow for network latency.
    pub fn connection_timeout_ms(mut self, timeout_ms: u64) -> Result<Self> {
        if timeout_ms < 1000 {
            return Err(BandwidthError::configuration("connection_timeout_ms", ConfigurationIssue::OutOfRange));
        }
        
        self.connection_timeout_ms = timeout_ms;
        Ok(self)
    }

    /// Set response timeout in milliseconds
    /// 
    /// Configures maximum time to wait for server responses.
    /// Minimum timeout is 1000ms to allow for processing delays.
    pub fn response_timeout_ms(mut self, timeout_ms: u64) -> Result<Self> {
        if timeout_ms < 1000 {
            return Err(BandwidthError::configuration("response_timeout_ms", ConfigurationIssue::OutOfRange));
        }
        
        self.response_timeout_ms = timeout_ms;
        Ok(self)
    }

    /// Enable or disable automatic reconnection
    /// 
    /// When enabled, client will automatically attempt to reconnect
    /// when connection is lost due to network issues.
    pub fn auto_reconnect(mut self, enabled: bool) -> Self {
        self.auto_reconnect = enabled;
        self
    }

    /// Set reconnection delay in milliseconds
    /// 
    /// Configures delay between reconnection attempts.
    /// Minimum delay is 100ms to prevent tight retry loops.
    pub fn reconnect_delay_ms(mut self, delay_ms: u64) -> Result<Self> {
        if delay_ms < 100 {
            return Err(BandwidthError::configuration("reconnect_delay_ms", ConfigurationIssue::OutOfRange));
        }
        
        self.reconnect_delay_ms = delay_ms;
        Ok(self)
    }

    /// Set maximum reconnection attempts
    /// 
    /// Configures maximum number of reconnection attempts (0 = infinite).
    /// Helps prevent infinite retry loops in permanent failure scenarios.
    pub fn max_reconnect_attempts(mut self, max_attempts: u32) -> Self {
        self.max_reconnect_attempts = max_attempts;
        self
    }

    /// Set handler for bandwidth event reception
    /// 
    /// Called when bandwidth events are received from the server. Provides real-time
    /// bandwidth measurements including download/upload speeds and classifications.
    pub fn on_bandwidth_event<F>(mut self, handler: F) -> Self
    where
        F: Fn(&BandwidthEvent) + Send + Sync + 'static,
    {
        self.event_handlers.bandwidth_event = Some(handler.into_handler());
        self
    }

    /// Set handler for successful connection establishment
    /// 
    /// Called when client successfully connects to the bandwidth server.
    /// Useful for initialization, logging, and UI updates.
    pub fn on_connected<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.event_handlers.connected = Some(handler.into_handler());
        self
    }

    /// Set handler for connection termination
    /// 
    /// Called when client disconnects from server, either gracefully
    /// or due to network issues. Provides opportunity for cleanup.
    pub fn on_disconnected<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.event_handlers.disconnected = Some(handler.into_handler());
        self
    }

    /// Set handler for client error events
    /// 
    /// Called when non-fatal errors occur during client operation.
    /// Provides detailed error information for logging and recovery.
    pub fn on_error<F>(mut self, handler: F) -> Self
    where
        F: Fn(&BandwidthError) + Send + Sync + 'static,
    {
        self.event_handlers.client_error = Some(handler.into_handler());
        self
    }

    /// Transition to HandlersSet state after configuring event handlers
    /// 
    /// This method finalizes event handler configuration and enables
    /// the build() method to construct the client.
    pub fn finalize_handlers(self) -> BandwidthClientBuilder<HandlersSet> {
        BandwidthClientBuilder {
            server_addr: self.server_addr,
            connection_timeout_ms: self.connection_timeout_ms,
            response_timeout_ms: self.response_timeout_ms,
            auto_reconnect: self.auto_reconnect,
            reconnect_delay_ms: self.reconnect_delay_ms,
            max_reconnect_attempts: self.max_reconnect_attempts,
            event_handlers: self.event_handlers,
            _state: PhantomData,
        }
    }

    /// Build the bandwidth client with current configuration
    /// 
    /// Validates all configuration and constructs a fully functional client
    /// ready for connection. This is the final step in the builder pattern.
    pub fn build(self) -> Result<BandwidthClient> {
        let config = BandwidthClientConfig {
            server_addr: self.server_addr.ok_or_else(|| {
                BandwidthError::configuration("server_addr", ConfigurationIssue::Missing)
            })?,
            connection_timeout_ms: self.connection_timeout_ms,
            response_timeout_ms: self.response_timeout_ms,
            auto_reconnect: self.auto_reconnect,
            reconnect_delay_ms: self.reconnect_delay_ms,
            max_reconnect_attempts: self.max_reconnect_attempts,
        };

        BandwidthClient::new(config, self.event_handlers)
    }
}

impl BandwidthClientBuilder<HandlersSet> {
    /// Set connection timeout in milliseconds
    /// 
    /// Configures how long to wait for initial connection establishment.
    /// Minimum timeout is 1000ms to allow for network latency.
    pub fn connection_timeout_ms(mut self, timeout_ms: u64) -> Result<Self> {
        if timeout_ms < 1000 {
            return Err(BandwidthError::configuration("connection_timeout_ms", ConfigurationIssue::OutOfRange));
        }
        
        self.connection_timeout_ms = timeout_ms;
        Ok(self)
    }

    /// Set response timeout in milliseconds
    /// 
    /// Configures maximum time to wait for server responses.
    /// Minimum timeout is 1000ms to allow for processing delays.
    pub fn response_timeout_ms(mut self, timeout_ms: u64) -> Result<Self> {
        if timeout_ms < 1000 {
            return Err(BandwidthError::configuration("response_timeout_ms", ConfigurationIssue::OutOfRange));
        }
        
        self.response_timeout_ms = timeout_ms;
        Ok(self)
    }

    /// Enable or disable automatic reconnection
    /// 
    /// When enabled, client will automatically attempt to reconnect
    /// when connection is lost due to network issues.
    pub fn auto_reconnect(mut self, enabled: bool) -> Self {
        self.auto_reconnect = enabled;
        self
    }

    /// Set reconnection delay in milliseconds
    /// 
    /// Configures delay between reconnection attempts.
    /// Minimum delay is 100ms to prevent tight retry loops.
    pub fn reconnect_delay_ms(mut self, delay_ms: u64) -> Result<Self> {
        if delay_ms < 100 {
            return Err(BandwidthError::configuration("reconnect_delay_ms", ConfigurationIssue::OutOfRange));
        }
        
        self.reconnect_delay_ms = delay_ms;
        Ok(self)
    }

    /// Set maximum reconnection attempts
    /// 
    /// Configures maximum number of reconnection attempts (0 = infinite).
    /// Helps prevent infinite retry loops in permanent failure scenarios.
    pub fn max_reconnect_attempts(mut self, max_attempts: u32) -> Self {
        self.max_reconnect_attempts = max_attempts;
        self
    }

    /// Build the bandwidth client with current configuration
    /// 
    /// Validates all configuration and constructs a fully functional client
    /// ready for connection. This is the final step in the builder pattern.
    pub fn build(self) -> Result<BandwidthClient> {
        let config = BandwidthClientConfig {
            server_addr: self.server_addr.ok_or_else(|| {
                BandwidthError::configuration("server_addr", ConfigurationIssue::Missing)
            })?,
            connection_timeout_ms: self.connection_timeout_ms,
            response_timeout_ms: self.response_timeout_ms,
            auto_reconnect: self.auto_reconnect,
            reconnect_delay_ms: self.reconnect_delay_ms,
            max_reconnect_attempts: self.max_reconnect_attempts,
        };

        BandwidthClient::new(config, self.event_handlers)
    }
}

impl Default for BandwidthClientBuilder<Initial> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}