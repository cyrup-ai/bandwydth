use lib_bandwydth::{network::dns::start_dns_client, start_monitor, MonitorConfig};
use std::{
    net::IpAddr,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Advanced Bandwidth Monitor Example ===");
    println!();

    // Create a DNS client for hostname resolution
    let _dns_handle = start_dns_client(1000, None, |_cache| {
        // DNS cache updates - we'll read from this later
    });

    // Submit some common IPs for resolution
    let common_ips = vec![
        "8.8.8.8",        // Google DNS
        "1.1.1.1",        // Cloudflare DNS
        "208.67.222.222", // OpenDNS
    ];

    // Submit IPs for resolution would need the executor handle
    // For now, we'll just track the IPs to resolve later
    let mut ips_to_resolve = Vec::new();
    for ip_str in &common_ips {
        if let Ok(ip) = ip_str.parse::<IpAddr>() {
            ips_to_resolve.push(ip);
        }
    }

    // Create advanced configuration
    let config = MonitorConfig {
        interface: None,             // Auto-detect primary interface
        resolve_dns: true,           // Enable DNS resolution
        dns_server: None,            // Use system DNS
        allow_system_fallback: true, // Allow fallback methods
        period_secs: 2,              // Poll every 2 seconds
    };

    // Track some statistics
    let stats_counter = Arc::new(Mutex::new(0u32));
    let max_speed = Arc::new(Mutex::new(0.0f64));

    let counter_clone = stats_counter.clone();
    let max_speed_clone = max_speed.clone();

    println!("Starting advanced bandwidth monitor...");
    println!("Configuration:");
    println!("  - Polling interval: {} seconds", config.period_secs);
    println!("  - DNS resolution: {}", config.resolve_dns);
    println!("  - System fallback: {}", config.allow_system_fallback);
    println!();

    // Start the monitor with advanced callback
    let handle = start_monitor(config, move |stats| {
        let mut counter = match counter_clone.lock() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to lock counter mutex: {e}");
                return;
            }
        };
        let mut max = match max_speed_clone.lock() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Failed to lock max speed mutex: {e}");
                return;
            }
        };

        *counter += 1;
        if stats.current_speed > *max {
            *max = stats.current_speed;
        }

        println!(
            "Measurement #{}: {:.2} MB/s | Quality: {:?} | History: {}",
            *counter,
            stats.current_speed,
            stats.bandwidth_class,
            stats.bandwidth_history.len()
        );

        // Show historical trend
        if stats.bandwidth_history.len() >= 3 {
            let recent: Vec<f64> = stats
                .bandwidth_history
                .iter()
                .rev()
                .take(3)
                .copied()
                .collect();
            let trend = if recent[0] > recent[2] {
                "↑ Increasing"
            } else if recent[0] < recent[2] {
                "↓ Decreasing"
            } else {
                "→ Stable"
            };
            println!(
                "  Trend: {} (last 3: {:.2}, {:.2}, {:.2})",
                trend, recent[2], recent[1], recent[0]
            );
        }

        // Show max speed so far
        println!("  Max speed this session: {:.2} MB/s", *max);
        println!();
    })?;

    println!("Monitor running... Will stop in 20 seconds.");
    println!("Generating some network activity to see bandwidth changes...");
    println!();

    // Let it run for 20 seconds
    thread::sleep(Duration::from_secs(20));

    println!("=== DNS Resolution Results ===");
    // TODO: Fix DNS agent implementation
    // let dns_cache = dns_agent.cache_snapshot();
    for ip_str in &common_ips {
        if let Ok(ip) = ip_str.parse::<IpAddr>() {
            // TODO: Fix DNS agent implementation
            // match dns_cache.get(&ip) {
            //     Some(hostname) => println!("{ip} -> {hostname}"),
            //     None => println!("{ip} -> (not resolved yet)"),
            // }
            println!("{ip} -> (DNS lookup disabled)");
        }
    }
    println!();

    println!("=== Final Statistics ===");
    let final_counter = match stats_counter.lock() {
        Ok(c) => *c,
        Err(e) => {
            eprintln!("Failed to lock counter mutex: {e}");
            0
        }
    };
    let final_max = match max_speed.lock() {
        Ok(m) => *m,
        Err(e) => {
            eprintln!("Failed to lock max speed mutex: {e}");
            0.0
        }
    };
    println!("Total measurements: {final_counter}");
    println!("Peak bandwidth: {final_max:.2} MB/s");
    println!();

    println!("Stopping monitor...");
    handle.stop();
    println!("Monitor stopped successfully!");

    Ok(())
}
