use color_eyre::Result;
use crossbeam_channel::{bounded, Receiver, Sender};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;

use opportunistic_event_ticker::{action::Action, app::App, event::EventHandler, tui::Tui};

#[tokio::main]
async fn main() -> Result<()> {
    // Install error hooks
    color_eyre::install()?;

    // Create the event channel with bounded capacity for backpressure
    let (action_tx, action_rx) = bounded(1024);

    // Initialize terminal
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let mut tui = Tui::new(terminal, action_tx.clone())?;

    // Create the application
    let mut app = App::new(action_tx.clone(), action_rx)?;

    // Set up the event handler
    let mut event_handler = EventHandler::new(action_tx.clone());

    // Enter TUI mode
    tui.enter()?;

    // Run the application
    let result = run_app(&mut app, &mut tui, event_handler).await;

    // Exit TUI mode
    tui.exit()?;

    // Return the result
    result
}

async fn run_app(
    app: &mut App,
    tui: &mut Tui<CrosstermBackend<io::Stdout>>,
    mut event_handler: EventHandler,
) -> Result<()> {
    // Start the event handler
    event_handler.start();

    // Initialize the app
    app.init()?;

    // Initial render
    tui.draw(|frame| app.render(frame))?;

    // Main event loop
    loop {
        // Process events without blocking
        let should_quit = app.process_events()?;

        if should_quit {
            break;
        }

        // Render if needed
        if app.should_render() {
            tui.draw(|frame| app.render(frame))?;
            app.rendered();
        }

        // Small yield to prevent CPU spinning
        // This is the only "tick" in the system - everything else is event-driven
        tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;
    }

    // Stop the event handler
    event_handler.stop();

    Ok(())
}
