// src/network/bandwidth.rs
use std::{
    collections::HashMap,
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::network::r#type::{NetBandwidthClass, NetBandwidthStats, NetUsageStatus};

#[cfg(target_os = "linux")]
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

// Types for network statistics
/// Statistics for a single network interface with static lifetime.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NetInterfaceStats {
    /// Interface name (e.g., "eth0", "wlan0").
    pub name: String,
    /// Total bytes received since last measurement.
    pub received_bytes: u64,
    /// Total bytes transmitted since last measurement.
    pub transmitted_bytes: u64,
}

/// Snapshot of all network interface statistics at a point in time.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NetSnapshot {
    /// Map of interface names to their current statistics.
    pub interfaces: HashMap<String, NetInterfaceStats>,
}

// All network bandwidth types are imported from r#type module

// Aggregator for bandwidth calculations
/// Aggregates network snapshots into bandwidth statistics using statistical analysis.
#[allow(dead_code)]
pub struct NetAggregator<const N: usize> {
    last: Option<NetSnapshot>,
    history: [f64; N],
    history_index: usize,
    mean: f64,
    m2: f64, // For Welford's variance calculation
    count: usize,
    interval: Duration,
    high_threshold: f64, // Mbps
    cv_threshold: f64,
}

#[allow(dead_code)]
impl<const N: usize> NetAggregator<N> {
    /// Create a new aggregator with specified interval and thresholds.
    pub const fn new(interval: Duration, high_threshold: f64, cv_threshold: f64) -> Self {
        Self {
            last: None,
            history: [0.0; N],
            history_index: 0,
            mean: 0.0,
            m2: 0.0,
            count: 0,
            interval,
            high_threshold,
            cv_threshold,
        }
    }

    /// Update with a new snapshot and calculate bandwidth.
    pub fn update(&mut self, snapshot: NetSnapshot) -> Option<NetBandwidthStats<N>> {
        let last = self.last.replace(snapshot.clone())?;

        let mut total_delta_bytes = 0u64;
        for (name, current) in &snapshot.interfaces {
            if *name == "lo" {
                continue;
            }
            if let Some(prev) = last.interfaces.get(name) {
                let rx = current.received_bytes.saturating_sub(prev.received_bytes);
                let tx = current
                    .transmitted_bytes
                    .saturating_sub(prev.transmitted_bytes);
                total_delta_bytes += rx + tx;
            }
        }

        let seconds = self.interval.as_secs_f64();
        let mbps = (total_delta_bytes as f64 * 8.0) / (1024.0 * 1024.0 * seconds);

        // Welford's online algorithm for mean and variance
        self.count = self.count.saturating_add(1);
        let delta = mbps - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = mbps - self.mean;
        self.m2 += delta * delta2;

        // Update history
        self.history[self.history_index] = mbps;
        self.history_index = (self.history_index + 1) % N;

        let usage_status = if self.count >= N {
            let variance = self.m2 / N as f64;
            let sd = variance.sqrt();
            let cv = if self.mean > 0.0 {
                sd / self.mean
            } else {
                f64::INFINITY
            };
            if self.mean > self.high_threshold && cv < self.cv_threshold {
                NetUsageStatus::SteadyHigh
            } else {
                NetUsageStatus::Inconclusive
            }
        } else {
            NetUsageStatus::Inconclusive
        };

        Some(NetBandwidthStats {
            current_speed: mbps,
            bandwidth_history: self.history,
            bandwidth_class: classify_bandwidth(mbps, &self.history, self.count),
            usage_status,
        })
    }
}

