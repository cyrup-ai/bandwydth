use std::{collections::HashMap, net::IpAddr, time::Duration};

use chrono::prelude::*;
use ratatui::{backend::Backend, Frame, Terminal};

use crate::{
    cli::{Opt, RenderOpts},
    concurrent::ticker_state::OpportunisticDebouncedTicker,
    display::{
        components::{
            AdvancedMonitorComponent, BandwidthGraphState, BandwidthGraphWidget,
            BandwidthStatsComponent, BandwidthUnitFamily, HeaderDetails, HelpText,
            InterfaceDebugComponent, Layout, Table,
        },
        ui_components_tickable::UiComponentsTickable,
        DisplayMode, UIState,
    },
    network::{
        display_connection_string, display_ip_or_host, BandwidthStats, Snapshot, Utilization,
    },
};

/// Terminal UI for bandwidth monitoring.
pub struct Ui<B>
where
    B: Backend + 'static,
{
    terminal: Terminal<B>,
    state: UIState,
    ip_to_host: HashMap<IpAddr, String>,
    opts: RenderOpts,
    // Opportunistic ticker for auto-ticking components with safe ownership
    opportunistic_ticker: OpportunisticDebouncedTicker<UiComponentsTickable>,
}

impl<B> Ui<B>
where
    B: Backend + 'static,
{
    /// Create a new UI instance with the given terminal backend and options.
    pub fn new(terminal_backend: B, opts: &Opt) -> std::io::Result<Self> {
        let mut terminal = Terminal::new(terminal_backend)
            .map_err(|e| std::io::Error::other(format!("Terminal error: {e:?}")))?;
        terminal
            .clear()
            .map_err(|e| std::io::Error::other(format!("Terminal error: {e:?}")))?;
        terminal
            .hide_cursor()
            .map_err(|e| std::io::Error::other(format!("Terminal error: {e:?}")))?;
        let state = {
            let mut state = UIState::default();
            state.interface_name.clone_from(&opts.interface);
            state.unit_family = BandwidthUnitFamily::new(opts.render_opts.unit_family);
            state.cumulative_mode = opts.render_opts.total_utilization;
            state.show_dns = opts.show_dns;
            state
        };
        
        // Create UI components tickable wrapper (safe ownership)
        let ui_components = UiComponentsTickable::new();
        
        // Create the opportunistic ticker with owned components (no unsafe code)
        let opportunistic_ticker = OpportunisticDebouncedTicker::new(ui_components);

        Ok(Ui {
            terminal,
            state,
            ip_to_host: Default::default(),
            opts: opts.render_opts,
            opportunistic_ticker,
        })
    }

    /// Set action sender for component event streams
    pub fn set_action_sender(&mut self, sender: crossbeam_channel::Sender<crate::action::Action>) {
        self.opportunistic_ticker.target_mut().bandwidth_stats_component_mut().set_action_sender(sender);
    }

    /// Handle stats update event - update component display state
    pub fn handle_stats_update(&mut self) {
        self.opportunistic_ticker.target_mut().bandwidth_stats_component_mut().update_display_state();
    }

    /// Output bandwidth data as text (for non-TUI mode).
    pub fn output_text(&mut self, write_to_stdout: &mut (dyn FnMut(&str) + Send)) {
        let state = &self.state;
        let ip_to_host = &self.ip_to_host;
        let local_time: DateTime<Local> = Local::now();
        let timestamp = local_time.timestamp();
        let mut no_traffic = true;

        let output_process_data = |write_to_stdout: &mut (dyn FnMut(&str) + Send),
                                   no_traffic: &mut bool| {
            for (proc_info, process_network_data) in &state.processes {
                write_to_stdout(&format!(
                    "process: <{timestamp}> \"{}\" up/down Bps: {}/{} connections: {}",
                    proc_info.name,
                    process_network_data.total_bytes_uploaded,
                    process_network_data.total_bytes_downloaded,
                    process_network_data.connection_count
                ));
                *no_traffic = false;
            }
        };

        let output_connections_data =
            |write_to_stdout: &mut (dyn FnMut(&str) + Send), no_traffic: &mut bool| {
                for (connection, connection_network_data) in &state.connections {
                    write_to_stdout(&format!(
                        "connection: <{timestamp}> {} up/down Bps: {}/{} process: \"{}\"",
                        display_connection_string(
                            &connection.remote_socket,
                            connection.local_socket.protocol,
                            &connection.local_socket,
                            ip_to_host,
                        ),
                        connection_network_data.total_bytes_uploaded,
                        connection_network_data.total_bytes_downloaded,
                        connection_network_data.process_name
                    ));
                    *no_traffic = false;
                }
            };

        let output_adressess_data = |write_to_stdout: &mut (dyn FnMut(&str) + Send),
                                     no_traffic: &mut bool| {
            for (remote_address, remote_address_network_data) in &state.remote_addresses {
                write_to_stdout(&format!(
                    "remote_address: <{timestamp}> {} up/down Bps: {}/{} connections: {}",
                    display_ip_or_host(remote_address, &ip_to_host.get(remote_address).cloned()),
                    remote_address_network_data.total_bytes_uploaded,
                    remote_address_network_data.total_bytes_downloaded,
                    remote_address_network_data.connection_count
                ));
                *no_traffic = false;
            }
        };

        // header
        write_to_stdout("Refreshing:");

        // body1
        if self.opts.processes {
            output_process_data(write_to_stdout, &mut no_traffic);
        }
        if self.opts.connections {
            output_connections_data(write_to_stdout, &mut no_traffic);
        }
        if self.opts.addresses {
            output_adressess_data(write_to_stdout, &mut no_traffic);
        }
        if !(self.opts.processes || self.opts.connections || self.opts.addresses) {
            output_process_data(write_to_stdout, &mut no_traffic);
            output_connections_data(write_to_stdout, &mut no_traffic);
            output_adressess_data(write_to_stdout, &mut no_traffic);
        }

        // body2: In case no traffic is detected
        if no_traffic {
            write_to_stdout("<NO TRAFFIC>");
        }

        // footer
        write_to_stdout("");
    }

    /// Wrapper for event-driven rendering
    pub fn render(&mut self, frame: &mut Frame) {
        // Extract data before rendering to avoid borrowing issues
        let current_mode = self.state.display_mode;
        let show_dns = self.state.show_dns;

        // Create tables with current data
        let connections_table = if self.opts.connections {
            Some(Table::create_connections_table(
                &self.state,
                &self.ip_to_host,
            ))
        } else {
            None
        };
        let processes_table = if self.opts.processes {
            Some(Table::create_processes_table(&self.state))
        } else {
            None
        };
        let addresses_table = if self.opts.addresses {
            Some(Table::create_remote_addresses_table(
                &self.state,
                &self.ip_to_host,
            ))
        } else {
            None
        };

        let area = frame.area();

        // Create layout for tab bar and content
        let parts = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .margin(0)
            .constraints([
                ratatui::layout::Constraint::Length(1), // Tab bar
                ratatui::layout::Constraint::Min(0),    // Content area
            ])
            .split(area);

        // Render tab bar
        Self::render_tab_bar_static(frame, parts[0], current_mode);

        // Render content based on current display mode
        match current_mode {
            DisplayMode::Overview => {
                // Split the content area for header, tables, graph, and footer
                let content_layout = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints([
                        ratatui::layout::Constraint::Length(1),      // Header
                        ratatui::layout::Constraint::Percentage(40), // Tables
                        ratatui::layout::Constraint::Percentage(40), // Graph
                        ratatui::layout::Constraint::Length(1),      // Footer
                    ])
                    .split(parts[1]);

                // Render header
                let mut header = HeaderDetails::new(&self.state, Duration::from_secs(0), false);
                header.render(frame, content_layout[0], Duration::from_millis(100));

                // Create layout instance for responsive rendering
                let layout = Layout {
                    header: HeaderDetails::new(&self.state, Duration::from_secs(0), false),
                    bandwidth_graph: BandwidthGraphWidget::default(),
                    footer: HelpText {
                        paused: false,
                        show_dns,
                    },
                };

                // Use responsive layout for table area
                let table_rects = layout.build_layout(content_layout[1]);

                // Render tables using responsive layout rectangles
                if !table_rects.is_empty() {
                    Self::render_tables_responsive(
                        frame,
                        &table_rects,
                        connections_table.as_ref(),
                        processes_table.as_ref(),
                        addresses_table.as_ref(),
                        self.opts,
                    );
                }

                // Render bandwidth graph using safe component access
                self.opportunistic_ticker.target_mut().bandwidth_graph_state_mut().render(frame, content_layout[2]);

                // Render footer
                let footer = HelpText {
                    paused: false,
                    show_dns,
                };
                footer.render(frame, content_layout[3]);
            }
            DisplayMode::InterfaceDebug => {
                self.opportunistic_ticker.target_mut().interface_debug_component_mut().render(frame, parts[1]);
            }
            DisplayMode::AdvancedMonitor => {
                self.opportunistic_ticker.target_mut().advanced_monitor_component_mut().render(frame, parts[1]);
            }
            DisplayMode::BandwidthStats => {
                self.opportunistic_ticker.target_mut().bandwidth_stats_component_mut().render(frame, parts[1]);
            }
        }
    }

    /// Draw the UI to the terminal.
    pub fn draw(&mut self, _paused: bool, _elapsed_time: Duration, _data_updates: usize) {
        log::debug!("UI::draw() called with mode: {:?}", self.state.display_mode);

        // The key insight: we need to replicate what render() does, but inside terminal.draw()
        // The render() method already handles all modes correctly with mutable access
        // The issue is that terminal.draw() creates an immutable borrow of frame
        // but our components need mutable access to self

        // Solution: Extract all the data we need before terminal.draw(), then render everything inside
        let current_mode = self.state.display_mode;
        let show_dns = self.state.show_dns;

        // Pre-create tables to avoid borrowing self inside the closure
        let connections_table = if self.opts.connections {
            Some(Table::create_connections_table(
                &self.state,
                &self.ip_to_host,
            ))
        } else {
            None
        };
        let processes_table = if self.opts.processes {
            Some(Table::create_processes_table(&self.state))
        } else {
            None
        };
        let addresses_table = if self.opts.addresses {
            Some(Table::create_remote_addresses_table(
                &self.state,
                &self.ip_to_host,
            ))
        } else {
            None
        };

        let result = self.terminal.draw(|frame| {
            let area = frame.area();

            // Create layout for tab bar and content
            let parts = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .margin(0)
                .constraints([
                    ratatui::layout::Constraint::Length(1), // Tab bar
                    ratatui::layout::Constraint::Min(0),    // Content area
                ])
                .split(area);

            // Render tab bar
            Self::render_tab_bar_static(frame, parts[0], current_mode);

            // Render content based on current display mode
            match current_mode {
                DisplayMode::Overview => {
                    // Split the content area for header, tables, graph, and footer
                    let content_layout = ratatui::layout::Layout::default()
                        .direction(ratatui::layout::Direction::Vertical)
                        .constraints([
                            ratatui::layout::Constraint::Length(1),      // Header
                            ratatui::layout::Constraint::Percentage(40), // Tables
                            ratatui::layout::Constraint::Percentage(40), // Graph
                            ratatui::layout::Constraint::Length(1),      // Footer
                        ])
                        .split(parts[1]);

                    // Render header
                    let mut header = HeaderDetails::new(&self.state, Duration::from_secs(0), false);
                    header.render(frame, content_layout[0], Duration::from_millis(100));

                    // Create layout instance for responsive rendering
                    let layout = Layout {
                        header: HeaderDetails::new(&self.state, Duration::from_secs(0), false),
                        bandwidth_graph: BandwidthGraphWidget::default(),
                        footer: HelpText {
                            paused: false,
                            show_dns,
                        },
                    };

                    // Use responsive layout for table area
                    let table_rects = layout.build_layout(content_layout[1]);

                    // Render tables using responsive layout rectangles
                    if !table_rects.is_empty() {
                        Self::render_tables_responsive(
                            frame,
                            &table_rects,
                            connections_table.as_ref(),
                            processes_table.as_ref(),
                            addresses_table.as_ref(),
                            self.opts,
                        );
                    }

                    // Render bandwidth graph using safe component access through ticker
                    self.opportunistic_ticker.target_mut().bandwidth_graph_state_mut().render(frame, content_layout[2]);

                    // Render footer
                    let footer = HelpText {
                        paused: false,
                        show_dns,
                    };
                    footer.render(frame, content_layout[3]);
                }
                DisplayMode::InterfaceDebug => {
                    self.opportunistic_ticker.target_mut().interface_debug_component_mut().render(frame, parts[1]);
                }
                DisplayMode::AdvancedMonitor => {
                    self.opportunistic_ticker.target_mut().advanced_monitor_component_mut().render(frame, parts[1]);
                }
                DisplayMode::BandwidthStats => {
                    self.opportunistic_ticker.target_mut().bandwidth_stats_component_mut().render(frame, parts[1]);
                }
            }
        });

        result.ok();
    }

    /// Render compact layout for small terminal sizes
    #[allow(dead_code)]
    fn render_compact_layout(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        layout: &mut Layout,
        _table_cycle_offset: usize,
        frame_duration: Duration,
    ) {
        // Use original layout splitting for small terminals
        let parts = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .margin(0)
            .constraints([
                ratatui::layout::Constraint::Length(1),
                ratatui::layout::Constraint::Min(0),
                ratatui::layout::Constraint::Length(1),
            ])
            .split(area);

        // Create tables with current data
        let connections_table = if self.opts.connections {
            Some(Table::create_connections_table(
                &self.state,
                &self.ip_to_host,
            ))
        } else {
            None
        };
        let processes_table = if self.opts.processes {
            Some(Table::create_processes_table(&self.state))
        } else {
            None
        };
        let addresses_table = if self.opts.addresses {
            Some(Table::create_remote_addresses_table(
                &self.state,
                &self.ip_to_host,
            ))
        } else {
            None
        };

        // Render tables directly
        Self::render_tables_static(
            frame,
            parts[1],
            connections_table.as_ref(),
            processes_table.as_ref(),
            addresses_table.as_ref(),
            self.opts,
        );

        // Render header and footer
        layout.header.render(frame, parts[0], frame_duration);
        layout.footer.render(frame, parts[2]);
    }

    fn render_tables_responsive(
        frame: &mut ratatui::Frame,
        table_rects: &[ratatui::layout::Rect],
        connections_table: Option<&Table>,
        processes_table: Option<&Table>,
        addresses_table: Option<&Table>,
        opts: RenderOpts,
    ) {
        let mut rect_idx = 0;

        // Render enabled tables in order, using available rectangles
        if let Some(table) = connections_table {
            if opts.connections && rect_idx < table_rects.len() {
                table.render(frame, table_rects[rect_idx]);
                rect_idx += 1;
            }
        }

        if let Some(table) = processes_table {
            if opts.processes && rect_idx < table_rects.len() {
                table.render(frame, table_rects[rect_idx]);
                rect_idx += 1;
            }
        }

        if let Some(table) = addresses_table {
            if opts.addresses && rect_idx < table_rects.len() {
                table.render(frame, table_rects[rect_idx]);
            }
        }
    }

    fn render_tables_static(
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        connections_table: Option<&Table>,
        processes_table: Option<&Table>,
        addresses_table: Option<&Table>,
        opts: RenderOpts,
    ) {
        // Create layout for tables
        let chunks = if opts.connections && opts.processes && opts.addresses {
            ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Percentage(33),
                    ratatui::layout::Constraint::Percentage(33),
                    ratatui::layout::Constraint::Percentage(34),
                ])
                .split(area)
        } else if (opts.addresses || opts.processes) && opts.connections
            || (opts.processes && opts.addresses)
        {
            ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Percentage(50),
                    ratatui::layout::Constraint::Percentage(50),
                ])
                .split(area)
        } else {
            vec![area].into()
        };

        let mut chunk_idx = 0;

        // Render connections table if enabled
        if let Some(table) = connections_table {
            if chunk_idx < chunks.len() {
                table.render(frame, chunks[chunk_idx]);
                chunk_idx += 1;
            }
        }

        // Render processes table if enabled
        if let Some(table) = processes_table {
            if chunk_idx < chunks.len() {
                table.render(frame, chunks[chunk_idx]);
                chunk_idx += 1;
            }
        }

        // Render addresses table if enabled
        if let Some(table) = addresses_table {
            if chunk_idx < chunks.len() {
                table.render(frame, chunks[chunk_idx]);
            }
        }
    }

    /// Get the number of tables to display based on render options.
    pub fn get_table_count(&self) -> usize {
        let mut count = 0;
        if self.opts.connections {
            count += 1;
        }
        if self.opts.processes {
            count += 1;
        }
        if self.opts.addresses {
            count += 1;
        }
        count
    }

    /// Update bandwidth classification and speed.
    pub fn update_bandwidth_stats(
        &mut self,
        stats: &crate::network::BandwidthStats,
        utilization: Option<&Utilization>,
    ) {
        self.state.update_bandwidth_stats(stats);
        // Update the bandwidth graph with new data and utilization
        self.opportunistic_ticker.target_mut().bandwidth_graph_state_mut().update_bandwidth(stats, utilization);
        // Update the bandwidth stats component with new stats
        self.opportunistic_ticker.target_mut().bandwidth_stats_component_mut().update_stats(stats, &self.state);
    }

    /// Update the UI state with network utilization data.
    pub fn update_utilization(
        &mut self,
        connections_to_procs: HashMap<crate::network::LocalSocket, crate::os::ProcessInfo>,
        network_utilization: Utilization,
    ) {
        self.state.update(connections_to_procs, network_utilization);
    }

    /// Update the bandwidth graph with new data (event-driven)
    pub fn update_bandwidth_graph(
        &mut self,
        stats: &BandwidthStats,
        utilization: Option<&Utilization>,
    ) {
        self.opportunistic_ticker.target_mut().bandwidth_graph_state_mut().update_bandwidth(stats, utilization);
    }

    /// Switch to the next display mode (tab navigation)
    pub fn next_tab(&mut self) {
        self.state.display_mode = self.state.display_mode.next();
        // Sync the display mode to the components
        self.opportunistic_ticker.target_mut().set_display_mode(self.state.display_mode);
    }

    /// Switch to the previous display mode (shift+tab navigation)
    pub fn previous_tab(&mut self) {
        self.state.display_mode = self.state.display_mode.previous();
        // Sync the display mode to the components
        self.opportunistic_ticker.target_mut().set_display_mode(self.state.display_mode);
    }

    /// Update with network snapshot for components
    pub fn update_snapshot(&mut self, snapshot: &Snapshot) {
        self.opportunistic_ticker.target_mut().interface_debug_component_mut().update(snapshot.clone());
        self.opportunistic_ticker.target_mut().advanced_monitor_component_mut().update_snapshot(snapshot);
    }

    /// Update advanced monitor component with bandwidth stats
    pub fn update_advanced_monitor_stats(&mut self, stats: &BandwidthStats) {
        self.opportunistic_ticker.target_mut().advanced_monitor_component_mut().update_stats(stats);
        self.opportunistic_ticker.target_mut().bandwidth_stats_component_mut().update_stats(stats, &self.state);
    }

    /// Update the DNS cache with resolved hostnames.
    pub fn update_dns_cache(&mut self, dns_table: HashMap<IpAddr, String>) {
        // Merge new DNS results into existing cache
        for (ip, hostname) in dns_table {
            self.ip_to_host.insert(ip, hostname);
        }
    }

    /// Get the current display mode
    pub fn current_display_mode(&self) -> DisplayMode {
        self.state.display_mode
    }

    /// Tick all components that implement the Tickable trait
    pub fn tick(&mut self) {
        use crate::concurrent::tickable::Tickable;

        // Sync display mode and tick through opportunistic ticker
        self.opportunistic_ticker.target_mut().set_display_mode(self.state.display_mode);
        self.opportunistic_ticker.target_mut().tick();
    }

    /// Render the tab bar (static version to avoid borrowing issues)
    fn render_tab_bar_static(
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        current_mode: DisplayMode,
    ) {
        use ratatui::{
            style::{Color, Modifier, Style},
            text::{Line, Span},
            widgets::{Block, Borders, Paragraph},
        };

        let mut spans = Vec::new();

        for (i, mode) in DisplayMode::all().iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" | "));
            }

            let style = if *mode == current_mode {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            spans.push(Span::styled(mode.display_name(), style));
        }

        let tabs = Paragraph::new(Line::from(spans))
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::NONE));

        frame.render_widget(tabs, area);
    }

    /// Get a reference to the bandwidth graph state component
    pub fn bandwidth_graph_state(&self) -> &BandwidthGraphState {
        self.opportunistic_ticker.target().bandwidth_graph_state()
    }

    /// Get a reference to the bandwidth stats component
    pub fn bandwidth_stats_component(&self) -> &BandwidthStatsComponent {
        self.opportunistic_ticker.target().bandwidth_stats_component()
    }

    /// Get a reference to the advanced monitor component
    pub fn advanced_monitor_component(&self) -> &AdvancedMonitorComponent {
        self.opportunistic_ticker.target().advanced_monitor_component()
    }

    /// Get a reference to the interface debug component
    pub fn interface_debug_component(&self) -> &InterfaceDebugComponent {
        self.opportunistic_ticker.target().interface_debug_component()
    }

    /// Opportunistic ticking on any event - returns true if a tick occurred
    pub fn on_any_event(&mut self) -> bool {
        self.opportunistic_ticker.on_any_event()
    }

    /// Schedule a timer tick to ensure animations continue
    pub fn schedule_timer(&mut self) {
        self.opportunistic_ticker.schedule_timer();
    }

    /// Try to poll for timer ticks (non-blocking)
    pub fn try_poll_timer(&mut self) -> bool {
        self.opportunistic_ticker.try_poll_timer()
    }

    /// Clean up and restore terminal state.
    pub fn end(&mut self) {
        let _ = self.terminal.show_cursor(); // Ignore error on cleanup
    }
}
