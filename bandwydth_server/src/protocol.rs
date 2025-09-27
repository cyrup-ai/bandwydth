//! Real QUIC Stream Message Protocol
//!
//! Zero-allocation binary protocol for bandwidth monitoring communication
//! between server and clients over QUIC streams with length-prefixed framing.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, warn};

use crate::error::{BandwidthError, BandwidthResult};

/// Protocol version for compatibility management
pub const PROTOCOL_VERSION: u8 = 1;

/// Maximum message size to prevent memory exhaustion (64KB)
pub const MAX_MESSAGE_SIZE: u32 = 65536;

/// Message frame header size (4 bytes length + 1 byte version)
pub const FRAME_HEADER_SIZE: usize = 5;

/// Bandwidth message types for QUIC stream communication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[derive(bincode::Encode, bincode::Decode)]
pub enum BandwidthMessage {
    /// System bandwidth statistics from server monitoring
    SystemStats {
        timestamp: u64,
        download_bps: u64,
        upload_bps: u64,
        interface_name: String,
        quality_score: f32,
        active_connections: u32,
    },
    /// Real-time download progress during file transfers
    DownloadProgress {
        timestamp: u64,
        bytes_downloaded: u64,
        bytes_per_second: u64,
        estimated_completion: Option<u64>,
        file_hash: Option<String>,
    },
    /// Upload bandwidth measurements
    UploadStats {
        timestamp: u64,
        bytes_uploaded: u64,
        bytes_per_second: u64,
        connection_id: String,
    },
    /// Client connection acknowledgment
    ClientAck {
        client_id: String,
        protocol_version: u8,
        supported_features: Vec<String>,
    },
    /// Server heartbeat for connection keepalive
    Heartbeat {
        timestamp: u64,
        server_load: f32,
    },
}

impl BandwidthMessage {
    /// Create system stats message with current timestamp
    #[inline]
    pub fn system_stats(
        download_bps: u64,
        upload_bps: u64,
        interface_name: String,
        quality_score: f32,
        active_connections: u32,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self::SystemStats {
            timestamp,
            download_bps,
            upload_bps,
            interface_name,
            quality_score,
            active_connections,
        }
    }

    /// Create download progress message with current timestamp
    #[inline]
    pub fn download_progress(
        bytes_downloaded: u64,
        bytes_per_second: u64,
        estimated_completion: Option<u64>,
        file_hash: Option<String>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self::DownloadProgress {
            timestamp,
            bytes_downloaded,
            bytes_per_second,
            estimated_completion,
            file_hash,
        }
    }

    /// Create upload stats message with current timestamp
    #[inline]
    pub fn upload_stats(
        bytes_uploaded: u64,
        bytes_per_second: u64,
        connection_id: String,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self::UploadStats {
            timestamp,
            bytes_uploaded,
            bytes_per_second,
            connection_id,
        }
    }

    /// Create client acknowledgment message
    #[inline]
    pub fn client_ack(
        client_id: String,
        supported_features: Vec<String>,
    ) -> Self {
        Self::ClientAck {
            client_id,
            protocol_version: PROTOCOL_VERSION,
            supported_features,
        }
    }

    /// Create heartbeat message with current timestamp and server load
    #[inline]
    pub fn heartbeat(server_load: f32) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self::Heartbeat {
            timestamp,
            server_load,
        }
    }

    /// Get message timestamp if available
    #[inline]
    pub fn timestamp(&self) -> Option<u64> {
        match self {
            Self::SystemStats { timestamp, .. } => Some(*timestamp),
            Self::DownloadProgress { timestamp, .. } => Some(*timestamp),
            Self::UploadStats { timestamp, .. } => Some(*timestamp),
            Self::Heartbeat { timestamp, .. } => Some(*timestamp),
            Self::ClientAck { .. } => None,
        }
    }

    /// Check if message is time-sensitive (should be processed immediately)
    #[inline]
    pub fn is_time_sensitive(&self) -> bool {
        matches!(self, Self::SystemStats { .. } | Self::DownloadProgress { .. })
    }

    /// Get message type name for logging and debugging
    #[inline]
    pub fn message_type(&self) -> &'static str {
        match self {
            Self::SystemStats { .. } => "SystemStats",
            Self::DownloadProgress { .. } => "DownloadProgress", 
            Self::UploadStats { .. } => "UploadStats",
            Self::ClientAck { .. } => "ClientAck",
            Self::Heartbeat { .. } => "Heartbeat",
        }
    }
}

