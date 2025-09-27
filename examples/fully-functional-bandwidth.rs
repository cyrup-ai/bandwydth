use std::{
    collections::HashMap,
    env,
    process::Command,
    time::{Duration, Instant},
};

use lib_bandwydth::{
    network::{aggregator::NetAggregator, Utilization},
    network::{Direction, Sniffer},
};
use tokio::time::interval;

/// Setup Rio with IPC for privileged execution
async fn setup_rio_session() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Using Rio IPC for privileged bandwidth monitoring...");

    let current_exe = env::current_exe()?;
    let project_dir = current_exe.parent().unwrap().parent().unwrap();

    // Create command to run in Rio
    let sudo_cmd = format!(
        "cd {} && sudo {}",
        project_dir.display(),
        current_exe.display()
    );

    println!("📺 Launching Rio with command: {}", sudo_cmd);

    // Use Rio IPC to create new window with our command
    let output = Command::new("rio").args(["-e", &sudo_cmd]).spawn()?;

    println!("✅ Rio launched successfully with PID: {}", output.id());

    println!("✅ Tmux session created successfully!");
    println!("📺 Monitoring tmux session output...");
    println!("⌨️  Please enter your password in the tmux session when prompted");
    println!("🔍 I'll read and display the output here for iteration...");
    println!();

    // Keep the main thread alive and provide instructions
    println!("📋 Instructions:");
    println!("1. Rio terminal launched with sudo command");
    println!("2. Enter your password in the Rio window when prompted");
    println!("3. The bandwidth monitor will run in that window");
    println!("4. Press Ctrl+C here to stop monitoring this process");
    println!();

    // Wait for user interrupt
    tokio::signal::ctrl_c().await?;
    println!("🛑 Monitoring stopped");
    println!("📝 Note: The Rio window may still be running - check it for results");

    Ok(())
}

/// Check if we're running with sudo privileges
fn check_privileges() -> bool {
    env::var("SUDO_USER").is_ok() || unsafe { libc::geteuid() } == 0
}

