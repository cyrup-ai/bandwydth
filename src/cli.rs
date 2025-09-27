use std::path::PathBuf;

/// Command-line options for the bandwidth monitor.
#[derive(Debug, Clone, Default)]
pub struct Opt {
    /// Network interface to monitor (None = all interfaces).
    pub interface: Option<String>,
    /// Use raw mode output instead of TUI.
    pub raw: bool,
    /// Disable DNS hostname resolution.
    pub no_resolve: bool,
    /// Show DNS queries in the output.
    pub show_dns: bool,
    /// Rendering options for the display.
    pub render_opts: RenderOpts,
    /// Path to configuration file to load settings from.
    pub config_path: Option<PathBuf>,
}

/// Display rendering options.
#[derive(Debug, Clone, Copy, Default)]
pub struct RenderOpts {
    /// Show remote addresses table.
    pub addresses: bool,
    /// Show connections table.
    pub connections: bool,
    /// Show processes table.
    pub processes: bool,
    /// Show total utilization.
    pub total_utilization: bool,
    /// Unit family for bandwidth display.
    pub unit_family: UnitFamily,
}

/// Unit family for displaying bandwidth measurements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnitFamily {
    /// Binary bytes (1024-based, e.g., KiB, MiB, GiB).
    #[default]
    BinBytes,
    /// Binary bits (1024-based, e.g., Kibit, Mibit, Gibit).
    BinBits,
    /// SI bytes (1000-based, e.g., KB, MB, GB).
    SiBytes,
    /// SI bits (1000-based, e.g., Kbit, Mbit, Gbit).
    SiBits,
}

impl Opt {
    /// Load options from a config file if it exists.
    pub fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        use std::fs;

        let contents = fs::read_to_string(path)?;

        // Parse simple key=value format
        let mut opt = Self::default();
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "interface" => opt.interface = Some(value.to_string()),
                    "raw" => opt.raw = value.parse().unwrap_or(false),
                    "no_resolve" => opt.no_resolve = value.parse().unwrap_or(false),
                    "show_dns" => opt.show_dns = value.parse().unwrap_or(false),
                    "unit_family" => {
                        opt.render_opts.unit_family = match value {
                            "BinBits" => UnitFamily::BinBits,
                            "SiBytes" => UnitFamily::SiBytes,
                            "SiBits" => UnitFamily::SiBits,
                            _ => UnitFamily::BinBytes,
                        };
                    }
                    _ => {} // Ignore unknown keys
                }
            }
        }

        Ok(opt)
    }
}

/// Run the CLI-based bandwidth monitor (non-TUI mode).
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    use crate::network::NetBandwidthStats;
    use crate::{start_monitor, MonitorConfig};

    println!("Running in CLI mode...");
    println!("Press Ctrl+C to stop monitoring.\n");

    // Create default config with 1-second intervals
    let config = MonitorConfig {
        period_secs: 1,
        ..Default::default()
    };

    // Start monitoring
    let handle = start_monitor(config, |stats: NetBandwidthStats<10>| {
        println!(
            "Current Speed: {:.2} MB/s | Class: {:?} | History: {} samples",
            stats.current_speed,
            stats.bandwidth_class,
            stats.bandwidth_history.len()
        );
    })?;

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    println!("\nStopping monitor...");
    handle.stop();

    Ok(())
}
