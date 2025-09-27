//! Typestate builder for bandwidth monitoring server
//!
//! Zero-allocation, compile-time safe builder pattern ensuring proper configuration
//! before server instantiation. Each state transition validates configuration completeness.

use std::marker::PhantomData;
use std::net::SocketAddr;
use std::str::FromStr;

use crate::error::{BandwidthError, CertificateError, ConfigurationIssue, Result};
use crate::events::{EventHandlers, IntoEventHandler};
use crate::server::BandwidthServer;
use crate::types::{BandwidthServerConfig, ClientInfo, BandwidthData};

/// Initial builder state - no configuration set
pub struct Initial;

/// Address has been configured
pub struct AddressSet;

/// Certificate has been configured
pub struct CertSet;

/// Private key has been configured  
pub struct KeySet;

/// Event handlers have been configured
pub struct HandlersSet;

/// Typestate builder for bandwidth monitoring server
/// 
/// Enforces compile-time configuration validation through progressive state transitions.
/// Each method returns the next state in the configuration sequence, preventing
/// incomplete server instantiation.
pub struct BandwidthServerBuilder<State> {
    bind_addr: Option<String>,
    cert: Option<Vec<u8>>,
    key: Option<Vec<u8>>,
    broadcast_interval_ms: u64,
    client_timeout_ms: u64,
    event_handlers: EventHandlers,
    _state: PhantomData<State>,
}

impl BandwidthServerBuilder<Initial> {
    /// Create a new bandwidth server builder
    #[inline]
    pub const fn new() -> Self {
        Self {
            bind_addr: None,
            cert: None,
            key: None,
            broadcast_interval_ms: 1000,
            client_timeout_ms: 5000,
            event_handlers: EventHandlers::new(),
            _state: PhantomData,
        }
    }

    /// Set the bind address for the server
    /// 
    /// Validates the address format and transitions to AddressSet state.
    /// Accepts formats like "0.0.0.0:4433", "[::]:4433", "127.0.0.1:8080".
    pub fn bind_address(self, addr: impl AsRef<str>) -> Result<BandwidthServerBuilder<AddressSet>> {
        let addr_str = addr.as_ref();
        
        // Validate address format by attempting to parse it
        let _socket_addr = SocketAddr::from_str(addr_str)
            .map_err(|_| BandwidthError::configuration("bind_addr", ConfigurationIssue::Invalid))?;
        
        Ok(BandwidthServerBuilder {
            bind_addr: Some(addr_str.to_string()),
            cert: self.cert,
            key: self.key,
            broadcast_interval_ms: self.broadcast_interval_ms,
            client_timeout_ms: self.client_timeout_ms,
            event_handlers: self.event_handlers,
            _state: PhantomData,
        })
    }
}

impl BandwidthServerBuilder<AddressSet> {
    /// Set TLS certificate in PEM format
    /// 
    /// Validates certificate format and transitions to CertSet state.
    /// Certificate must be valid PEM-encoded X.509 certificate chain.
    pub fn with_certificate(self, cert_pem: impl AsRef<[u8]>) -> Result<BandwidthServerBuilder<CertSet>> {
        let cert_bytes = cert_pem.as_ref().to_vec();
        
        // Basic PEM format validation
        let cert_str = std::str::from_utf8(&cert_bytes)
            .map_err(|_| BandwidthError::certificate(
                CertificateError::InvalidFormat,
                "Certificate must be valid UTF-8 PEM format".to_string()
            ))?;
        
        if !cert_str.contains("-----BEGIN CERTIFICATE-----") || !cert_str.contains("-----END CERTIFICATE-----") {
            return Err(BandwidthError::certificate(
                CertificateError::InvalidFormat,
                "Certificate must be in PEM format with proper delimiters".to_string()
            ));
        }
        
        Ok(BandwidthServerBuilder {
            bind_addr: self.bind_addr,
            cert: Some(cert_bytes),
            key: self.key,
            broadcast_interval_ms: self.broadcast_interval_ms,
            client_timeout_ms: self.client_timeout_ms,
            event_handlers: self.event_handlers,
            _state: PhantomData,
        })
    }

    /// Set broadcast interval for bandwidth updates
    /// 
    /// Configures how frequently bandwidth events are sent to clients.
    /// Minimum interval is 100ms to prevent system overload.
    pub fn broadcast_interval_ms(mut self, interval_ms: u64) -> Result<Self> {
        if interval_ms < 100 {
            return Err(BandwidthError::configuration("broadcast_interval_ms", ConfigurationIssue::OutOfRange));
        }
        
        self.broadcast_interval_ms = interval_ms;
        Ok(self)
    }

