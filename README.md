# ProgressHub Bandwidth Monitor

A cross-platform network bandwidth monitoring library for Rust that provides real-time network statistics and bandwidth classification.

## Features

- **Real-time Monitoring**: Continuously monitor network bandwidth with configurable polling intervals
- **Cross-platform Support**: Works on Linux, macOS, Windows, and BSD systems
- **Bandwidth Classification**: Automatically classifies bandwidth as Excellent, Good, Fair, or Poor
- **Historical Data**: Maintains configurable history of bandwidth measurements
- **DNS Resolution**: Optional background DNS resolution for network endpoints
- **Thread-safe**: Designed for multi-threaded applications

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
progresshub-bandwidth = "0.1.0"
```

## Quick Start

```rust
use progresshub_bandwidth::{start_monitor, MonitorConfig};
use std::{thread, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = MonitorConfig::default();
    
    // Start monitoring with a callback
    let handle = start_monitor(config, |stats| {
        println!("Bandwidth: {:.2} MB/s ({:?})", 
                 stats.current_speed, 
                 stats.bandwidth_class);
    })?;
    
    // Let it run for a while
    thread::sleep(Duration::from_secs(10));
    
    // Stop monitoring
    handle.stop();
    Ok(())
}
```

## Configuration

The `MonitorConfig` struct allows you to customize the monitoring behavior:

```rust
use progresshub_bandwidth::MonitorConfig;

let config = MonitorConfig {
    interface: Some("eth0".to_string()),  // Specific interface (None = auto-detect)
    resolve_dns: true,                    // Enable DNS resolution
    dns_server: None,                     // Custom DNS server (None = system default)
    allow_system_fallback: true,          // Fallback if raw sockets unavailable
    period_secs: 1,                       // Polling interval in seconds
};
```

## Bandwidth Classification

The library automatically classifies bandwidth quality:

- **Excellent**: > 800 Mbps
- **Good**: 400-800 Mbps  
- **Fair**: 160-400 Mbps
- **Poor**: < 160 Mbps

## Platform Support

### Linux
- Uses `/proc/net/dev` for optimal performance
- Supports all network interfaces
- No additional dependencies

### macOS/BSD/Windows
- Uses the `sysinfo` crate for cross-platform compatibility
- Supports all major platforms
- Automatic fallback from Linux-specific code

## Examples

Run the included example:

```bash
cargo run --example simple_monitor
```

## Architecture

- **Monitor**: Main monitoring loop and thread management
- **Aggregator**: Calculates bandwidth from interface statistics deltas
- **Interface Stats**: Cross-platform network interface statistics reader
- **DNS Agent**: Background DNS resolution service
- **Types**: Core data structures and bandwidth classification

## DNS Resolution

Optional background DNS resolution for network endpoints:

```rust
use progresshub_bandwidth::DnsAgent;
use std::net::IpAddr;

let dns = DnsAgent::spawn();

// Submit IP for resolution
let ip: IpAddr = "8.8.8.8".parse().unwrap();
dns.submit(ip);

// Later, check if resolved
if let Some(hostname) = dns.lookup(ip) {
    println!("IP {} resolved to {}", ip, hostname);
}
```

## Error Handling

The library uses standard Rust error handling:

- `std::io::Result` for I/O operations
- Graceful fallbacks for unsupported platforms
- Non-blocking DNS resolution with error caching

## Performance

- Minimal CPU overhead (single background thread)
- Configurable polling intervals (1+ seconds recommended)
- Memory usage scales with history buffer size
- Zero-copy interface statistics reading where possible

## Requirements

- Rust 1.70+ (2021 edition)
- Platform-specific network access (may require elevated privileges for raw sockets on some systems)

## License

Licensed under either of:
- MIT License
- Apache License, Version 2.0

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Changelog

### v0.1.0
- Initial release
- Cross-platform bandwidth monitoring
- Background DNS resolution
- Configurable polling and history
- Bandwidth quality classification
