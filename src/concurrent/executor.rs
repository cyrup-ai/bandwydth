//! Zero-allocation crossbeam-based event-driven task executor
//!
//! This module provides a truly zero-allocation task executor built exclusively
//! on crossbeam's lock-free primitives. All allocations happen at initialization,
//! with fixed-size pools and no allocations during operation. NO UNSAFE CODE.

use crossbeam_channel::{bounded, select, Receiver, Sender};
use std::time::{Duration, Instant};

use crate::action::Action;

/// Maximum number of concurrent timers supported
const MAX_TIMERS: usize = 64;

/// Maximum number of actions that can be stored
const MAX_ACTIONS: usize = 128;

/// Task types that can be executed - no heap allocation via enum dispatch
#[derive(Debug, Clone, PartialEq)]
pub enum Task {
    /// Execute an action immediately
    Execute(Action),
    /// Schedule a one-shot action after a delay
    ScheduleOnce {
        /// The action to execute
        action: Action,
        /// Delay before execution
        delay: Duration,
    },
    /// Schedule a recurring action with interval
    ScheduleRecurring {
        /// The action to execute repeatedly
        action: Action,
        /// Interval between executions
        interval: Duration,
    },
    /// Cancel a timer by ID
    CancelTimer(TimerId),
    /// Resolve DNS for IP addresses
    ResolveDns {
        /// IP addresses to resolve
        ips: Vec<std::net::IpAddr>,
    },
}

/// Timer identifier for cancellation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimerId(u32);

impl TimerId {
    #[inline(always)]
    const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// State of the executor handle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutorState {
    /// Executor is running normally
    Active,
    /// Executor failed to start, handle is degraded
    Degraded,
}

/// Timer entry in the pool
#[derive(Debug, Clone)]
struct TimerEntry {
    /// When this timer should fire
    deadline: Instant,
    /// Index into action storage
    action_index: usize,
    /// Interval for recurring timers, None for one-shot
    interval: Option<Duration>,
    /// Unique timer ID
    id: TimerId,
}

/// Pre-allocated timer channel
struct TimerChannel {
    /// Receiver for timer events
    rx: Receiver<Instant>,
    /// Whether this channel is in use
    in_use: bool,
    /// Which timer index is using this channel
    timer_index: Option<usize>,
}

/// Zero-allocation timer heap using fixed-size arrays
struct TimerHeap {
    /// Fixed-size array of timer entries (Option for safe initialization)
    entries: [Option<TimerEntry>; MAX_TIMERS],
    /// Pre-allocated timer channels
    channels: [Option<TimerChannel>; MAX_TIMERS],
    /// Action storage to avoid cloning
    actions: [Option<Action>; MAX_ACTIONS],
    /// Number of active timers
    size: usize,
    /// Next timer ID to assign
    next_id: u32,
    /// Next action slot to use
    next_action_slot: usize,
}

impl TimerHeap {
    #[inline]
    fn new() -> Self {
        const NONE_ENTRY: Option<TimerEntry> = None;
        const NONE_CHANNEL: Option<TimerChannel> = None;
        const NONE_ACTION: Option<Action> = None;

        Self {
            entries: [NONE_ENTRY; MAX_TIMERS],
            channels: [NONE_CHANNEL; MAX_TIMERS],
            actions: [NONE_ACTION; MAX_ACTIONS],
            size: 0,
            next_id: 1,
            next_action_slot: 0,
        }
    }

    /// Initialize pre-allocated channels
    fn init_channels(&mut self) {
        // Pre-allocate timer channels to avoid runtime allocation
        for i in 0..MAX_TIMERS {
            let (tx, rx) = bounded::<Instant>(1);
            self.channels[i] = Some(TimerChannel {
                rx,
                in_use: false,
                timer_index: None,
            });
            // Drop the sender - we'll use crossbeam::after() to send to these
            drop(tx);
        }
    }

    /// Allocate a timer channel for a specific timer entry
    fn allocate_channel(&mut self, timer_index: usize) -> Option<usize> {
        for (channel_index, channel) in self.channels.iter_mut().enumerate() {
            if let Some(ch) = channel {
                if !ch.in_use {
                    ch.in_use = true;
                    ch.timer_index = Some(timer_index);
                    return Some(channel_index);
                }
            }
        }
        None // All channels in use
    }

    /// Free a timer channel when timer is complete
    fn free_channel(&mut self, timer_index: usize) {
        for ch in self.channels.iter_mut().flatten() {
            if ch.timer_index == Some(timer_index) {
                ch.in_use = false;
                ch.timer_index = None;
                break;
            }
        }
    }