/// Blazing-fast bandwidth classification with statistical analysis
///
/// Analyzes current speed and historical patterns to determine bandwidth class.
/// Zero-allocation design works directly with fixed-size arrays.
#[inline(always)]
pub fn classify_bandwidth<const N: usize>(
    current_mbps: f64,
    history: &[f64; N],
    sample_count: usize,
) -> NetBandwidthClass {
    // Fast path: classify based on current speed first
    let speed_class = match current_mbps {
        x if x >= 10000.0 => NetBandwidthClass::Blazing, // 10+ Gbps
        x if x >= 1000.0 => NetBandwidthClass::Blazing,  // 1+ Gbps
        x if x >= 500.0 => NetBandwidthClass::Good,      // 500+ Mbps
        x if x >= 100.0 => NetBandwidthClass::Average,   // 100+ Mbps
        x if x >= 10.0 => NetBandwidthClass::Poor,       // 10+ Mbps
        _ => NetBandwidthClass::Inconclusive,
    };

    // Need sufficient samples for statistical analysis
    if sample_count < N.min(5) {
        return speed_class;
    }

    // Calculate percentiles for consistency check
    let samples = sample_count.min(N);
    let mut sorted = [0.0; N];
    sorted[..samples].copy_from_slice(&history[..samples]);

    // Fast partial sort for percentiles (only sort what we need)
    let p25_idx = samples / 4;
    let p75_idx = (samples * 3) / 4;

    // Find 25th percentile using partial ordering for f64
    sorted[..samples].select_nth_unstable_by(p25_idx, |a, b| {
        a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
    });
    let p25 = sorted[p25_idx];

    // Find 75th percentile using partial ordering for f64
    sorted[p25_idx..samples].select_nth_unstable_by(p75_idx - p25_idx, |a, b| {
        a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
    });
    let p75 = sorted[p75_idx];

    // If current speed is significantly below historical performance, downgrade
    if current_mbps < p25 * 0.5 {
        match speed_class {
            NetBandwidthClass::Blazing => NetBandwidthClass::Good,
            NetBandwidthClass::Good => NetBandwidthClass::Average,
            NetBandwidthClass::Average => NetBandwidthClass::Poor,
            other => other,
        }
    } else if current_mbps > p75 * 1.5 && samples >= N / 2 {
        // If current speed is significantly above historical, we might be bursting
        // Keep current classification but note this for usage status
        speed_class
    } else {
        speed_class
    }
}

/// Reads current network interface statistics from the system.
///
/// On Linux, reads from `/proc/net/dev`. On other platforms, returns empty snapshot.
pub fn read_snapshot() -> io::Result<NetSnapshot> {
    #[cfg(target_os = "linux")]
    {
        read_snapshot_linux()
    }

    #[cfg(not(target_os = "linux"))]
    {
        read_snapshot_fallback()
    }
}

#[cfg(target_os = "linux")]
fn read_snapshot_linux() -> io::Result<NetSnapshot> {
    let file = File::open("/proc/net/dev")?;
    let reader = BufReader::new(file);
    let mut interfaces = HashMap::new();

    for line in reader.lines().skip(2) {
        let line = line?;
        let parts: Vec<&str> = line.trim().split(':').collect();
        if parts.len() != 2 {
            continue;
        }

        let name = parts[0].trim();
        let data: Vec<&str> = parts[1].trim().split_whitespace().collect();
        if data.len() < 16 {
            continue;
        }

        let rx_bytes = data[0].parse::<u64>().unwrap_or(0);
        let tx_bytes = data[8].parse::<u64>().unwrap_or(0);

        interfaces.insert(
            name.clone(),
            NetInterfaceStats {
                name,
                received_bytes: rx_bytes,
                transmitted_bytes: tx_bytes,
            },
        );
    }

    Ok(NetSnapshot { interfaces })
}

