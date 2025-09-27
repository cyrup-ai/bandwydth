use crate::action::{Action, ActionError};
use crossbeam_channel::{bounded, select, Sender};
use crossterm::event::{self, Event as CrosstermEvent, KeyEventKind};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

/// Error type for event handling operations
#[derive(Debug)]
pub enum EventError {
    /// Failed to spawn event handler thread
    ThreadSpawn(std::io::Error),
}

impl std::fmt::Display for EventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventError::ThreadSpawn(e) => write!(f, "Failed to spawn event handler thread: {}", e),
        }
    }
}

impl std::error::Error for EventError {}

/// Handles terminal events and converts them to actions
pub struct EventHandler {
    action_tx: Sender<Action>,
    stop_flag: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl EventHandler {
    /// Create a new event handler
    #[inline]
    pub fn new(action_tx: Sender<Action>) -> Self {
        Self {
            action_tx,
            stop_flag: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    /// Start the event handler in a separate thread
    pub fn start(&mut self) -> Result<(), EventError> {
        let action_tx = self.action_tx.clone();
        let stop_flag = self.stop_flag.clone();

        let handle = thread::Builder::new()
            .name("event-handler".to_string())
            .spawn(move || {
                if let Err(e) = event_loop(action_tx, stop_flag) {
                    eprintln!("Event handler error: {}", e);
                }
            })
            .map_err(|e| {
                eprintln!("Failed to spawn event handler thread: {}", e);
                eprintln!("Event handler will not be active until manually restarted");
                EventError::ThreadSpawn(e)
            })?;

        self.handle = Some(handle);
        Ok(())
    }

    /// Stop the event handler
    pub fn stop(mut self) {
        self.stop_flag.store(true, Ordering::Release);

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    /// Check if the event handler is running
    #[inline]
    pub fn is_running(&self) -> bool {
        !self.stop_flag.load(Ordering::Acquire)
    }
}

/// Main event loop that runs in a separate thread
fn event_loop(action_tx: Sender<Action>, stop_flag: Arc<AtomicBool>) -> Result<(), ActionError> {
    // Create a channel for shutdown signaling
    let (shutdown_tx, shutdown_rx) = bounded::<()>(1);

    // Monitor stop flag in another thread
    let stop_flag_clone = stop_flag.clone();
    thread::spawn(move || {
        while !stop_flag_clone.load(Ordering::Acquire) {
            thread::sleep(Duration::from_millis(10));
        }
        let _ = shutdown_tx.send(());
    });

    loop {
        // Use select! for efficient event handling
        select! {
            recv(shutdown_rx) -> _ => {
                // Shutdown requested
                break;
            }
            default => {
                // Poll for terminal events with a short timeout
                if event::poll(Duration::from_millis(10)).unwrap_or(false) {
                    match event::read() {
                        Ok(event) => {
                            if let Some(action) = convert_event(event) {
                                // Send action, ignore if channel is full (backpressure)
                                let _ = action_tx.try_send(action);
                            }
                        }
                        Err(e) => {
                            // Send error action
                            let _ = action_tx.try_send(Action::Error(format!("Event read error: {}", e)));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Convert crossterm events to actions
#[inline(always)]
fn convert_event(event: CrosstermEvent) -> Option<Action> {
    match event {
        CrosstermEvent::Key(key_event) => {
            // Only process key press events to avoid duplicates
            if key_event.kind == KeyEventKind::Press {
                Some(Action::Key(key_event))
            } else {
                None
            }
        }
        CrosstermEvent::Mouse(mouse_event) => Some(Action::Mouse(mouse_event)),
        CrosstermEvent::Resize(width, height) => Some(Action::Resize(width, height)),
        _ => None,
    }
}

/// Event processor for batch processing
pub struct EventProcessor {
    batch: Vec<Action>,
    batch_size: usize,
}

impl EventProcessor {
    /// Create a new event processor with specified batch size
    #[inline]
    pub const fn new(batch_size: usize) -> Self {
        Self {
            batch: Vec::new(),
            batch_size,
        }
    }

    /// Add an action to the batch
    #[inline]
    pub fn add(&mut self, action: Action) -> bool {
        self.batch.push(action);
        self.batch.len() >= self.batch_size
    }

    /// Process the current batch
    #[inline]
    pub fn process<F>(&mut self, mut handler: F) -> Result<(), ActionError>
    where
        F: FnMut(Action) -> Result<(), ActionError>,
    {
        for action in self.batch.drain(..) {
            handler(action)?;
        }
        Ok(())
    }

    /// Get the current batch size
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.batch.len()
    }

    /// Check if the batch is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.batch.is_empty()
    }

    /// Clear the batch without processing
    #[inline]
    pub fn clear(&mut self) {
        self.batch.clear();
    }
}

/// Event debouncer for rate limiting
pub struct EventDebouncer {
    last_event: Option<std::time::Instant>,
    min_interval: Duration,
}

impl EventDebouncer {
    /// Create a new debouncer with minimum interval between events
    #[inline]
    pub const fn new(min_interval: Duration) -> Self {
        Self {
            last_event: None,
            min_interval,
        }
    }

    /// Check if an event should be processed
    #[inline]
    pub fn should_process(&mut self) -> bool {
        let now = std::time::Instant::now();

        match self.last_event {
            Some(last) => {
                if now.duration_since(last) >= self.min_interval {
                    self.last_event = Some(now);
                    true
                } else {
                    false
                }
            }
            None => {
                self.last_event = Some(now);
                true
            }
        }
    }

    /// Reset the debouncer
    #[inline]
    pub fn reset(&mut self) {
        self.last_event = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_processor() {
        let mut processor = EventProcessor::new(3);

        assert_eq!(processor.len(), 0);
        assert!(processor.is_empty());

        assert!(!processor.add(Action::Render));
        assert_eq!(processor.len(), 1);

        assert!(!processor.add(Action::Render));
        assert_eq!(processor.len(), 2);

        assert!(processor.add(Action::Render));
        assert_eq!(processor.len(), 3);

        let mut count = 0;
        processor
            .process(|_| {
                count += 1;
                Ok(())
            })
            .unwrap();

        assert_eq!(count, 3);
        assert!(processor.is_empty());
    }

    #[test]
    fn test_event_debouncer() {
        let mut debouncer = EventDebouncer::new(Duration::from_millis(10));

        assert!(debouncer.should_process());
        assert!(!debouncer.should_process());

        std::thread::sleep(Duration::from_millis(11));
        assert!(debouncer.should_process());

        debouncer.reset();
        assert!(debouncer.should_process());
    }
}