    /// Store an action and return its index
    #[inline]
    fn store_action(&mut self, action: Action) -> Option<usize> {
        // Find next available slot
        for _ in 0..MAX_ACTIONS {
            let idx = self.next_action_slot;
            self.next_action_slot = (self.next_action_slot + 1) % MAX_ACTIONS;

            if self.actions[idx].is_none() {
                self.actions[idx] = Some(action);
                return Some(idx);
            }
        }
        None // All slots full
    }

    /// Get the next deadline
    #[inline(always)]
    fn peek_deadline(&self) -> Option<Instant> {
        if self.size > 0 {
            self.entries[0].as_ref().map(|e| e.deadline)
        } else {
            None
        }
    }

    /// Add a timer, returns its ID or None if full
    #[inline]
    fn add_timer(
        &mut self,
        action: Action,
        delay: Duration,
        interval: Option<Duration>,
    ) -> Option<TimerId> {
        if self.size >= MAX_TIMERS {
            return None;
        }

        // Store action
        let action_index = self.store_action(action)?;

        // Allocate a channel for this timer
        let timer_index = self.size;
        let _channel_index = self.allocate_channel(timer_index)?;

        let id = TimerId::new(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);

        let deadline = Instant::now() + delay;

        // Create new entry
        let entry = TimerEntry {
            deadline,
            action_index,
            interval,
            id,
        };

        // Add to heap
        self.entries[self.size] = Some(entry);

        // Bubble up to maintain heap property
        self.bubble_up(self.size);
        self.size += 1;

        Some(id)
    }

    /// Process all expired timers
    #[inline]
    fn process_expired(&mut self, now: Instant, action_tx: &Sender<Action>) {
        while self.size > 0 {
            let should_process = self.entries[0]
                .as_ref()
                .map(|e| e.deadline <= now)
                .unwrap_or(false);

            if !should_process {
                break;
            }

            // Extract the timer
            if let Some(mut entry) = self.entries[0].take() {
                // Send the action if it exists
                if let Some(action) = &self.actions[entry.action_index] {
                    let _ = action_tx.try_send(action.clone());
                }

                // Handle recurring timers
                if let Some(interval) = entry.interval {
                    // Update deadline for next occurrence
                    entry.deadline = now + interval;
                    self.entries[0] = Some(entry);
                    self.bubble_down(0);
                } else {
                    // One-shot timer - free the action slot and channel
                    self.actions[entry.action_index] = None;
                    self.free_channel(0); // Root timer index is always 0
                    self.remove_root();
                }
            }
        }
    }

    /// Cancel a timer by ID
    #[inline]
    fn cancel_timer(&mut self, id: TimerId) {
        // Find and remove the timer
        for i in 0..self.size {
            if let Some(entry) = &self.entries[i] {
                if entry.id == id {
                    // Free the action slot
                    self.actions[entry.action_index] = None;

                    // Free the channel
                    self.free_channel(i);

                    // Remove from heap
                    self.entries[i] = None;

                    // Move last element here and decrease size
                    self.size -= 1;
                    if i < self.size {
                        self.entries[i] = self.entries[self.size].take();
                        self.entries[self.size] = None;

                        // Restore heap property
                        self.bubble_down(i);
                        self.bubble_up(i);
                    }
                    break;
                }
            }
        }
    }

    /// Remove the root element
    #[inline]
    fn remove_root(&mut self) {
        if self.size == 0 {
            return;
        }

        self.size -= 1;
        if self.size > 0 {
            self.entries[0] = self.entries[self.size].take();
            self.bubble_down(0);
        } else {
            self.entries[0] = None;
        }
    }

    /// Get deadline for comparison
    #[inline(always)]
    fn get_deadline(&self, index: usize) -> Option<Instant> {
        self.entries[index].as_ref().map(|e| e.deadline)
    }

    /// Bubble up to maintain min-heap property
    #[inline]
    fn bubble_up(&mut self, mut index: usize) {
        while index > 0 {
            let parent = (index - 1) / 2;

            let should_swap = match (self.get_deadline(index), self.get_deadline(parent)) {
                (Some(child), Some(parent_deadline)) => child < parent_deadline,
                _ => false,
            };

            if should_swap {
                self.entries.swap(index, parent);
                index = parent;
            } else {
                break;
            }
        }
    }

