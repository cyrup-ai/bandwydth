use std::collections::HashMap;

#[cfg(target_os = "linux")]
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[cfg(not(target_os = "linux"))]
use sysinfo::Networks;

use crate::network::r#type::{InterfaceStats, Snapshot};

/// Read current network interface statistics from the system.
///
/// On Linux, this reads from `/proc/net/dev`. Returns statistics for all
/// network interfaces including bytes received and transmitted.
///
/// # Returns
///
/// A `Snapshot` containing current statistics for all network interfaces.
///
/// # Errors
///
/// Returns an I/O error if the system statistics cannot be read.
pub fn read_snapshot() -> std::io::Result<Snapshot> {
    #[cfg(target_os = "linux")]
    {
        read_snapshot_linux()
    }

    #[cfg(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    {
        read_snapshot_unix()
    }

    #[cfg(target_os = "windows")]
    {
        read_snapshot_windows()
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    {
        // Fallback for unsupported platforms
        Ok(Snapshot::default())
    }
}

#[cfg(target_os = "linux")]
fn read_snapshot_linux() -> std::io::Result<Snapshot> {
    let file = File::open("/proc/net/dev")?;
    let reader = BufReader::new(file);
    let mut interfaces = HashMap::new();

    for line in reader.lines().skip(2) {
        let line = line?;
        let parts: Vec<&str> = line.trim().split(':').collect();
        if parts.len() != 2 {
            continue;
        }

        let name = parts[0].trim().to_string();
        let data: Vec<&str> = parts[1].trim().split_whitespace().collect();
        if data.len() < 16 {
            continue;
        }

        let rx_bytes = data[0].parse::<u64>().unwrap_or(0);
        let tx_bytes = data[8].parse::<u64>().unwrap_or(0);

        interfaces.insert(
            name.clone(),
            InterfaceStats {
                name,
                received_bytes: rx_bytes,
                transmitted_bytes: tx_bytes,
            },
        );
    }

    Ok(Snapshot { interfaces })
}

#[cfg(any(
    target_os = "macos",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
fn read_snapshot_unix() -> std::io::Result<Snapshot> {
    let mut interfaces = HashMap::new();

    // Create networks and ensure we get the current data
    let mut networks = Networks::new();
    // Refresh to get actual current data
    networks.refresh(true);

    for (interface_name, network) in networks.iter() {
        // Skip loopback and virtual interfaces that often have 0 bytes
        if !interface_name.starts_with("lo")
            && !interface_name.starts_with("gif")
            && !interface_name.starts_with("stf")
        {
            interfaces.insert(
                interface_name.clone(),
                InterfaceStats {
                    name: interface_name.clone(),
                    received_bytes: network.received(),
                    transmitted_bytes: network.transmitted(),
                },
            );
        }
    }

    Ok(Snapshot { interfaces })
}

#[cfg(target_os = "windows")]
fn read_snapshot_windows() -> std::io::Result<Snapshot> {
    let mut interfaces = HashMap::new();

    // Create networks and ensure we get the current data
    let mut networks = Networks::new();
    // Refresh to get actual current data
    networks.refresh(true);

    for (interface_name, network) in networks.iter() {
        // Skip loopback and virtual interfaces
        if !interface_name.contains("Loopback") && !interface_name.contains("Pseudo") {
            interfaces.insert(
                interface_name.clone(),
                InterfaceStats {
                    name: interface_name.clone(),
                    received_bytes: network.received(),
                    transmitted_bytes: network.transmitted(),
                },
            );
        }
    }

    Ok(Snapshot { interfaces })
}