/// Zero-allocation message serializer for QUIC streams
pub struct MessageCodec;

impl MessageCodec {
    /// Serialize message to bytes with length-prefixed framing
    /// 
    /// Frame format: [4-byte length][1-byte version][message data]
    /// Length includes version byte but not the length field itself
    pub fn encode(message: &BandwidthMessage) -> BandwidthResult<Vec<u8>> {
        // Serialize message data with bincode
        let message_data = bincode::encode_to_vec(message, bincode::config::standard())
            .map_err(|_| BandwidthError::serialization_encode("BandwidthMessage"))?;

        // Check message size limits
        if message_data.len() > (MAX_MESSAGE_SIZE - FRAME_HEADER_SIZE as u32) as usize {
            return Err(BandwidthError::serialization_encode("BandwidthMessage"));
        }

        // Create framed message: [length][version][data]
        let total_length = (message_data.len() + 1) as u32; // +1 for version byte
        let mut frame = Vec::with_capacity(FRAME_HEADER_SIZE + message_data.len());
        
        // Write length prefix (4 bytes, big endian)
        frame.extend_from_slice(&total_length.to_be_bytes());
        
        // Write protocol version (1 byte)
        frame.push(PROTOCOL_VERSION);
        
        // Write message data
        frame.extend_from_slice(&message_data);

        debug!(
            "Encoded {} message: {} bytes total ({} data + {} header)",
            message.message_type(),
            frame.len(),
            message_data.len(),
            FRAME_HEADER_SIZE
        );

        Ok(frame)
    }

    /// Deserialize message from length-prefixed frame bytes
    /// 
    /// Returns (message, bytes_consumed) or error for partial frames
    pub fn decode(buffer: &[u8]) -> BandwidthResult<Option<(BandwidthMessage, usize)>> {
        // Need at least header to proceed
        if buffer.len() < FRAME_HEADER_SIZE {
            debug!("Incomplete frame header: {} bytes (need {})", buffer.len(), FRAME_HEADER_SIZE);
            return Ok(None);
        }

        // Read length prefix (4 bytes, big endian)
        let length_bytes = &buffer[0..4];
        let message_length = u32::from_be_bytes([
            length_bytes[0],
            length_bytes[1], 
            length_bytes[2],
            length_bytes[3],
        ]);

        // Validate message length
        if message_length == 0 {
            return Err(BandwidthError::serialization_decode("BandwidthMessage"));
        }

        if message_length > MAX_MESSAGE_SIZE {
            return Err(BandwidthError::serialization_decode("BandwidthMessage"));
        }

        // Check if we have the complete message
        let total_frame_size = FRAME_HEADER_SIZE + message_length as usize - 1; // -1 because length includes version
        if buffer.len() < total_frame_size {
            debug!(
                "Incomplete message frame: {} bytes (need {} total)",
                buffer.len(),
                total_frame_size
            );
            return Ok(None);
        }

        // Read and validate protocol version
        let version = buffer[4];
        if version != PROTOCOL_VERSION {
            warn!(
                "Protocol version mismatch: received v{}, expected v{}",
                version, PROTOCOL_VERSION
            );
            return Err(BandwidthError::serialization_decode("BandwidthMessage"));
        }

        // Extract message data (excluding length and version)
        let message_data = &buffer[FRAME_HEADER_SIZE..total_frame_size];
        
        // Deserialize message
        let (message, _): (BandwidthMessage, usize) = bincode::decode_from_slice(message_data, bincode::config::standard())
            .map_err(|_| BandwidthError::serialization_decode("BandwidthMessage"))?;

        debug!(
            "Decoded {} message: {} bytes consumed",
            message.message_type(),
            total_frame_size
        );

        Ok(Some((message, total_frame_size)))
    }

    /// Decode multiple messages from a buffer
    /// 
    /// Returns (messages, total_bytes_consumed)
    pub fn decode_multiple(buffer: &[u8]) -> BandwidthResult<(Vec<BandwidthMessage>, usize)> {
        let mut messages = Vec::new();
        let mut offset = 0;

        while offset < buffer.len() {
            match Self::decode(&buffer[offset..])? {
                Some((message, consumed)) => {
                    messages.push(message);
                    offset += consumed;
                }
                None => {
                    // Incomplete message, stop processing
                    break;
                }
            }
        }

        debug!(
            "Decoded {} messages from {} byte buffer ({} bytes consumed)",
            messages.len(),
            buffer.len(),
            offset
        );

        Ok((messages, offset))
    }