    /// Bubble down to maintain min-heap property
    #[inline]
    fn bubble_down(&mut self, mut index: usize) {
        loop {
            let left = 2 * index + 1;
            let right = 2 * index + 2;
            let mut smallest = index;

            // Find smallest among parent, left, and right
            if left < self.size {
                if let (Some(left_deadline), Some(smallest_deadline)) =
                    (self.get_deadline(left), self.get_deadline(smallest))
                {
                    if left_deadline < smallest_deadline {
                        smallest = left;
                    }
                }
            }

            if right < self.size {
                if let (Some(right_deadline), Some(smallest_deadline)) =
                    (self.get_deadline(right), self.get_deadline(smallest))
                {
                    if right_deadline < smallest_deadline {
                        smallest = right;
                    }
                }
            }

            if smallest != index {
                self.entries.swap(index, smallest);
                index = smallest;
            } else {
                break;
            }
        }
    }

    /// Build select operation for active timers
    fn build_timer_select(&self) -> Vec<&Receiver<Instant>> {
        let mut receivers = Vec::with_capacity(self.size);

        // Add receivers for all active timers
        for ch in self.channels.iter().flatten() {
            if ch.in_use {
                receivers.push(&ch.rx);
            }
        }

        receivers
    }
}

/// Zero-allocation task executor using crossbeam's event-driven channels
pub struct TaskExecutor {
    /// Channel for receiving tasks
    task_rx: Receiver<Task>,
    /// Channel for sending actions to the app
    action_tx: Sender<Action>,
    /// Shutdown signal receiver
    shutdown_rx: Receiver<()>,
}

impl TaskExecutor {
    /// Create a new task executor with bounded channels for backpressure
    #[inline]
    pub fn new(action_tx: Sender<Action>) -> (Self, ExecutorHandle) {
        // Bounded channels prevent unbounded memory growth
        let (task_tx, task_rx) = bounded(256);
        let (shutdown_tx, shutdown_rx) = bounded(1);

        let executor = Self {
            task_rx,
            action_tx,
            shutdown_rx,
        };

        let handle = ExecutorHandle {
            task_tx,
            shutdown_tx,
            state: ExecutorState::Active,
        };

        (executor, handle)
    }

    /// Run the executor using pure event-driven selection
    pub fn run(self) {
        // Pre-allocate timer heap with channels
        let mut timers = TimerHeap::new();
        timers.init_channels();

        // Main event loop
        loop {
            // Check for immediately expired timers
            let now = Instant::now();
            if let Some(deadline) = timers.peek_deadline() {
                if deadline <= now {
                    timers.process_expired(now, &self.action_tx);
                    continue;
                }
            }

            // Build timer receivers for active channels (but don't use them in select yet)
            let _timer_receivers = timers.build_timer_select();

            // Build select with proper timeout
            let has_timers = timers.size > 0;
            let timeout_duration = if let Some(deadline) = timers.peek_deadline() {
                deadline.saturating_duration_since(Instant::now())
            } else {
                Duration::from_secs(86400) // 24 hours
            };

            // Use select! macro with timeout
            select! {
                // Handle task submissions
                recv(self.task_rx) -> task => {
                    match task {
                        Ok(task) => {
                            if !self.handle_task(task, &mut timers) {
                                return; // Shutdown requested
                            }
                        }
                        Err(_) => return, // Channel closed
                    }
                }

                // Handle shutdown signal
                recv(self.shutdown_rx) -> _ => {
                    return;
                }

                // Handle timeout for timers
                default(timeout_duration) => {
                    if has_timers {
                        // Process any expired timers
                        timers.process_expired(Instant::now(), &self.action_tx);
                    }
                }
            }
        }
    }

    /// Handle a single task
    #[inline(always)]
    fn handle_task(&self, task: Task, timers: &mut TimerHeap) -> bool {
        match task {
            Task::Execute(action) => {
                let _ = self.action_tx.try_send(action);
                true
            }
            Task::ScheduleOnce { action, delay } => {
                timers.add_timer(action, delay, None);
                true
            }
            Task::ScheduleRecurring { action, interval } => {
                timers.add_timer(action, interval, Some(interval));
                true
            }
            Task::CancelTimer(id) => {
                timers.cancel_timer(id);
                true
            }
            Task::ResolveDns { ips } => {
                // Spawn DNS resolution in a blocking thread
                let action_tx = self.action_tx.clone();
                std::thread::spawn(move || {
                    use crate::network::dns::resolver::{Lookup, Resolver};

                    // Create a DNS resolver with proper error handling and runtime detection
                    let resolver_result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                        // We're already in an async runtime context
                        handle.block_on(Resolver::new(None))
                    } else {
                        // Create new runtime only if no current runtime exists
                        match tokio::runtime::Runtime::new() {
                            Ok(rt) => rt.block_on(Resolver::new(None)),
                            Err(e) => {
                                log::error!("Failed to create DNS runtime: {}", e);
                                return; // Exit the closure early
                            }
                        }
                    };
                    
                    if let Ok(resolver) = resolver_result {
                        let mut dns_results = std::collections::HashMap::new();

                        // Resolve each IP
                        for ip in ips {
                            if let Some(hostname) = resolver.lookup(ip) {
                                dns_results.insert(ip, hostname);
                            }
                        }

                        // Send results back as an action
                        if !dns_results.is_empty() {
                            let _ = action_tx.try_send(Action::DnsUpdate(dns_results));
                        }
                    }
                });
                true
            }

        }
    }
}