#[cfg(target_os = "macos")]
fn read_snapshot_fallback() -> io::Result<NetSnapshot> {
    use std::process::Command;
    use std::time::Duration;
    use std::sync::mpsc;
    use std::thread;
    
    // Use netstat on macOS to get interface statistics with robust parsing
    log::debug!("Executing netstat command for interface statistics");
    
    // Implement timeout using thread-based approach
    let (tx, rx) = mpsc::channel();
    let _handle = thread::spawn(move || {
        let result = Command::new("netstat")
            .args(&["-i", "-b"])
            .output();
        let _ = tx.send(result);
    });
    
    let output = match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(result) => result?,
        Err(_) => {
            log::warn!("netstat command timed out after 10 seconds");
            return Ok(NetSnapshot::default());
        }
    };
    
    if !output.status.success() {
        log::warn!("netstat command failed with status: {}, stderr: {}", 
                  output.status, String::from_utf8_lossy(&output.stderr));
        return Ok(NetSnapshot::default());
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut interfaces = std::collections::HashMap::new();
    let lines: Vec<&str> = stdout.lines().collect();
    
    if lines.is_empty() {
        log::warn!("netstat returned empty output");
        return Ok(NetSnapshot::default());
    }
    
    // Validate header line to ensure we're parsing the right format
    let header = lines[0];
    if !header.contains("Name") || !header.contains("Ibytes") || !header.contains("Obytes") {
        log::warn!("netstat output format doesn't match expected headers: {}", header);
        log::debug!("Full netstat output:\n{}", stdout);
        return Ok(NetSnapshot::default());
    }
    
    // Find column indices dynamically instead of hardcoding
    let header_parts: Vec<&str> = header.split_whitespace().collect();
    let ibytes_idx = header_parts.iter().position(|&x| x == "Ibytes");
    let obytes_idx = header_parts.iter().position(|&x| x == "Obytes");
    
    let (ibytes_idx, obytes_idx) = match (ibytes_idx, obytes_idx) {
        (Some(rx), Some(tx)) => (rx, tx),
        _ => {
            log::warn!("Could not find Ibytes/Obytes columns in netstat output");
            log::debug!("Header parts: {:?}", header_parts);
            return Ok(NetSnapshot::default());
        }
    };
    
    log::debug!("Found Ibytes at column {}, Obytes at column {}", ibytes_idx, obytes_idx);
    
    // Parse data lines with robust error handling
    for (line_num, line) in lines.iter().skip(1).enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        // Validate we have enough columns
        let max_required_idx = ibytes_idx.max(obytes_idx);
        if parts.len() <= max_required_idx {
            log::debug!("Line {} has insufficient columns ({}), need at least {}, skipping: {}", 
                       line_num + 2, parts.len(), max_required_idx + 1, line);
            continue;
        }
        
        let name = parts[0];
        
        // Skip invalid interface names
        if name.is_empty() || name == "-" || name.starts_with('*') {
            log::debug!("Skipping interface with invalid name: '{}'", name);
            continue;
        }
        
        // Parse byte counts with validation
        let rx_bytes = match parts[ibytes_idx].parse::<u64>() {
            Ok(val) => val,
            Err(e) => {
                log::debug!("Failed to parse rx_bytes '{}' for interface {}: {}, skipping", 
                           parts[ibytes_idx], name, e);
                continue;
            }
        };
        
        let tx_bytes = match parts[obytes_idx].parse::<u64>() {
            Ok(val) => val,
            Err(e) => {
                log::debug!("Failed to parse tx_bytes '{}' for interface {}: {}, skipping", 
                           parts[obytes_idx], name, e);
                continue;
            }
        };
        
        log::debug!("Parsed interface {}: rx={} bytes, tx={} bytes", name, rx_bytes, tx_bytes);
        
        interfaces.insert(
            name.to_string(),
            NetInterfaceStats {
                name: name.to_string(),
                received_bytes: rx_bytes,
                transmitted_bytes: tx_bytes,
            },
        );
    }
    
    if interfaces.is_empty() {
        log::debug!("netstat fallback found no valid interfaces, using empty snapshot");
    } else {
        log::debug!("netstat fallback found {} interfaces: {:?}", 
                   interfaces.len(), interfaces.keys().collect::<Vec<_>>());
    }
    
    Ok(NetSnapshot { interfaces })
}

#[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
fn read_snapshot_fallback() -> io::Result<NetSnapshot> {
    // Minimal fallback for other platforms
    Ok(NetSnapshot::default())
}

// Monitor control
/// Handle for controlling a running network bandwidth monitor.
pub struct NetMonitorHandle {
    running: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl NetMonitorHandle {
    /// Stops the monitoring thread and waits for it to complete.
    pub fn stop(self) {
        self.running.store(false, Ordering::Release);
        if let Some(handle) = self.thread {
            let _ = handle.join();
        }
    }
}

use crate::network::r#type::Snapshot;

impl From<Snapshot> for NetSnapshot {
    fn from(snapshot: Snapshot) -> Self {
        let interfaces = snapshot
            .interfaces
            .into_iter()
            .map(|(name, stats)| {
                let net_stats = NetInterfaceStats {
                    name: name.clone(),
                    received_bytes: stats.received_bytes,
                    transmitted_bytes: stats.transmitted_bytes,
                };
                (name, net_stats)
            })
            .collect();

        NetSnapshot { interfaces }
    }
}