/// Fully functional bandwidth monitoring example that streams real-time stats
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if we need to setup tmux for privilege escalation
    if !check_privileges() {
        println!("🔐 Bandwidth monitoring requires elevated privileges for packet capture");
        println!("🚀 Setting up Rio session for secure password input...");
        return setup_rio_session().await;
    }

    // Initialize logger to see what's happening
    simplelog::TermLogger::init(
        simplelog::LevelFilter::Debug,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .unwrap_or_else(|_| {
        eprintln!("Failed to initialize logger");
    });

    println!("🚀 Starting fully functional bandwidth monitoring with elevated privileges...");
    println!(
        "👤 Running as: {}",
        env::var("USER").unwrap_or_else(|_| "unknown".to_string())
    );
    println!("🔐 Effective UID: {}", unsafe { libc::geteuid() });

    // Test OS layer first - get open sockets
    println!("\n📊 Testing OS layer - getting open sockets...");
    match lib_bandwydth::os::get_input(None, false, None) {
        Ok(os_input) => {
            println!("✅ OS input initialized successfully!");
            println!(
                "📡 Found {} network interfaces",
                os_input.interfaces_with_frames.len()
            );

            // Test socket enumeration
            match (os_input.get_open_sockets)() {
                Ok(open_sockets) => {
                    println!(
                        "✅ Got {} open sockets from OS layer",
                        open_sockets.sockets_to_procs.len()
                    );

                    // Show first few sockets for debugging
                    let mut count = 0;
                    for (socket, proc_info) in &open_sockets.sockets_to_procs {
                        if count < 5 {
                            println!(
                                "   🔌 Socket: {}:{} ({:?}) -> Process: {} (PID: {})",
                                socket.ip,
                                socket.port,
                                socket.protocol,
                                proc_info.name,
                                proc_info.pid
                            );
                            count += 1;
                        }
                    }
                    if open_sockets.sockets_to_procs.len() > 5 {
                        println!(
                            "   ... and {} more sockets",
                            open_sockets.sockets_to_procs.len() - 5
                        );
                    }
                }
                Err(e) => {
                    println!("❌ Failed to get open sockets: {}", e);
                    return Err(e.into());
                }
            }

            // Now test packet capture
            println!("\n📦 Testing packet capture...");

            let mut aggregator = NetAggregator::<10>::new(Duration::from_secs(1), 100.0, 0.5);
            let mut utilization = Utilization::new();

            // Initialize sniffers for each interface
            let mut sniffers: Vec<Sniffer> = Vec::new();
            for (interface, frames) in os_input.interfaces_with_frames {
                println!("🌐 Setting up sniffer for interface: {}", interface.name);
                let sniffer = Sniffer::new(interface, frames, true); // show_dns = true for testing
                sniffers.push(sniffer);
            }

            if sniffers.is_empty() {
                println!("❌ No sniffers available - cannot capture packets");
                return Ok(());
            }

            println!("✅ Initialized {} packet sniffers", sniffers.len());
            println!("\n🔍 Starting real-time bandwidth monitoring...");
            println!("📊 Will show bandwidth stats every 2 seconds");
            println!("🛑 Press Ctrl+C to stop\n");

            // Main monitoring loop
            let mut packet_interval = interval(Duration::from_millis(50)); // Check for packets frequently
            let mut stats_interval = interval(Duration::from_secs(2)); // Show stats every 2 seconds
            let mut last_stats_time = Instant::now();
            let mut total_packets_captured = 0u64;
            let mut last_packet_count = 0u64;

            loop {
                tokio::select! {
                    // Capture packets
                    _ = packet_interval.tick() => {
                        for sniffer in sniffers.iter_mut() {
                            // Process packets in batches
                            for _ in 0..10 {
                                if let Some(segment) = sniffer.next_segment() {
                                    total_packets_captured += 1;

                                    println!("📦 Packet #{}: {} bytes {} on {} ({}:{} {:?})",
                                            total_packets_captured,
                                            segment.data_length,
                                            match segment.direction {
                                                Direction::Download => "↓ downloaded",
                                                Direction::Upload => "↑ uploaded",
                                            },
                                            segment.interface_name,
                                            segment.connection.remote_socket.ip,
                                            segment.connection.remote_socket.port,
                                            segment.connection.local_socket.protocol
                                    );

                                    utilization.ingest(segment);
                                } else {
                                    // No more packets from this sniffer
                                    break;
                                }
                            }
                        }
                    }

                    // Show bandwidth stats
                    _ = stats_interval.tick() => {
                        let packets_this_period = total_packets_captured - last_packet_count;
                        last_packet_count = total_packets_captured;

                        println!("\n📊 === BANDWIDTH STATS (every 2s) ===");
                        println!("⏱️  Time: {:.2}s since start", last_stats_time.elapsed().as_secs_f64());
                        println!("📦 Packets captured this period: {}", packets_this_period);
                        println!("📈 Total packets captured: {}", total_packets_captured);

                        // Get current utilization
                        let current_utilization = utilization.clone_and_reset();
                        println!("🔌 Active connections: {}", current_utilization.connections.len());

                        // Calculate total bandwidth
                        let mut total_download = 0u128;
                        let mut total_upload = 0u128;

                        for (connection, connection_info) in &current_utilization.connections {
                            total_download += connection_info.total_bytes_downloaded;
                            total_upload += connection_info.total_bytes_uploaded;

                            if connection_info.total_bytes_downloaded > 0 || connection_info.total_bytes_uploaded > 0 {
                                println!("  📡 {}:{} -> ↓{} bytes ↑{} bytes",
                                        connection.remote_socket.ip,
                                        connection.remote_socket.port,
                                        connection_info.total_bytes_downloaded,
                                        connection_info.total_bytes_uploaded);
                            }
                        }

                        // Get updated socket info
                        match (os_input.get_open_sockets)() {
                            Ok(sockets) => {
                                println!("🔌 Current open sockets: {}", sockets.sockets_to_procs.len());
                            }
                            Err(e) => {
                                println!("⚠️  Failed to refresh sockets: {}", e);
                            }
                        }

                        // Create network snapshot for bandwidth calculation
                        let mut interfaces = HashMap::new();
                        interfaces.insert(
                            "total".to_string(),
                            lib_bandwydth::network::bandwidth::NetInterfaceStats {
                                name: "total".to_string(),
                                received_bytes: total_download as u64,
                                transmitted_bytes: total_upload as u64,
                            },
                        );

                        let snapshot = lib_bandwydth::network::bandwidth::NetSnapshot { interfaces };

                        if let Some(bandwidth_stats) = aggregator.update(snapshot) {
                            println!("🚀 Bandwidth: {:.2} Mbps ({} MB/s)",
                                    bandwidth_stats.current_speed,
                                    bandwidth_stats.current_speed / 8.0);
                            println!("📊 Classification: {:?}", bandwidth_stats.bandwidth_class);
                        } else {
                            println!("⏳ Bandwidth: Calculating... (need more data)");
                        }

                        println!("💾 Total this period: ↓{:.2} KB ↑{:.2} KB",
                                total_download as f64 / 1024.0,
                                total_upload as f64 / 1024.0);

                        if packets_this_period == 0 {
                            println!("⚠️  No packets captured this period - check network activity or permissions");
                        }

                        println!("=====================================\n");

                        last_stats_time = Instant::now();
                    }

                    // Handle Ctrl+C gracefully
                    _ = tokio::signal::ctrl_c() => {
                        println!("\n🛑 Received Ctrl+C, shutting down gracefully...");
                        break;
                    }
                }
            }

            println!("📊 Final stats:");
            println!("📦 Total packets captured: {}", total_packets_captured);
            println!("✅ Bandwidth monitoring completed successfully!");
        }
        Err(e) => {
            println!("❌ Failed to initialize OS input: {}", e);
            println!("💡 Hint: You may need to run with sudo for packet capture permissions");
            return Err(e.into());
        }
    }

    Ok(())
}