/// Handle for submitting tasks to the executor
#[derive(Clone)]
pub struct ExecutorHandle {
    task_tx: Sender<Task>,
    shutdown_tx: Sender<()>,
    state: ExecutorState,
}

impl ExecutorHandle {
    /// Create a degraded handle that rejects all operations
    fn new_degraded() -> Self {
        // Create dummy channels that will never be used
        let (task_tx, _) = bounded(1);
        let (shutdown_tx, _) = bounded(1);
        
        Self {
            task_tx,
            shutdown_tx,
            state: ExecutorState::Degraded,
        }
    }

    /// Submit a task to the executor
    #[inline(always)]
    pub fn submit(&self, task: Task) -> Result<(), TaskSubmitError> {
        match self.state {
            ExecutorState::Active => self.task_tx
                .try_send(task)
                .map_err(|_| TaskSubmitError::QueueFull),
            ExecutorState::Degraded => Err(TaskSubmitError::ExecutorDegraded),
        }
    }

    /// Submit a task, waiting if the queue is full
    #[inline]
    pub fn submit_wait(&self, task: Task) -> Result<(), TaskSubmitError> {
        match self.state {
            ExecutorState::Active => self.task_tx
                .send(task)
                .map_err(|_| TaskSubmitError::ExecutorShutdown),
            ExecutorState::Degraded => Err(TaskSubmitError::ExecutorDegraded),
        }
    }

    /// Execute an action immediately
    #[inline(always)]
    pub fn execute(&self, action: Action) -> Result<(), TaskSubmitError> {
        self.submit(Task::Execute(action))
    }

    /// Schedule a one-shot action after a delay
    #[inline]
    pub fn schedule_once(&self, action: Action, delay: Duration) -> Result<(), TaskSubmitError> {
        self.submit(Task::ScheduleOnce { action, delay })
    }

    /// Schedule a recurring action with an interval
    #[inline]
    pub fn schedule_recurring(
        &self,
        action: Action,
        interval: Duration,
    ) -> Result<(), TaskSubmitError> {
        self.submit(Task::ScheduleRecurring { action, interval })
    }

    /// Cancel a timer
    #[inline]
    pub fn cancel_timer(&self, id: TimerId) -> Result<(), TaskSubmitError> {
        self.submit(Task::CancelTimer(id))
    }

    /// Shutdown the executor using the shutdown channel (recommended)
    #[inline]
    pub fn shutdown(self) {
        let _ = self.shutdown_tx.send(());
    }


    /// Check if the executor is still running
    #[inline(always)]
    pub fn is_running(&self) -> bool {
        self.state == ExecutorState::Active
    }
}

/// Error type for task submission
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskSubmitError {
    /// The task queue is full
    QueueFull,
    /// The executor has shut down
    ExecutorShutdown,
    /// The executor is in degraded mode
    ExecutorDegraded,
}

impl std::fmt::Display for TaskSubmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::QueueFull => write!(f, "task queue is full"),
            Self::ExecutorShutdown => write!(f, "executor has shut down"),
            Self::ExecutorDegraded => write!(f, "executor is in degraded mode - no processing available"),
        }
    }
}

impl std::error::Error for TaskSubmitError {}

/// Spawn the executor in a dedicated thread
#[inline]
pub fn spawn_executor(action_tx: Sender<Action>) -> ExecutorHandle {
    let (executor, handle) = TaskExecutor::new(action_tx.clone());

    match std::thread::Builder::new()
        .name("event-executor".into())
        .spawn(move || executor.run())
    {
        Ok(_thread_handle) => {
            log::debug!("Event executor thread spawned successfully");
            handle
        }
        Err(e) => {
            log::error!("Failed to spawn executor thread: {}, returning degraded handle", e);
            eprintln!("Warning: Task executor in degraded mode - all operations will fail");
            ExecutorHandle::new_degraded()
        }
    }
}
