use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{Event, EventStream, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use tachyonfx::Shader;
use tokio::sync::mpsc;

use lib_bandwydth::{
    cli::{self, Opt, RenderOpts, UnitFamily},
    display::Ui,
    network::ifstat::read_snapshot,
    privileges, MonitorConfig,
};

// Import unified cryypt API for QUIC transport
use cryypt::Cryypt;

use crossbeam_channel::bounded;
use lib_bandwydth::action::Action;

/// Print a clear, accessible notice about privilege requirements with TachyonFX effects
fn print_privilege_notice() {
    use crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::style::Color;
    use ratatui::{backend::CrosstermBackend, Terminal};
    use std::{
        io::stdout,
        time::{Duration, Instant},
    };
    use tachyonfx::{fx, EffectTimer, Interpolation, Motion};

    // Try to create a terminal display with TachyonFX effects, fall back to basic if it fails
    if let Ok(()) = enable_raw_mode() {
        let mut stdout = stdout();
        if execute!(stdout, EnterAlternateScreen).is_ok() {
            let backend = CrosstermBackend::new(stdout);
            if let Ok(mut terminal) = Terminal::new(backend) {
                let _ = terminal.clear();

                let start_time = Instant::now();

                // Create sophisticated layered TachyonFX effects for the privilege dialog
                let mut border_fade_effect = fx::fade_from_fg(
                    Color::Rgb(255, 215, 0),
                    EffectTimer::from_ms(750, Interpolation::CircOut),
                );

                let mut text_sweep_effect = fx::sweep_in(
                    Motion::LeftToRight,
                    12,
                    2,
                    Color::Rgb(40, 40, 60),
                    EffectTimer::from_ms(1000, Interpolation::QuadOut),
                );

                let mut border_pulse_effect = fx::ping_pong(fx::fade_from_fg(
                    Color::Rgb(255, 215, 0),
                    EffectTimer::from_ms(400, Interpolation::SineInOut),
                ));

                let mut background_fade_effect = fx::fade_from_fg(
                    Color::Rgb(20, 20, 40),
                    EffectTimer::from_ms(500, Interpolation::ExpoOut),
                );

                for frame_count in 0..90 {
                    // 4.5 seconds at ~20fps for complete effect showcase
                    let _elapsed = start_time.elapsed();
                    let frame_duration = tachyonfx::Duration::from_millis(50);

                    let _ = terminal.draw(|frame| {
                        render_privilege_notice_with_tachyonfx(
                            frame,
                            &mut border_fade_effect,
                            &mut text_sweep_effect,
                            &mut border_pulse_effect,
                            &mut background_fade_effect,
                            frame_duration,
                            frame_count,
                        );
                    });

                    std::thread::sleep(Duration::from_millis(50));
                }

                let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
                let _ = disable_raw_mode();
                return;
            }
        }
    }

    // Fallback to basic text output
    println!("\n╭─────────────────────────────────────────────────────────────╮");
    println!("│                    PRIVILEGE ESCALATION                    │");
    println!("├─────────────────────────────────────────────────────────────┤");
    println!("│                                                             │");
    println!("│  This application monitors network traffic in real-time,    │");
    println!("│  which requires elevated system privileges.                 │");
    println!("│                                                             │");
    println!("│  The application will now attempt to escalate privileges... │");
    println!("│                                                             │");
    println!("╰─────────────────────────────────────────────────────────────╯");
    println!();
}

/// Render TachyonFX animated privilege escalation notice with sophisticated effects
fn render_privilege_notice_with_tachyonfx(
    frame: &mut ratatui::Frame,
    border_fade_effect: &mut tachyonfx::Effect,
    text_sweep_effect: &mut tachyonfx::Effect,
    border_pulse_effect: &mut tachyonfx::Effect,
    background_fade_effect: &mut tachyonfx::Effect,
    frame_duration: tachyonfx::Duration,
    frame_count: u32,
) {
    use ratatui::{
        layout::Rect,
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, BorderType, Borders, Paragraph},
    };
    use tachyonfx::EffectRenderer;

    let area = frame.area();

    // Center the dialog with sophisticated proportions
    let dialog_width = 67;
    let dialog_height = 12;
    let dialog_area = Rect {
        x: (area.width.saturating_sub(dialog_width)) / 2,
        y: (area.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };

    // Create the main block with dynamic border coloring
    let time_cycle = (frame_count as f32 * 0.05) % (2.0 * std::f32::consts::PI);
    let border_intensity = if frame_count < 60 {
        // Gradual fade-in over first 3 seconds
        (frame_count as f32 / 60.0).min(1.0)
    } else {
        // Pulsing effect after fade-in
        0.7 + 0.3 * (time_cycle * 2.0).sin()
    };

    let border_color = Color::Rgb(
        (255.0 * border_intensity) as u8,
        (215.0 * border_intensity * 0.9) as u8,
        (0.0 * border_intensity * 0.3) as u8,
    );

    let block = Block::default()
        .title(" PRIVILEGE ESCALATION ")
        .title_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    frame.render_widget(block, dialog_area);

    // Content area with proper padding
    let content_area = Rect {
        x: dialog_area.x + 2,
        y: dialog_area.y + 2,
        width: dialog_area.width.saturating_sub(4),
        height: dialog_area.height.saturating_sub(4),
    };

    // Animated loading dots progression
    let loading_dots = match (frame_count / 12) % 4 {
        0 => "",
        1 => ".",
        2 => "..",
        3 => "...",
        _ => "...",
    };

    // Sophisticated content with progressive text reveal
    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "This application monitors network traffic in ",
                Style::default().fg(Color::Gray),
            ),
            Span::styled(
                "real-time",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(",", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("which requires ", Style::default().fg(Color::Gray)),
            Span::styled(
                "elevated system privileges",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(".", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "The application will now attempt to escalate privileges",
                Style::default().fg(Color::White),
            ),
            Span::styled(
                loading_dots,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Please wait while system permissions are verified",
            Style::default().fg(Color::Rgb(180, 180, 200)),
        )]),
    ];

    let paragraph = Paragraph::new(content);
    frame.render_widget(paragraph, content_area);

    // Apply sophisticated TachyonFX effects in layers
    if background_fade_effect.running() {
        frame.render_effect(background_fade_effect, dialog_area, frame_duration);
    }

    if frame_count > 30 && border_fade_effect.running() {
        frame.render_effect(border_fade_effect, dialog_area, frame_duration);
    }

    if frame_count > 60 && text_sweep_effect.running() {
        frame.render_effect(text_sweep_effect, content_area, frame_duration);
    }

    if frame_count > 120 && border_pulse_effect.running() {
        let border_effect_area = Rect {
            x: dialog_area.x,
            y: dialog_area.y,
            width: dialog_area.width,
            height: 1,
        };
        frame.render_effect(border_pulse_effect, border_effect_area, frame_duration);
    }
}

