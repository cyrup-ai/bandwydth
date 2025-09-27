//! Comprehensive error handling for bandwidth monitoring server
//!
//! Zero-allocation error types with detailed context and recovery information

use std::fmt;
use std::io;
use std::net::AddrParseError;
use std::time::SystemTimeError;

/// Result type alias for bandwidth server operations
pub type Result<T> = core::result::Result<T, BandwidthError>;

/// Type alias for backward compatibility
pub type BandwidthResult<T> = Result<T>;

/// Comprehensive error enumeration for all bandwidth server operations
#[derive(Debug)]
pub enum BandwidthError {
    /// QUIC transport layer errors
    Quic {
        message: &'static str,
        context: Option<String>,
    },
    
    /// Certificate and TLS configuration errors
    Certificate {
        reason: CertificateError,
        context: String,
    },
    
    /// Network address and binding errors  
    Network {
        kind: NetworkError,
        address: Option<String>,
    },
    
    /// Message serialization and deserialization errors
    Serialization {
        operation: SerializationOp,
        type_name: &'static str,
    },
    
    /// Client connection and lifecycle errors
    Client {
        client_id: Option<String>,
        error: ClientError,
    },
    
    /// Server configuration and validation errors
    Configuration {
        field: &'static str,
        issue: ConfigurationIssue,
    },
    
    /// System time and timestamp errors
    Time {
        operation: &'static str,
        source: SystemTimeError,
    },
    
    /// I/O operations (file, network, etc.)
    Io {
        operation: &'static str,
        source: io::Error,
    },
    
    /// Event handler execution errors
    EventHandler {
        handler_type: &'static str,
        message: String,
    },
}

#[derive(Debug, Clone)]
pub enum CertificateError {
    InvalidFormat,
    ExpiredCertificate,
    UntrustedCertificate,
    KeyMismatch,
    MissingPrivateKey,
    InvalidPrivateKey,
}

#[derive(Debug, Clone)]
pub enum NetworkError {
    InvalidAddress,
    AddressInUse,
    AddressNotAvailable,
    PermissionDenied,
    ConnectionRefused,
    Timeout,
    UnreachableHost,
}

#[derive(Debug, Clone)]
pub enum SerializationOp {
    Encode,
    Decode,
}

#[derive(Debug, Clone)]
pub enum ClientError {
    HandshakeFailed,
    Disconnected,
    MessageTooLarge,
    InvalidMessage,
    Timeout,
    ProtocolViolation,
}

#[derive(Debug, Clone)]
pub enum ConfigurationIssue {
    Missing,
    Invalid,
    OutOfRange,
    Conflict,
}

impl BandwidthError {
    #[inline]
    pub fn quic(message: &'static str) -> Self {
        Self::Quic {
            message,
            context: None,
        }
    }
    
    #[inline] 
    pub fn quic_with_context(message: &'static str, context: String) -> Self {
        Self::Quic {
            message,
            context: Some(context),
        }
    }
    
    #[inline]
    pub fn certificate(reason: CertificateError, context: String) -> Self {
        Self::Certificate { reason, context }
    }
    
    #[inline]
    pub fn network(kind: NetworkError, address: Option<String>) -> Self {
        Self::Network { kind, address }
    }
    
    #[inline]
    pub fn serialization_encode(type_name: &'static str) -> Self {
        Self::Serialization {
            operation: SerializationOp::Encode,
            type_name,
        }
    }
    
    #[inline]
    pub fn serialization_decode(type_name: &'static str) -> Self {
        Self::Serialization {
            operation: SerializationOp::Decode,
            type_name,
        }
    }
    
    #[inline]
    pub fn client_error(client_id: Option<String>, error: ClientError) -> Self {
        Self::Client { client_id, error }
    }
    
    #[inline]
    pub fn configuration(field: &'static str, issue: ConfigurationIssue) -> Self {
        Self::Configuration { field, issue }
    }
    
    #[inline]
    pub fn time_error(operation: &'static str, source: SystemTimeError) -> Self {
        Self::Time { operation, source }
    }
    
    #[inline]
    pub fn io_error(operation: &'static str, source: io::Error) -> Self {
        Self::Io { operation, source }
    }
    
    #[inline]
    pub fn event_handler_error(handler_type: &'static str, message: String) -> Self {
        Self::EventHandler { handler_type, message }
    }
    
    /// Check if error indicates a recoverable condition
    #[inline]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Client { .. } | 
            Self::Network { kind: NetworkError::Timeout | NetworkError::ConnectionRefused, .. } |
            Self::EventHandler { .. }
        )
    }
    
    /// Check if error requires immediate shutdown
    #[inline]
    pub const fn requires_shutdown(&self) -> bool {
        matches!(
            self,
            Self::Certificate { .. } |
            Self::Configuration { .. } |
            Self::Network { 
                kind: NetworkError::AddressInUse | NetworkError::PermissionDenied, 
                .. 
            }
        )
    }
}

impl fmt::Display for BandwidthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Quic { message, context } => {
                write!(f, "QUIC error: {}", message)?;
                if let Some(ctx) = context {
                    write!(f, " ({})", ctx)?;
                }
                Ok(())
            }
            Self::Certificate { reason, context } => {
                write!(f, "Certificate error: {:?} ({})", reason, context)
            }
            Self::Network { kind, address } => {
                write!(f, "Network error: {:?}", kind)?;
                if let Some(addr) = address {
                    write!(f, " for address '{}'", addr)?;
                }
                Ok(())
            }
            Self::Serialization { operation, type_name } => {
                write!(f, "Serialization error: failed to {:?} {}", operation, type_name)
            }
            Self::Client { client_id, error } => {
                write!(f, "Client error: {:?}", error)?;
                if let Some(id) = client_id {
                    write!(f, " (client: {})", id)?;
                }
                Ok(())
            }
            Self::Configuration { field, issue } => {
                write!(f, "Configuration error: {} is {:?}", field, issue)
            }
            Self::Time { operation, source } => {
                write!(f, "Time error during {}: {}", operation, source)
            }
            Self::Io { operation, source } => {
                write!(f, "I/O error during {}: {}", operation, source)
            }
            Self::EventHandler { handler_type, message } => {
                write!(f, "Event handler error in {}: {}", handler_type, message)
            }
        }
    }
}

impl std::error::Error for BandwidthError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Time { source, .. } => Some(source),
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<io::Error> for BandwidthError {
    #[inline]
    fn from(error: io::Error) -> Self {
        Self::io_error("unknown", error)
    }
}

impl From<AddrParseError> for BandwidthError {
    #[inline]
    fn from(_: AddrParseError) -> Self {
        Self::network(NetworkError::InvalidAddress, None)
    }
}

impl From<SystemTimeError> for BandwidthError {
    #[inline]
    fn from(error: SystemTimeError) -> Self {
        Self::time_error("unknown", error)
    }
}

impl From<bincode::error::EncodeError> for BandwidthError {
    #[inline]
    fn from(_: bincode::error::EncodeError) -> Self {
        Self::serialization_encode("unknown")
    }
}

impl From<bincode::error::DecodeError> for BandwidthError {
    #[inline]
    fn from(_: bincode::error::DecodeError) -> Self {
        Self::serialization_decode("unknown")
    }
}