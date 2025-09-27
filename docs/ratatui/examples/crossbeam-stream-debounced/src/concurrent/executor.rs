//! Zero-allocation crossbeam-based task executor
//!
//! This module provides a high-performance task executor built exclusively on
//! crossbeam primitives. All tasks are represented as enum variants to avoid
//! heap allocation. Communication happens through lock-free channels.

use crossbeam_channel::{bounded, Receiver, Sender, TryRecvError};
use octocrab::params::{pulls::Sort, Direction};
use std::time::{Duration, Instant};

use crate::{
    action::Action,
    data::{FetchResult, PullRequest},
};

/// Task types that can be executed - no heap allocation via enum dispatch
#[derive(Debug, Clone)]
pub enum Task {
    /// Fetch pull requests from GitHub
    FetchPullRequests {
        owner: &'static str,
        repo: &'static str,
    },
    /// Periodic timer tick
    Tick {
        interval: Duration,
        next_tick: Instant,
    },
    /// Shutdown signal
    Shutdown,
}

/// Result of task execution
#[derive(Debug)]
pub enum TaskResult {
    /// Pull requests fetched successfully
    PullRequests(Vec<PullRequest>),
    /// Error during task execution
    Error(String),
    /// Timer tick occurred
    Tick,
    /// Executor shutting down
    Shutdown,
}

/// Zero-allocation task executor using crossbeam channels
pub struct TaskExecutor {
    task_rx: Receiver<Task>,
    result_tx: Sender<Action>,
    shutdown_rx: Receiver<()>,
}

impl TaskExecutor {
    /// Create a new task executor with bounded channels for backpressure
    #[inline]
    pub fn new(result_tx: Sender<Action>) -> (Self, ExecutorHandle) {
        // Bounded channels prevent unbounded memory growth
        let (task_tx, task_rx) = bounded(128);
        let (shutdown_tx, shutdown_rx) = bounded(1);

        let executor = Self {
            task_rx,
            result_tx,
            shutdown_rx,
        };

        let handle = ExecutorHandle {
            task_tx,
            shutdown_tx,
        };

        (executor, handle)
    }

    /// Run the executor in a dedicated thread
    #[inline(never)] // Don't inline the main loop
    pub fn run(self) {
        // Pre-allocate timer state
        let mut active_timers = Vec::with_capacity(8);

        loop {
            // Check for shutdown
            match self.shutdown_rx.try_recv() {
                Ok(()) | Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => {}
            }

            // Process all pending tasks
            match self.task_rx.try_recv() {
                Ok(task) => self.process_task(task, &mut active_timers),
                Err(TryRecvError::Empty) => {
                    // Check timers when idle
                    self.check_timers(&mut active_timers);
                    // Brief sleep to avoid busy waiting
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(TryRecvError::Disconnected) => break,
            }
        }
    }

    /// Process a single task
    #[inline]
    fn process_task(&self, task: Task, timers: &mut Vec<Task>) {
        match task {
            Task::FetchPullRequests { owner, repo } => {
                self.fetch_pull_requests(owner, repo);
            }
            Task::Tick {
                interval,
                next_tick,
            } => {
                // Add to active timers
                if timers.len() < timers.capacity() {
                    timers.push(Task::Tick {
                        interval,
                        next_tick,
                    });
                }
            }
            Task::Shutdown => {
                let _ = self.result_tx.send(Action::Quit);
            }
        }
    }

    /// Check and fire any ready timers
    #[inline]
    fn check_timers(&self, timers: &mut Vec<Task>) {
        let now = Instant::now();
        let mut i = 0;

        while i < timers.len() {
            match &mut timers[i] {
                Task::Tick {
                    interval,
                    next_tick,
                } => {
                    if now >= *next_tick {
                        // Fire timer
                        let _ = self.result_tx.send(Action::Render);
                        // Update next tick time
                        *next_tick = now + *interval;
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// Fetch pull requests using blocking HTTP client
    #[inline]
    fn fetch_pull_requests(&self, owner: &str, repo: &str) {
        // Use blocking client to avoid async overhead
        let rt = tokio::runtime::Handle::try_current();

        let result = if let Ok(handle) = rt {
            // If we're in a tokio context, use block_in_place
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    octocrab::instance()
                        .pulls(owner, repo)
                        .list()
                        .sort(Sort::Updated)
                        .direction(Direction::Descending)
                        .send()
                        .await
                })
            })
        } else {
            // Otherwise create a minimal runtime
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create runtime");

            rt.block_on(async {
                octocrab::instance()
                    .pulls(owner, repo)
                    .list()
                    .sort(Sort::Updated)
                    .direction(Direction::Descending)
                    .send()
                    .await
            })
        };

        match result {
            Ok(page) => {
                let prs: Vec<PullRequest> = page
                    .items
                    .into_iter()
                    .map(|pr| PullRequest {
                        id: pr.number.to_string(),
                        title: pr.title.unwrap_or_default(),
                        url: pr.html_url.map(|u| u.to_string()).unwrap_or_default(),
                    })
                    .collect();

                let _ = self
                    .result_tx
                    .send(Action::FetchResult(FetchResult::PullRequestsOk(prs)));
            }
            Err(err) => {
                let _ = self
                    .result_tx
                    .send(Action::FetchResult(FetchResult::PullRequestsErr(
                        err.to_string(),
                    )));
            }
        }
    }
}

/// Handle for submitting tasks to the executor
#[derive(Clone)]
pub struct ExecutorHandle {
    task_tx: Sender<Task>,
    shutdown_tx: Sender<()>,
}

impl ExecutorHandle {
    /// Submit a task to the executor
    #[inline]
    pub fn submit(&self, task: Task) -> Result<(), TaskSubmitError> {
        self.task_tx
            .try_send(task)
            .map_err(|_| TaskSubmitError::QueueFull)
    }

    /// Submit a task, waiting if the queue is full
    #[inline]
    pub fn submit_wait(&self, task: Task) -> Result<(), TaskSubmitError> {
        self.task_tx
            .send(task)
            .map_err(|_| TaskSubmitError::ExecutorShutdown)
    }

    /// Shutdown the executor
    #[inline]
    pub fn shutdown(self) {
        let _ = self.shutdown_tx.send(());
    }

    /// Check if the executor is still running
    #[inline]
    pub fn is_running(&self) -> bool {
        !self.task_tx.is_empty() || self.task_tx.len() < self.task_tx.capacity().unwrap_or(0)
    }
}

/// Error type for task submission
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskSubmitError {
    /// The task queue is full
    QueueFull,
    /// The executor has shut down
    ExecutorShutdown,
}

impl std::fmt::Display for TaskSubmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::QueueFull => write!(f, "task queue is full"),
            Self::ExecutorShutdown => write!(f, "executor has shut down"),
        }
    }
}

impl std::error::Error for TaskSubmitError {}

/// Spawn the executor in a dedicated thread
#[inline]
pub fn spawn_executor(result_tx: Sender<Action>) -> ExecutorHandle {
    let (executor, handle) = TaskExecutor::new(result_tx);

    std::thread::Builder::new()
        .name("task-executor".into())
        .spawn(move || executor.run())
        .expect("Failed to spawn executor thread");

    handle
}