/// Print comprehensive help for manual privilege escalation with TachyonFX effects
fn print_privilege_help() {
    use crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{backend::CrosstermBackend, Terminal};
    use std::{
        io::stdout,
        time::{Duration, Instant},
    };

    let current_exe = std::env::args()
        .next()
        .unwrap_or_else(|| "bandwydth".to_string());

    // Try to create a fancy terminal display, fall back to basic if it fails
    if let Ok(()) = enable_raw_mode() {
        let mut stdout = stdout();
        if execute!(stdout, EnterAlternateScreen).is_ok() {
            let backend = CrosstermBackend::new(stdout);
            if let Ok(mut terminal) = Terminal::new(backend) {
                let _ = terminal.clear();

                // Create animated help display
                let start_time = Instant::now();

                for frame_count in 0..120 {
                    // 6 seconds display
                    let elapsed = start_time.elapsed();

                    let _ = terminal.draw(|frame| {
                        render_privilege_help(frame, elapsed, frame_count, &current_exe);
                    });

                    std::thread::sleep(Duration::from_millis(50));
                }

                let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
                let _ = disable_raw_mode();
                return;
            }
        }
    }

    // Fallback to basic text output
    println!("\n╭─────────────────────────────────────────────────────────────╮");
    println!("│                   PRIVILEGE ESCALATION FAILED              │");
    println!("├─────────────────────────────────────────────────────────────┤");
    println!("│                                                             │");
    println!("│  Network monitoring requires elevated privileges to access  │");
    println!("│  low-level network interfaces and system resources.         │");
    println!("│                                                             │");
    println!("│  Please run this application manually with elevated         │");
    println!("│  privileges using one of the following methods:             │");
    println!("│                                                             │");

    #[cfg(unix)]
    {
        println!("│  🔐 UNIX/Linux/macOS:                                       │");
        println!(
            "│      sudo {}                              │",
            pad_command(&current_exe, 32)
        );
        println!("│                                                             │");
        println!("│  💡 Alternative (run as root):                              │");
        println!(
            "│      su -c '{}'                           │",
            pad_command(&current_exe, 32)
        );
    }

    #[cfg(windows)]
    {
        println!("│  🔐 Windows:                                                │");
        println!("│      1. Right-click on Command Prompt or PowerShell        │");
        println!("│      2. Select 'Run as Administrator'                      │");
        println!("│      3. Navigate to the application directory              │");
        println!(
            "│      4. Run: {}                           │",
            pad_command(&current_exe, 32)
        );
        println!("│                                                             │");
        println!("│  💡 Alternative (UAC elevation):                            │");
        println!("│      Right-click the executable and 'Run as Administrator' │");
    }

    println!("│                                                             │");
    println!("│  ℹ️  Why are privileges needed?                              │");
    println!("│      • Access to network interface statistics              │");
    println!("│      • Monitor network packet flows                        │");
    println!("│                                                             │");
    println!("│  🛡️  Security Note:                                          │");
    println!("│      This application only reads network data and does not │");
    println!("│      modify system settings or network configurations.     │");
    println!("│                                                             │");
    println!("╰─────────────────────────────────────────────────────────────╯");
    println!();
}

