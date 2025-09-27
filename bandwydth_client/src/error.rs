//! Comprehensive error handling for bandwidth monitoring client
//!
//! Zero-allocation error types with detailed context and recovery information
//! optimized for client-side operations and connection management

use std::fmt;
use std::io;
use std::net::AddrParseError;
use std::time::SystemTimeError;

/// Result type alias for bandwidth client operations
pub type Result<T> = core::result::Result<T, BandwidthError>;

/// Comprehensive error enumeration for all bandwidth client operations
#[derive(Debug)]
pub enum BandwidthError {
    /// QUIC transport layer errors
    Quic {
        message: &'static str,
        context: Option<String>,
    },
    
    /// Connection establishment and management errors
    Connection {
        kind: ConnectionError,
        server_address: Option<String>,
    },
    
    /// Message serialization and deserialization errors
    Serialization {
        operation: SerializationOp,
        type_name: &'static str,
    },
    
    /// Server communication and protocol errors
    Server {
        error: ServerError,
        context: String,
    },
    
    /// Client configuration and validation errors
    Configuration {
        field: &'static str,
        issue: ConfigurationIssue,
    },
    
    /// System time and timestamp errors
    Time {
        operation: &'static str,
        source: SystemTimeError,
    },
    
    /// I/O operations (network, file, etc.)
    Io {
        operation: &'static str,
        source: io::Error,
    },
    
    /// Event handler execution errors
    EventHandler {
        handler_type: &'static str,
        message: String,
    },
    
    /// Authentication and certificate validation errors
    Authentication {
        reason: AuthError,
        details: String,
    },
}

#[derive(Debug, Clone)]
pub enum ConnectionError {
    Failed,
    Timeout,
    ServerUnreachable,
    HandshakeFailed,
    Disconnected,
    NetworkUnreachable,
    PermissionDenied,
    InvalidServerAddress,
}

#[derive(Debug, Clone)]
pub enum SerializationOp {
    Encode,
    Decode,
}

#[derive(Debug, Clone)]
pub enum ServerError {
    ProtocolViolation,
    UnexpectedDisconnection,
    InvalidResponse,
    ServerOverloaded,
    ServiceUnavailable,
    InternalError,
}

#[derive(Debug, Clone)]
pub enum ConfigurationIssue {
    Missing,
    Invalid,
    OutOfRange,
    Conflict,
}

#[derive(Debug, Clone)]
pub enum AuthError {
    InvalidCertificate,
    UntrustedCertificate,
    CertificateExpired,
    AuthenticationFailed,
    ProtocolMismatch,
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
    pub fn connection(kind: ConnectionError, server_address: Option<String>) -> Self {
        Self::Connection { kind, server_address }
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
    pub fn server_error(error: ServerError, context: String) -> Self {
        Self::Server { error, context }
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
    
    #[inline]
    pub fn authentication_error(reason: AuthError, details: String) -> Self {
        Self::Authentication { reason, details }
    }
    
    /// Check if error indicates a recoverable condition
    #[inline]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Connection { 
                kind: ConnectionError::Timeout | ConnectionError::NetworkUnreachable, 
                .. 
            } |
            Self::Server { 
                error: ServerError::ServerOverloaded | ServerError::ServiceUnavailable, 
                .. 
            } |
            Self::EventHandler { .. }
        )
    }
    
    /// Check if error requires reconnection
    #[inline]
    pub const fn requires_reconnection(&self) -> bool {
        matches!(
            self,
            Self::Connection { 
                kind: ConnectionError::Disconnected | ConnectionError::HandshakeFailed,
                .. 
            } |
            Self::Server { 
                error: ServerError::UnexpectedDisconnection,
                .. 
            }
        )
    }
    
    /// Check if error is permanent and should not be retried
    #[inline]
    pub const fn is_permanent(&self) -> bool {
        matches!(
            self,
            Self::Configuration { .. } |
            Self::Authentication { .. } |
            Self::Connection {
                kind: ConnectionError::PermissionDenied | ConnectionError::InvalidServerAddress,
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
            Self::Connection { kind, server_address } => {
                write!(f, "Connection error: {:?}", kind)?;
                if let Some(addr) = server_address {
                    write!(f, " to server '{}'", addr)?;
                }
                Ok(())
            }
            Self::Serialization { operation, type_name } => {
                write!(f, "Serialization error: failed to {:?} {}", operation, type_name)
            }
            Self::Server { error, context } => {
                write!(f, "Server error: {:?} ({})", error, context)
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
            Self::Authentication { reason, details } => {
                write!(f, "Authentication error: {:?} ({})", reason, details)
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
        Self::connection(ConnectionError::InvalidServerAddress, None)
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