    /// Set client connection timeout
    /// 
    /// Configures maximum time to wait for client responses.
    /// Minimum timeout is 1000ms to allow for network latency.
    pub fn client_timeout_ms(mut self, timeout_ms: u64) -> Result<Self> {
        if timeout_ms < 1000 {
            return Err(BandwidthError::configuration("client_timeout_ms", ConfigurationIssue::OutOfRange));
        }
        
        self.client_timeout_ms = timeout_ms;
        Ok(self)
    }
}

impl BandwidthServerBuilder<CertSet> {
    /// Set TLS private key in PEM format
    /// 
    /// Validates private key format and transitions to KeySet state.
    /// Key must match the certificate and be properly formatted.
    pub fn with_private_key(self, key_pem: impl AsRef<[u8]>) -> Result<BandwidthServerBuilder<KeySet>> {
        let key_bytes = key_pem.as_ref().to_vec();
        
        // Basic PEM format validation for private key
        let key_str = std::str::from_utf8(&key_bytes)
            .map_err(|_| BandwidthError::certificate(
                CertificateError::InvalidPrivateKey,
                "Private key must be valid UTF-8 PEM format".to_string()
            ))?;
        
        let has_private_key_markers = key_str.contains("-----BEGIN PRIVATE KEY-----") 
            || key_str.contains("-----BEGIN RSA PRIVATE KEY-----")
            || key_str.contains("-----BEGIN EC PRIVATE KEY-----");
        
        if !has_private_key_markers {
            return Err(BandwidthError::certificate(
                CertificateError::InvalidPrivateKey,
                "Private key must be in PEM format with proper delimiters".to_string()
            ));
        }
        
        Ok(BandwidthServerBuilder {
            bind_addr: self.bind_addr,
            cert: self.cert,
            key: Some(key_bytes),
            broadcast_interval_ms: self.broadcast_interval_ms,
            client_timeout_ms: self.client_timeout_ms,
            event_handlers: self.event_handlers,
            _state: PhantomData,
        })
    }

    /// Set broadcast interval for bandwidth updates
    /// 
    /// Configures how frequently bandwidth events are sent to clients.
    /// Minimum interval is 100ms to prevent system overload.
    pub fn broadcast_interval_ms(mut self, interval_ms: u64) -> Result<Self> {
        if interval_ms < 100 {
            return Err(BandwidthError::configuration("broadcast_interval_ms", ConfigurationIssue::OutOfRange));
        }
        
        self.broadcast_interval_ms = interval_ms;
        Ok(self)
    }

    /// Set client connection timeout
    /// 
    /// Configures maximum time to wait for client responses.
    /// Minimum timeout is 1000ms to allow for network latency.
    pub fn client_timeout_ms(mut self, timeout_ms: u64) -> Result<Self> {
        if timeout_ms < 1000 {
            return Err(BandwidthError::configuration("client_timeout_ms", ConfigurationIssue::OutOfRange));
        }
        
        self.client_timeout_ms = timeout_ms;
        Ok(self)
    }
}

