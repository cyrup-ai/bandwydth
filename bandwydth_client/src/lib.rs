//! High-performance bandwidth monitoring client with QUIC transport
//!
//! This crate provides a bandwidth monitoring client implementation
//! using post-quantum QUIC transport.

mod builder;
mod client;
mod error;
mod events;
mod types;

pub use builder::BandwidthClientBuilder;
pub use client::{BandwidthClient, ConnectionState};
pub use error::{BandwidthError, Result};
pub use events::EventHandlers;
pub use types::{
    BandwidthEvent,
    BandwidthClientConfig,
    get_timestamp_millis,
};

// Re-export protocol from server for shared message format
pub use bandwydth_server::{BandwidthMessage, MessageCodec, StreamBuffer, PROTOCOL_VERSION};