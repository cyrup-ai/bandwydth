//! High-performance bandwidth monitoring server with QUIC transport
//!
//! This crate provides a bandwidth monitoring server implementation
//! using post-quantum QUIC transport.

mod builder;
mod client;
mod error;
mod events;
mod protocol;
mod server;
mod stats;
mod types;

pub use builder::BandwidthServerBuilder;
pub use error::{BandwidthError, Result};
pub use events::EventHandlers;
pub use protocol::{BandwidthMessage, MessageCodec, StreamBuffer, PROTOCOL_VERSION};
pub use server::BandwidthServer;
pub use stats::{AtomicBandwidthStats, BandwidthMonitor};
pub use client::ClientHandler;
pub use types::{
    BandwidthEvent,
    BandwidthClass,
    BandwidthServerConfig,
    BandwidthData,
    ClientInfo,
};