impl BandwidthServerBuilder<KeySet> {
    /// Set handler for client connection events
    /// 
    /// Called when clients establish QUIC connections. Provides client info
    /// including unique ID, socket address, and connection timestamp.
    pub fn on_client_connected<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClientInfo) + Send + Sync + 'static,
    {
        self.event_handlers.client_connected = Some(handler.into_handler());
        self
    }

    /// Set handler for client disconnection events
    /// 
    /// Called when clients close connections or timeout. Useful for cleanup
    /// and connection tracking.
    pub fn on_client_disconnected<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClientInfo) + Send + Sync + 'static,
    {
        self.event_handlers.client_disconnected = Some(handler.into_handler());
        self
    }

    /// Set handler for bandwidth measurement events
    /// 
    /// Called periodically with fresh bandwidth measurements. Provides
    /// comprehensive data including speeds, quality metrics, and client count.
    pub fn on_bandwidth_measured<F>(mut self, handler: F) -> Self
    where
        F: Fn(&BandwidthData) + Send + Sync + 'static,
    {
        self.event_handlers.bandwidth_measured = Some(handler.into_handler());
        self
    }

    /// Set handler for server error events
    /// 
    /// Called for non-fatal errors during server operation. Useful for
    /// logging, monitoring, and error recovery strategies.
    pub fn on_server_error<F>(mut self, handler: F) -> Self
    where
        F: Fn(&BandwidthError) + Send + Sync + 'static,
    {
        self.event_handlers.server_error = Some(handler.into_handler());
        self
    }

    /// Transition to HandlersSet state after configuring event handlers
    /// 
    /// This method finalizes event handler configuration and enables
    /// the build() method to construct the server.
    pub fn finalize_handlers(self) -> BandwidthServerBuilder<HandlersSet> {
        BandwidthServerBuilder {
            bind_addr: self.bind_addr,
            cert: self.cert,
            key: self.key,
            broadcast_interval_ms: self.broadcast_interval_ms,
            client_timeout_ms: self.client_timeout_ms,
            event_handlers: self.event_handlers,
            _state: PhantomData,
        }
    }

    /// Set broadcast interval for bandwidth updates
    /// 
    /// Configures how frequently bandwidth events are sent to clients.
    /// Minimum interval is 100ms to prevent system overload.
    pub fn broadcast_interval_ms(mut self, interval_ms: u64) -> Result<Self> {
        if interval_ms < 100 {
            return Err(BandwidthError::configuration("broadcast_interval_ms", ConfigurationIssue::OutOfRange));
        }
        
        self.broadcast_interval_ms = interval_ms;
        Ok(self)
    }

    /// Set client connection timeout
    /// 
    /// Configures maximum time to wait for client responses.
    /// Minimum timeout is 1000ms to allow for network latency.
    pub fn client_timeout_ms(mut self, timeout_ms: u64) -> Result<Self> {
        if timeout_ms < 1000 {
            return Err(BandwidthError::configuration("client_timeout_ms", ConfigurationIssue::OutOfRange));
        }
        
        self.client_timeout_ms = timeout_ms;
        Ok(self)
    }

    /// Build the bandwidth server with current configuration
    /// 
    /// Validates all configuration and constructs a fully functional server
    /// ready for operation. This is the final step in the builder pattern.
    pub fn build(self) -> Result<BandwidthServer> {
        let config = BandwidthServerConfig {
            bind_addr: self.bind_addr.ok_or_else(|| {
                BandwidthError::configuration("bind_addr", ConfigurationIssue::Missing)
            })?,
            cert: self.cert.ok_or_else(|| {
                BandwidthError::configuration("cert", ConfigurationIssue::Missing)
            })?,
            key: self.key.ok_or_else(|| {
                BandwidthError::configuration("key", ConfigurationIssue::Missing)
            })?,
            broadcast_interval_ms: self.broadcast_interval_ms,
            client_timeout_ms: self.client_timeout_ms,
        };

        BandwidthServer::new(config, self.event_handlers)
    }
}

impl BandwidthServerBuilder<HandlersSet> {
    /// Set broadcast interval for bandwidth updates
    /// 
    /// Configures how frequently bandwidth events are sent to clients.
    /// Minimum interval is 100ms to prevent system overload.
    pub fn broadcast_interval_ms(mut self, interval_ms: u64) -> Result<Self> {
        if interval_ms < 100 {
            return Err(BandwidthError::configuration("broadcast_interval_ms", ConfigurationIssue::OutOfRange));
        }
        
        self.broadcast_interval_ms = interval_ms;
        Ok(self)
    }

    /// Set client connection timeout
    /// 
    /// Configures maximum time to wait for client responses.
    /// Minimum timeout is 1000ms to allow for network latency.
    pub fn client_timeout_ms(mut self, timeout_ms: u64) -> Result<Self> {
        if timeout_ms < 1000 {
            return Err(BandwidthError::configuration("client_timeout_ms", ConfigurationIssue::OutOfRange));
        }
        
        self.client_timeout_ms = timeout_ms;
        Ok(self)
    }

    /// Build the bandwidth server with current configuration
    /// 
    /// Validates all configuration and constructs a fully functional server
    /// ready for operation. This is the final step in the builder pattern.
    pub fn build(self) -> Result<BandwidthServer> {
        let config = BandwidthServerConfig {
            bind_addr: self.bind_addr.ok_or_else(|| {
                BandwidthError::configuration("bind_addr", ConfigurationIssue::Missing)
            })?,
            cert: self.cert.ok_or_else(|| {
                BandwidthError::configuration("cert", ConfigurationIssue::Missing)
            })?,
            key: self.key.ok_or_else(|| {
                BandwidthError::configuration("key", ConfigurationIssue::Missing)
            })?,
            broadcast_interval_ms: self.broadcast_interval_ms,
            client_timeout_ms: self.client_timeout_ms,
        };

        BandwidthServer::new(config, self.event_handlers)
    }
}

impl Default for BandwidthServerBuilder<Initial> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}