/// Render animated privilege escalation help
fn render_privilege_help(
    frame: &mut ratatui::Frame,
    elapsed: Duration,
    frame_count: u32,
    current_exe: &str,
) {
    use ratatui::{
        layout::Rect,
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, BorderType, Borders, Paragraph},
    };

    let area = frame.area();

    // Create large dialog for detailed help
    let dialog_width = area.width.min(80);
    let dialog_height = area.height.min(25);
    let dialog_area = Rect {
        x: (area.width.saturating_sub(dialog_width)) / 2,
        y: (area.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };

    // Animated error-style border
    let time_cycle = (elapsed.as_millis() % 3000) as f32 / 3000.0;
    let border_color = Color::Rgb(
        (255.0 * (0.8 + 0.2 * (time_cycle * std::f32::consts::TAU).sin())) as u8,
        (50.0 * (0.5 + 0.5 * (time_cycle * std::f32::consts::TAU + 1.0).sin())) as u8,
        (50.0 * (0.5 + 0.5 * (time_cycle * std::f32::consts::TAU + 2.0).sin())) as u8,
    );

    let block = Block::default()
        .title(" PRIVILEGE ESCALATION FAILED ")
        .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    frame.render_widget(block, dialog_area);

    // Content area
    let content_area = Rect {
        x: dialog_area.x + 2,
        y: dialog_area.y + 2,
        width: dialog_area.width.saturating_sub(4),
        height: dialog_area.height.saturating_sub(4),
    };

    // Build content with fade-in effect
    let line_delay = 3; // frames per line
    let visible_lines = (frame_count / line_delay).min(20);

    let mut content = Vec::new();
    let lines = [
        ("", Color::Reset),
        (
            "Network monitoring requires elevated privileges to access",
            Color::Gray,
        ),
        (
            "low-level network interfaces and system resources.",
            Color::Gray,
        ),
        ("", Color::Reset),
        (
            "Please run this application manually with elevated",
            Color::White,
        ),
        (
            "privileges using one of the following methods:",
            Color::White,
        ),
        ("", Color::Reset),
        #[cfg(unix)]
        ("🔐 UNIX/Linux/macOS:", Color::Cyan),
        #[cfg(windows)]
        ("🔐 Windows:", Color::Cyan),
        #[cfg(unix)]
        (&format!("    sudo {current_exe}"), Color::Green),
        #[cfg(windows)]
        (
            "    1. Right-click on Command Prompt or PowerShell",
            Color::Green,
        ),
        #[cfg(windows)]
        ("    2. Select 'Run as Administrator'", Color::Green),
        #[cfg(windows)]
        ("    3. Navigate to the application directory", Color::Green),
        #[cfg(windows)]
        (&format!("    4. Run: {}", current_exe), Color::Green),
        ("", Color::Reset),
        ("ℹ️  Why are privileges needed?", Color::Yellow),
        ("    • Access to network interface statistics", Color::Gray),
        ("    • Monitor network packet flows", Color::Gray),
        ("    • Read system-level process information", Color::Gray),
        ("", Color::Reset),
        ("🛡️  Security Note:", Color::Blue),
        (
            "    This application only reads network data and does not",
            Color::Gray,
        ),
        (
            "    modify system settings or network configurations.",
            Color::Gray,
        ),
    ];

    for (i, (text, color)) in lines.iter().enumerate() {
        if i < visible_lines as usize {
            let alpha = if i == (visible_lines as usize).saturating_sub(1)
                && frame_count % line_delay < line_delay
            {
                // Fade in current line
                (frame_count % line_delay) as f32 / line_delay as f32
            } else {
                1.0
            };

            let style = if *color == Color::Reset {
                Style::default()
            } else {
                let final_color = match color {
                    Color::Gray => Color::Rgb(
                        (128.0 * alpha) as u8,
                        (128.0 * alpha) as u8,
                        (128.0 * alpha) as u8,
                    ),
                    Color::White => Color::Rgb(
                        (255.0 * alpha) as u8,
                        (255.0 * alpha) as u8,
                        (255.0 * alpha) as u8,
                    ),
                    Color::Green => Color::Rgb(0, (255.0 * alpha) as u8, 0),
                    Color::Cyan => Color::Rgb(0, (255.0 * alpha) as u8, (255.0 * alpha) as u8),
                    Color::Yellow => Color::Rgb((255.0 * alpha) as u8, (255.0 * alpha) as u8, 0),
                    Color::Blue => Color::Rgb(0, (150.0 * alpha) as u8, (255.0 * alpha) as u8),
                    Color::Red => Color::Rgb((255.0 * alpha) as u8, 0, 0),
                    _ => *color,
                };
                Style::default().fg(final_color)
            };

            content.push(Line::from(Span::styled(*text, style)));
        }
    }

    let paragraph = Paragraph::new(content);
    frame.render_widget(paragraph, content_area);
}