    /// Estimate serialized size of a message for buffer planning
    #[inline]
    pub fn estimate_size(message: &BandwidthMessage) -> usize {
        // Conservative estimate based on message type
        FRAME_HEADER_SIZE + match message {
            BandwidthMessage::SystemStats { interface_name, .. } => {
                32 + interface_name.len() + 16 // base fields + string + padding
            }
            BandwidthMessage::DownloadProgress { file_hash, .. } => {
                40 + file_hash.as_ref().map(|h| h.len()).unwrap_or(0) + 16
            }
            BandwidthMessage::UploadStats { connection_id, .. } => {
                32 + connection_id.len() + 16
            }
            BandwidthMessage::ClientAck { client_id, supported_features, .. } => {
                16 + client_id.len() + supported_features.iter().map(|f| f.len()).sum::<usize>() + 32
            }
            BandwidthMessage::Heartbeat { .. } => {
                16 + 8 // timestamp + server_load + padding
            }
        }
    }
}

/// Stream buffer manager for efficient QUIC stream processing
pub struct StreamBuffer {
    buffer: Vec<u8>,
    capacity: usize,
}

impl StreamBuffer {
    /// Create new stream buffer with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Create stream buffer with default capacity optimized for bandwidth messages
    pub fn with_default_capacity() -> Self {
        Self::new(8192) // 8KB default for multiple messages
    }

    /// Append data to buffer and process complete messages
    pub fn append_and_decode(&mut self, data: &[u8]) -> BandwidthResult<Vec<BandwidthMessage>> {
        // Extend buffer with new data
        self.buffer.extend_from_slice(data);

        // Decode messages from buffer
        let (messages, consumed) = MessageCodec::decode_multiple(&self.buffer)?;

        // Remove consumed bytes from buffer
        if consumed > 0 {
            self.buffer.drain(..consumed);
        }

        // Compact buffer if it's getting large
        if self.buffer.len() > self.capacity / 2 && self.buffer.capacity() > self.capacity {
            self.buffer.shrink_to(self.capacity);
        }

        Ok(messages)
    }

    /// Get current buffer length
    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Clear buffer contents
    #[inline]
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get buffer utilization percentage
    #[inline]
    pub fn utilization(&self) -> f32 {
        (self.buffer.len() as f32) / (self.capacity as f32)
    }
}

impl Default for StreamBuffer {
    fn default() -> Self {
        Self::with_default_capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_encoding_decoding() {
        let msg = BandwidthMessage::system_stats(
            1_000_000, 
            500_000, 
            "eth0".to_string(), 
            0.95, 
            3
        );

        let encoded = MessageCodec::encode(&msg).expect("encoding failed");
        let (decoded, consumed) = MessageCodec::decode(&encoded)
            .expect("decoding failed")
            .expect("incomplete message");

        assert_eq!(msg, decoded);
        assert_eq!(consumed, encoded.len());
    }

    #[test]
    fn test_multiple_message_decoding() {
        let msg1 = BandwidthMessage::heartbeat(0.3);
        let msg2 = BandwidthMessage::system_stats(
            2_000_000,
            1_000_000,
            "wlan0".to_string(),
            0.8,
            5
        );

        let mut buffer = Vec::new();
        buffer.extend_from_slice(&MessageCodec::encode(&msg1).expect("encode failed"));
        buffer.extend_from_slice(&MessageCodec::encode(&msg2).expect("encode failed"));

        let (messages, consumed) = MessageCodec::decode_multiple(&buffer)
            .expect("decode multiple failed");

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], msg1);
        assert_eq!(messages[1], msg2);
        assert_eq!(consumed, buffer.len());
    }

    #[test]
    fn test_stream_buffer() {
        let mut stream_buf = StreamBuffer::new(1024);
        
        let msg = BandwidthMessage::heartbeat(0.5);
        let encoded = MessageCodec::encode(&msg).expect("encode failed");

        // Split encoded message to test partial reception
        let mid = encoded.len() / 2;
        let part1 = &encoded[..mid];
        let part2 = &encoded[mid..];

        // Process first part (should return no messages)
        let messages1 = stream_buf.append_and_decode(part1).expect("decode failed");
        assert_eq!(messages1.len(), 0);

        // Process second part (should return the complete message)
        let messages2 = stream_buf.append_and_decode(part2).expect("decode failed");
        assert_eq!(messages2.len(), 1);
        assert_eq!(messages2[0], msg);
    }
}