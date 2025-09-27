// Debug utility to diagnose interface discovery and packet capture issues
use log::{info, warn, error, debug};
use pnet::datalink;
use std::collections::HashMap;
use std::io::Write;

use crate::os::shared::{get_datalink_channel, OsInputOutput};
use crate::network::Sniffer;

/// Comprehensive interface discovery and packet capture diagnostic
pub fn diagnose_network_interfaces(interface_name: Option<&str>) {
    info!("=== NETWORK INTERFACE DIAGNOSTIC ===");
    
    // First check BPF access on macOS
    diagnose_bpf_access();
    
    // 1. List all discovered interfaces
    let all_interfaces = datalink::interfaces();
    info!("Discovered {} total network interfaces:", all_interfaces.len());
    
    for (i, interface) in all_interfaces.iter().enumerate() {
        info!("Interface {}: {}", i + 1, interface.name);
        info!("  - Up: {}", interface.is_up());
        info!("  - Loopback: {}", interface.is_loopback());
        info!("  - Point-to-point: {}", interface.is_point_to_point());
        info!("  - Multicast: {}", interface.is_multicast());
        info!("  - IPs: {:?}", interface.ips);
        info!("  - MAC: {:?}", interface.mac);
        info!("  - Index: {:?}", interface.index);
    }
    
    // 2. Filter interfaces according to get_input logic
    let filtered_interfaces = if let Some(name) = interface_name {
        all_interfaces.into_iter()
            .filter(|iface| iface.name == name)
            .collect::<Vec<_>>()
    } else {
        all_interfaces
    }.into_iter()
        .filter(|interface| {
            let keep = if cfg!(target_os = "windows") {
                !interface.ips.is_empty()
            } else if cfg!(target_os = "macos") {
                // macOS: Be more permissive - include point-to-point interfaces (VPNs)
                // and interfaces that might not report as "up" but are functional
                !interface.ips.is_empty() && !interface.is_loopback()
            } else {
                interface.is_up() && !interface.is_loopback() && !interface.ips.is_empty()
            };
            
            if !keep {
                debug!("FILTERED OUT: {} (up={}, loopback={}, has_ips={})", 
                       interface.name, interface.is_up(), interface.is_loopback(), 
                       !interface.ips.is_empty());
            } else {
                info!("PASSED FILTER: {} (up={}, loopback={}, has_ips={})", 
                      interface.name, interface.is_up(), interface.is_loopback(), 
                      !interface.ips.is_empty());
            }
            keep
        })
        .collect::<Vec<_>>();
    
    if filtered_interfaces.is_empty() {
        error!("NO INTERFACES passed the filter! This will cause get_input() to fail.");
        return;
    }
    
    info!("After filtering: {} interfaces available", filtered_interfaces.len());
    
    // 3. Test datalink channel creation for each filtered interface
    let mut successful_channels = Vec::new();
    let mut failed_channels = HashMap::new();
    
    for interface in &filtered_interfaces {
        info!("Testing datalink channel for interface: {}", interface.name);
        
        match get_datalink_channel(interface) {
            Ok(rx) => {
                info!("  ✅ SUCCESS: Created datalink channel for {}", interface.name);
                successful_channels.push((interface.clone(), rx));
            }
            Err(err) => {
                error!("  ❌ FAILED: {} - {}", interface.name, err);
                failed_channels.insert(interface.name.clone(), format!("{}", err));
            }
        }
    }
    
    // 4. Summary
    info!("=== DATALINK CHANNEL RESULTS ===");
    info!("Successful channels: {}", successful_channels.len());
    info!("Failed channels: {}", failed_channels.len());
    
    if successful_channels.is_empty() {
        error!("ALL DATALINK CHANNELS FAILED! This is the root cause.");
        for (iface, error) in &failed_channels {
            error!("  {} failed: {}", iface, error);
        }
        return;
    }
    
    // 5. Test packet capture on successful interfaces
    info!("=== PACKET CAPTURE TEST ===");
    let mut sniffers = Vec::new();
    
    for (interface, rx) in successful_channels {
        info!("Creating sniffer for interface: {}", interface.name);
        let sniffer = Sniffer::new(interface, rx, false);
        sniffers.push(sniffer);
    }
    
    info!("Testing packet capture for 5 seconds...");
    
    // Generate test traffic to ensure we have packets to capture
    generate_test_traffic();
    
    let start_time = std::time::Instant::now();
    let mut total_segments = 0u64;
    let mut interface_segments = HashMap::new();
    
    while start_time.elapsed() < std::time::Duration::from_secs(5) {
        for (_i, sniffer) in sniffers.iter_mut().enumerate() {
            if let Some(segment) = sniffer.next_segment() {
                total_segments += 1;
                *interface_segments.entry(segment.interface_name.clone()).or_insert(0u64) += 1;
                
                if total_segments <= 10 {
                    info!("Captured segment {}: {:?} -> {:?} ({} bytes, {:?})", 
                          total_segments,
                          segment.connection.remote_socket,
                          segment.connection.local_socket,
                          segment.data_length,
                          segment.direction);
                }
            }
        }
        
        // Brief pause to avoid busy-waiting
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    
    // 6. Final results
    info!("=== PACKET CAPTURE RESULTS ===");
    info!("Total segments captured: {}", total_segments);
    
    if total_segments == 0 {
        warn!("NO PACKETS CAPTURED! Possible causes:");
        warn!("  1. No network traffic during test period");
        warn!("  2. Interface filtering too restrictive");
        warn!("  3. Packet parsing issues");
        warn!("  4. macOS SIP blocking even with sudo");
        warn!("  5. VPN or virtualization interference");
    } else {
        info!("Packet capture working! Segments per interface:");
        for (iface, count) in &interface_segments {
            info!("  {}: {} segments", iface, count);
        }
    }
    
    info!("=== DIAGNOSTIC COMPLETE ===");
}

/// Test get_input function directly
pub fn test_get_input(interface_name: Option<&str>, resolve: bool, dns_server: Option<std::net::Ipv4Addr>) -> Result<OsInputOutput, eyre::Error> {
    info!("Testing get_input() with interface: {:?}, resolve: {}, dns_server: {:?}", 
          interface_name, resolve, dns_server);
    
    match crate::os::get_input(interface_name, resolve, dns_server) {
        Ok(os_input) => {
            info!("✅ get_input() SUCCESS!");
            info!("  - {} interfaces with frame receivers", os_input.interfaces_with_frames.len());
            for (i, (iface, _)) in os_input.interfaces_with_frames.iter().enumerate() {
                info!("    {}: {}", i + 1, iface.name);
            }
            Ok(os_input)
        }
        Err(e) => {
            error!("❌ get_input() FAILED: {}", e);
            Err(e)
        }
    }
}

/// Test BPF device access on macOS to diagnose packet capture issues
#[cfg(target_os = "macos")]
pub fn diagnose_bpf_access() {
    info!("=== BPF DEVICE ACCESS DIAGNOSTIC ===");
    
    // Check effective UID
    let uid = unsafe { libc::geteuid() };
    info!("Effective UID: {} ({})", uid, if uid == 0 { "root" } else { "non-root" });
    
    if uid != 0 {
        error!("NOT RUNNING AS ROOT - this is likely the problem!");
        error!("Run with: sudo cargo run");
        return;
    }
    
    // Test individual BPF device access
    info!("Testing BPF device access...");
    let mut accessible_devices = Vec::new();
    let mut permission_denied_devices = Vec::new();
    let mut not_found_count = 0;
    
    for i in 0..20 {  // Test first 20 BPF devices
        let device_path = format!("/dev/bpf{}", i);
        let c_path = match std::ffi::CString::new(device_path.as_bytes()) {
            Ok(path) => path,
            Err(e) => {
                info!("  ❌ /dev/bpf{}: Invalid path (contains null bytes): {}", i, e);
                continue;
            }
        };
        
        let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDWR, 0) };
        
        if fd != -1 {
            info!("  ✅ /dev/bpf{}: Accessible", i);
            accessible_devices.push(i);
            unsafe { libc::close(fd); }
        } else {
            let error = std::io::Error::last_os_error();
            match error.kind() {
                std::io::ErrorKind::NotFound => {
                    not_found_count += 1;
                    if i < 5 {  // Only log first few
                        debug!("  ❓ /dev/bpf{}: Not found", i);
                    }
                }
                std::io::ErrorKind::PermissionDenied => {
                    error!("  ❌ /dev/bpf{}: Permission denied", i);
                    permission_denied_devices.push(i);
                }
                _ => {
                    error!("  ❌ /dev/bpf{}: {}", i, error);
                }
            }
        }
    }
    
    info!("BPF Device Summary:");
    info!("  - Accessible devices: {} ({:?})", accessible_devices.len(), accessible_devices);
    info!("  - Permission denied: {} ({:?})", permission_denied_devices.len(), permission_denied_devices);
    info!("  - Not found: {}", not_found_count);
    
    if accessible_devices.is_empty() {
        error!("NO BPF DEVICES ACCESSIBLE!");
        error!("This explains why packet capture fails.");
        error!("Possible causes:");
        error!("  1. System Integrity Protection (SIP) blocking access");
        error!("  2. macOS security policy preventing BPF access");
        error!("  3. Need to disable SIP: csrutil disable (requires recovery mode)");
        error!("  4. Try: sudo chmod 644 /dev/bpf*");
    }
}

#[cfg(not(target_os = "macos"))]
pub fn diagnose_bpf_access() {
    info!("BPF diagnostics only available on macOS");
}

/// Generate network traffic to test packet capture
pub fn generate_test_traffic() {
    info!("=== GENERATING TEST NETWORK TRAFFIC ===");
    
    use std::thread;
    use std::time::Duration;
    
    // Spawn background thread to generate traffic
    thread::spawn(|| {
        for i in 0..10 {
            // Generate HTTP requests
            if let Ok(mut stream) = std::net::TcpStream::connect("8.8.8.8:53") {
                let _ = stream.write_all(b"test");
                info!("Generated test traffic #{}", i + 1);
            }
            
            // Generate UDP traffic
            if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
                let _ = socket.send_to(b"test", "8.8.8.8:53");
            }
            
            thread::sleep(Duration::from_millis(100));
        }
        info!("Test traffic generation complete");
    });
    
    // Brief delay to let traffic start
    thread::sleep(Duration::from_millis(500));
}