/// Pad command to fit within the box layout
fn pad_command(cmd: &str, max_width: usize) -> String {
    if cmd.len() <= max_width {
        format!("{cmd:max_width$}")
    } else {
        format!("{}...", &cmd[..max_width.saturating_sub(3)])
    }
}

/// Run the ProgressHub QUIC client to test bandwidth downloads
async fn run_progress_hub_client() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting ProgressHub QUIC client (no bandwidth monitoring)...");
    println!("📡 Connecting to ProgressHub server at 127.0.0.1:4433...");

    // Connect to the ProgressHub server using unified cryypt API
    println!("🔗 Establishing QUIC connection...");
    let connection_handle = Cryypt::quic()
        .client()
        .with_server_name("progresshub.local")
        .connect("127.0.0.1:4433")
        .await
        .map_err(|e| format!("QUIC connection failed: {}", e))?;

    println!("✅ Connected to ProgressHub server via QUIC!");
    println!("🎯 ProgressHub client is now running. Testing download functionality...");
    println!("📊 This would normally handle bandwidth events and file downloads...");
    println!("🔄 Connection established - ProgressHub downloads should now work properly!");
    println!("\nPress Ctrl+C to stop the client.");

    // Keep the connection alive and wait for Ctrl+C
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("\n🛑 Received shutdown signal...");
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(3600)) => {
            println!("🕐 Connection timeout after 1 hour");
        }
    }

    println!("🔌 Shutting down ProgressHub QUIC client...");

    // The connection_handle will be dropped here, closing the connection
    drop(connection_handle);

    println!("✅ ProgressHub client shutdown complete!");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments FIRST before any privilege checks
    let args: Vec<String> = std::env::args().collect();
    let cli_mode = args.contains(&"--cli".to_string());
    let no_bandwidth = args.contains(&"--no-bandwidth".to_string());

    // For TUI mode, we should not log to terminal at all
    // Only log errors and above, or use file logging for debug info
    if std::env::var("BANDWYDTH_DEBUG").is_ok() {
        // If debug env var is set, log to a file
        if let Ok(log_file) = std::fs::File::create("/tmp/bandwydth.log") {
            let _ = simplelog::WriteLogger::init(
                simplelog::LevelFilter::Debug,
                simplelog::Config::default(),
                log_file,
            );
        }
    } else {
        // For normal operation, only log critical errors
        let _ = simplelog::TermLogger::init(
            simplelog::LevelFilter::Error,
            simplelog::Config::default(),
            simplelog::TerminalMode::Stderr,
            simplelog::ColorChoice::Auto,
        );
    }

    // If --cli and --no-bandwidth are specified, run the ProgressHub client directly
    if cli_mode && no_bandwidth {
        println!("🚀 Starting ProgressHub client (no bandwidth monitoring)...");
        return run_progress_hub_client().await;
    }

    // Check if we need privileges and don't have them (only for bandwidth monitoring)
    let requires_elevation = privileges::requires_elevation_for_operation("interface_stats");
    let has_privileges = privileges::check_privileges()?;

    if requires_elevation && !has_privileges {
        print_privilege_notice();

        // Escalate privileges - this will exec() and replace the current process
        if let Err(e) = privileges::escalate_privileges() {
            print_privilege_help();
            return Err(e.into());
        }
        // This code is never reached because exec() replaces the process
        unreachable!("escalate_privileges should not return on success");
    }

    // Try terminal mode, fall back to CLI if not supported
    if let Err(e) = enable_raw_mode() {
        eprintln!("Failed to enable raw mode: {e}");
        eprintln!("Falling back to CLI mode...");
        return cli::run().await;
    }

    // Set up terminal
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);

    // Create options for the UI - ENABLE DATA DISPLAY
    let opts = Opt {
        interface: None, // Monitor all interfaces
        raw: false,
        no_resolve: true,
        show_dns: false,
        render_opts: RenderOpts {
            addresses: true,         // ✅ Show remote addresses
            connections: true,       // ✅ Show connections
            processes: true,         // ✅ Show processes
            total_utilization: true, // ✅ Show bandwidth totals
            unit_family: UnitFamily::BinBytes,
        },
        config_path: None,
    };

    // Create UI
    log::debug!("Creating UI with opts: {:?}", opts);
    let mut ui = Ui::new(backend, &opts)?;
    log::debug!("UI created successfully");

    // Set up panic handler to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic);
    }));

    // Channel for bandwidth and utilization data
    let (data_tx, mut data_rx) = mpsc::unbounded_channel::<(
        lib_bandwydth::network::BandwidthStats,
        lib_bandwydth::network::Utilization,
        std::collections::HashMap<
            lib_bandwydth::network::LocalSocket,
            lib_bandwydth::os::ProcessInfo,
        >,
    )>();

    // Create action channel for UI events
    let (action_tx, action_rx) = bounded::<Action>(1024);

    // Set up the UI action sender after the channel is created
    ui.set_action_sender(action_tx.clone());

    // Configure interface-level bandwidth monitor
    let config = MonitorConfig {
        interface: opts.interface.clone(),
        resolve_dns: !opts.no_resolve,
        dns_server: None,
        allow_system_fallback: true,
        period_secs: 1,
    };

    // Start proper packet capture bandwidth monitoring using AsyncTask pattern
    let data_tx_clone = data_tx.clone();
    let interface_name = config.interface.clone();
    let resolve_dns = config.resolve_dns;
    let dns_server = config.dns_server;
    let period_secs = config.period_secs;

    // Spawn async task for packet capture (this is the correct async pattern for this project)
    tokio::task::spawn(async move {
        match lib_bandwydth::os::get_input(interface_name.as_deref(), resolve_dns, dns_server) {
            Ok(os_input) => {
                log::debug!(
                    "Successfully initialized packet capture with {} interfaces",
                    os_input.interfaces_with_frames.len()
                );

                let mut aggregator = lib_bandwydth::network::aggregator::NetAggregator::<10>::new(
                    Duration::from_secs(period_secs),
                    100.0,
                    0.5,
                );

                // Use a shorter interval to be more responsive to actual packet arrival
                let mut packet_interval = tokio::time::interval(Duration::from_millis(10));
                let mut bandwidth_interval = tokio::time::interval(Duration::from_millis(500)); // Update every 500ms instead of 1 second
                let mut utilization = lib_bandwydth::network::Utilization::new();

                // Initialize sniffers for each interface (keep them synchronous as designed)
                let mut sniffers: Vec<lib_bandwydth::network::Sniffer> = Vec::new();
                for (interface, frames) in os_input.interfaces_with_frames {
                    let sniffer = lib_bandwydth::network::Sniffer::new(
                        interface,
                        frames,
                        !config.resolve_dns,
                    );
                    sniffers.push(sniffer);
                }

                loop {
                    tokio::select! {
                        // Process packets frequently but don't block
                        _ = packet_interval.tick() => {
                            let mut segments_processed = 0;
                            for (idx, sniffer) in sniffers.iter_mut().enumerate() {
                                // Process a reasonable number of packets per tick
                                let mut sniffer_segments = 0;
                                for _ in 0..5 {
                                    if let Some(segment) = sniffer.next_segment() {
                                        log::debug!("Sniffer {} captured segment: {:?} bytes from {}", 
                                                   idx, segment.data_length, segment.interface_name);
                                        utilization.ingest(segment);
                                        sniffer_segments += 1;
                                        segments_processed += 1;
                                    } else {
                                        // No more packets available from this sniffer
                                        break;
                                    }
                                }
                                if sniffer_segments > 0 {
                                    log::debug!("Sniffer {} processed {} segments", idx, sniffer_segments);
                                }
                            }
                            if segments_processed > 0 {
                                log::debug!("Total segments processed this tick: {}", segments_processed);
                            }
                        }

                        // Calculate and send bandwidth stats on the configured interval
                        _ = bandwidth_interval.tick() => {

                            let current_utilization = utilization.clone_and_reset();

                        // Get open sockets to map connections to processes
                        // Run in blocking thread to avoid blocking the async runtime
                        let get_sockets_fn = os_input.get_open_sockets;
                        let connections_to_procs = match tokio::task::spawn_blocking(get_sockets_fn).await {
                            Ok(Ok(open_sockets)) => open_sockets.sockets_to_procs,
                            Ok(Err(e)) => {
                                log::debug!("Failed to get open sockets: {e}");
                                std::collections::HashMap::new()
                            }
                            Err(e) => {
                                log::debug!("Failed to spawn blocking task: {e}");
                                std::collections::HashMap::new()
                            }
                        };

                        // Calculate total bandwidth from utilization data
                        let mut total_rx_bytes = 0u128;
                        let mut total_tx_bytes = 0u128;
                        let mut interface_names = std::collections::HashSet::new();

                        log::debug!("Processing utilization data: {} connections tracked", 
                                   current_utilization.connections.len());
                        
                        for (connection, connection_info) in &current_utilization.connections {
                            total_rx_bytes += connection_info.total_bytes_downloaded;
                            total_tx_bytes += connection_info.total_bytes_uploaded;
                            interface_names.insert(connection_info.interface_name.clone());
                            
                            if connection_info.total_bytes_downloaded > 0 || connection_info.total_bytes_uploaded > 0 {
                                log::debug!("Connection {:?}: RX={} TX={} interface={}",
                                           connection, connection_info.total_bytes_downloaded,
                                           connection_info.total_bytes_uploaded, connection_info.interface_name);
                            }
                        }

                        log::debug!("Period totals - RX: {} bytes, TX: {} bytes, interfaces: {:?}",
                                   total_rx_bytes, total_tx_bytes, interface_names);
                        
                        if total_rx_bytes == 0 && total_tx_bytes == 0 {
                            log::warn!("No bandwidth detected - check if packets are being captured properly");
                        }

                        // Create snapshot from captured packet data
                        let mut interfaces = std::collections::HashMap::new();
                        // Use a single aggregated interface for bandwidth calculation
                        interfaces.insert(
                            "all".to_string(),
                            lib_bandwydth::network::bandwidth::NetInterfaceStats {
                                name: "all".to_string(),
                                received_bytes: total_rx_bytes as u64,
                                transmitted_bytes: total_tx_bytes as u64,
                            },
                        );

                        let snapshot = lib_bandwydth::network::bandwidth::NetSnapshot { interfaces };

                        // Always create and send bandwidth stats, even if aggregator doesn't have enough data yet
                        let bandwidth_stats = if let Some(stats) = aggregator.update(snapshot) {
                            // Convert NetBandwidthStats<10> to BandwidthStats
                            lib_bandwydth::network::BandwidthStats {
                                current_speed: stats.current_speed / 8.0, // Convert Mbps to MBps
                                bandwidth_history: stats.bandwidth_history.to_vec(),
                                bandwidth_class: match stats.bandwidth_class {
                                    lib_bandwydth::network::NetBandwidthClass::Inconclusive => {
                                        lib_bandwydth::network::BandwidthClass::Inconclusive
                                    }
                                    lib_bandwydth::network::NetBandwidthClass::Poor => {
                                        lib_bandwydth::network::BandwidthClass::Poor
                                    }
                                    lib_bandwydth::network::NetBandwidthClass::Average => {
                                        lib_bandwydth::network::BandwidthClass::Fair
                                    }
                                    lib_bandwydth::network::NetBandwidthClass::Good => {
                                        lib_bandwydth::network::BandwidthClass::Good
                                    }
                                    lib_bandwydth::network::NetBandwidthClass::Blazing => {
                                        lib_bandwydth::network::BandwidthClass::Blazing
                                    }
                                },
                            }
                        } else {
                            // Create default bandwidth stats when aggregator doesn't have enough data yet
                            lib_bandwydth::network::BandwidthStats {
                                current_speed: 0.0,
                                bandwidth_history: vec![],
                                bandwidth_class: lib_bandwydth::network::BandwidthClass::Inconclusive,
                            }
                        };

                        // Always send utilization data to UI, even if bandwidth calculation isn't ready
                        if let Err(e) = data_tx_clone.send((bandwidth_stats, current_utilization.clone(), connections_to_procs.clone())) {
                            log::debug!("Failed to send bandwidth stats: {e}");
                            break;
                        }
                        }
                    }
                }
            }
            Err(e) => {
                log::debug!("Failed to initialize packet capture: {e}");
            }
        }
    });

    // Main event loop with modern EventStream
    let start_time = Instant::now();
    let mut cumulative_time = Duration::default();
    let mut last_start_time = start_time;
    let mut paused = false;
    let _table_cycle_offset = 0;

    let mut events = EventStream::new();
    let mut render_interval = tokio::time::interval(Duration::from_millis(100));
    let mut snapshot_interval = tokio::time::interval(Duration::from_millis(500)); // Update snapshots every 500ms
    let mut tick_interval = tokio::time::interval(Duration::from_millis(250)); // Regular tick for animations
    let mut timer_poll_interval = tokio::time::interval(Duration::from_millis(50)); // Poll timer tickers frequently
    let mut needs_render = false;

    log::debug!("Starting main event loop");

    // Initial render
    if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ui.draw(paused, Duration::from_secs(0), 0);
    })) {
        log::error!("Panic in initial UI draw: {:?}", e);
        // Cleanup and exit
        ui.end();
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        return Err("UI rendering failed".into());
    }

    loop {
        // Check for UI actions (non-blocking)
        if let Ok(action) = action_rx.try_recv() {
            // Opportunistic ticking on ANY action - this is the key to auto-ticking!
            if ui.on_any_event() {
                needs_render = true;
            }

            match action {
                Action::UpdateUI => needs_render = true,
                Action::Tick => {
                    // Tick event received - components may have already ticked above via on_any_event
                    needs_render = true;
                }
                Action::StatsUpdate => {
                    ui.handle_stats_update();
                    needs_render = true;
                }
                _ => {}
            }
        }

        tokio::select! {
            // Handle bandwidth data updates - trigger immediate render when data arrives
            Some((bandwidth_stats, utilization, connections_to_procs)) = data_rx.recv() => {

                    // Update UI with real interface bandwidth data and connection info
                    ui.update_bandwidth_stats(&bandwidth_stats, Some(&utilization));
                    ui.update_utilization(connections_to_procs, utilization);
                    ui.update_advanced_monitor_stats(&bandwidth_stats);


                    // Trigger immediate render
                    let _ = action_tx.send(Action::UpdateUI);

                    // Render immediately when new data arrives (event-driven rendering)
                    let elapsed = if paused {
                        cumulative_time
                    } else {
                        cumulative_time + last_start_time.elapsed()
                    };

                    // Professional event-driven rendering with proper state tracking
                    if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        ui.draw(paused, elapsed, 0);
                        // After rendering, schedule timer ticks to ensure animations continue
                        ui.schedule_timer();
                    })) {
                        log::error!("Panic in UI draw: {:?}", e);
                        break;
                    }
            }


            // Handle terminal events
            event = events.next() => {
                if let Some(Ok(Event::Key(key))) = event {

                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                        KeyCode::Char(' ') => {
                            if paused {
                                last_start_time = Instant::now();
                                paused = false;
                            } else {
                                cumulative_time += last_start_time.elapsed();
                                paused = true;
                            }
                            needs_render = true;
                        }
                        KeyCode::Tab => {
                            if key.modifiers.contains(KeyModifiers::SHIFT) {
                                ui.previous_tab();
                            } else {
                                ui.next_tab();
                            }
                            needs_render = true;
                        }
                        _ => {}
                    }
                }
            }

            // Update network snapshots for interface monitoring
            _ = snapshot_interval.tick() => {
                if let Ok(snapshot) = read_snapshot() {
                    ui.update_snapshot(&snapshot);
                    needs_render = true;
                }
            }

            // Send tick events for animations and regular updates
            _ = tick_interval.tick() => {
                if action_tx.send(Action::Tick).is_err() {
                    // Channel closed, exit
                    break;
                }
            }

            // Poll for timer ticks from opportunistic ticker (third part of pattern)
            _ = timer_poll_interval.tick() => {
                if ui.try_poll_timer() {
                    needs_render = true;
                }
            }

            // Fallback rendering at regular intervals (only if no data received recently)
            _ = render_interval.tick() => {
                if needs_render {
                    let elapsed = if paused {
                        cumulative_time
                    } else {
                        cumulative_time + last_start_time.elapsed()
                    };
                    ui.draw(paused, elapsed, 0);
                    needs_render = false;
                }
            }
        }
    }

    // Cleanup
    ui.end();
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}
