use ratatui::{
    layout::{Constraint, Direction, Rect},
    Frame,
};
use std::time::Duration;

use crate::display::{BandwidthGraphState, BandwidthGraphWidget, HeaderDetails, HelpText};

const FIRST_HEIGHT_BREAKPOINT: u16 = 30;
const FIRST_WIDTH_BREAKPOINT: u16 = 120;

fn top_app_graph_and_bottom_split(rect: Rect) -> (Rect, Rect, Rect, Rect) {
    let parts = ratatui::layout::Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Length(0), // No tables area
            Constraint::Min(10),   // Bandwidth graph (takes remaining space)
            Constraint::Length(1), // Footer
        ])
        .split(rect);
    (parts[0], parts[1], parts[2], parts[3])
}

/// Layout manager for arranging UI components.
pub struct Layout<'a> {
    /// Header details component.
    pub header: HeaderDetails<'a>,
    /// Bandwidth graph widget.
    pub bandwidth_graph: BandwidthGraphWidget,
    /// Footer help text component.
    pub footer: HelpText,
}

impl Layout<'_> {
    fn progressive_split(&self, rect: Rect, splits: Vec<Direction>) -> Vec<Rect> {
        splits
            .into_iter()
            .fold(vec![rect], |mut layout, direction| {
                if let Some(last_rect) = layout.pop() {
                    let halves = ratatui::layout::Layout::default()
                        .direction(direction)
                        .margin(0)
                        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .split(last_rect);
                    layout.append(&mut halves.to_vec());
                }
                layout
            })
    }

    fn build_two_children_layout(&self, rect: Rect) -> Vec<Rect> {
        // if there are two elements
        if rect.height < FIRST_HEIGHT_BREAKPOINT && rect.width < FIRST_WIDTH_BREAKPOINT {
            // if the space is not enough, we drop one element
            vec![rect]
        } else if rect.width < FIRST_WIDTH_BREAKPOINT {
            // if the horizontal space is not enough, we drop one element and we split horizontally
            self.progressive_split(rect, vec![Direction::Vertical])
        } else {
            // by default we display two elements splitting vertically
            self.progressive_split(rect, vec![Direction::Horizontal])
        }
    }

    fn build_three_children_layout(&self, rect: Rect) -> Vec<Rect> {
        // if there are three elements
        if rect.height < FIRST_HEIGHT_BREAKPOINT && rect.width < FIRST_WIDTH_BREAKPOINT {
            //if the space is not enough, we drop two elements
            vec![rect]
        } else if rect.height < FIRST_HEIGHT_BREAKPOINT {
            // if the vertical space is not enough, we drop one element and we split vertically
            self.progressive_split(rect, vec![Direction::Horizontal])
        } else if rect.width < FIRST_WIDTH_BREAKPOINT {
            // if the horizontal space is not enough, we drop one element and we split horizontally
            self.progressive_split(rect, vec![Direction::Vertical])
        } else {
            // default layout
            let halves = ratatui::layout::Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(rect);
            let top_quarters = ratatui::layout::Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(halves[0]);

            vec![top_quarters[0], top_quarters[1], halves[1]]
        }
    }

    /// Build layout rectangles for child components based on screen size.
    pub fn build_layout(&self, rect: Rect) -> Vec<Rect> {
        // Use responsive layout based on terminal size and breakpoints
        let num_components = 3; // Default to 3 components for demonstration

        match num_components {
            1 => vec![rect],
            2 => self.build_two_children_layout(rect),
            3 => self.build_three_children_layout(rect),
            _ => {
                // For more than 3 components, use progressive split
                if rect.height < FIRST_HEIGHT_BREAKPOINT && rect.width < FIRST_WIDTH_BREAKPOINT {
                    // Very constrained space - show only one component
                    vec![rect]
                } else if rect.width < FIRST_WIDTH_BREAKPOINT {
                    // Limited width - vertical layout
                    self.progressive_split(rect, vec![Direction::Vertical, Direction::Vertical])
                } else {
                    // Full layout - mixed directions
                    self.progressive_split(rect, vec![Direction::Horizontal, Direction::Vertical])
                }
            }
        }
    }

    /// Render the layout and all its child components to the given frame.
    ///
    /// # Arguments
    /// * `frame` - The frame to render to
    /// * `rect` - The area to render in
    /// * `table_cycle_offset` - Offset for cycling through tables with Tab key
    /// * `bandwidth_graph_state` - State for the bandwidth graph widget
    /// * `frame_duration` - Duration since last frame for animations
    pub fn render(
        &mut self,
        frame: &mut Frame,
        rect: Rect,
        bandwidth_graph_state: &mut BandwidthGraphState,
        frame_duration: Duration,
    ) {
        let (top, app, graph, bottom) = top_app_graph_and_bottom_split(rect);
        let _layout_slots = self.build_layout(app);

        // Tables are now rendered directly by the UI component

        // Render header, bandwidth graph, and footer
        self.header.render(frame, top, frame_duration);
        frame.render_stateful_widget(&self.bandwidth_graph, graph, bandwidth_graph_state);
        self.footer.render(frame, bottom);
    }
}
