use serde::{Deserialize, Serialize};

/// Represents a GitHub pull request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequest {
    /// Unique identifier for the pull request
    pub id: String,
    /// Pull request number
    pub number: u64,
    /// Title of the pull request
    pub title: String,
    /// Username of the pull request author
    pub user: String,
    /// Current state (e.g., "open", "closed", "merged")
    pub state: String,
    /// ISO 8601 timestamp of when the PR was created
    pub created_at: String,
    /// ISO 8601 timestamp of when the PR was last updated
    pub updated_at: String,
    /// URL to the pull request on GitHub
    pub url: String,
    /// Whether this is a draft pull request
    pub draft: bool,
    /// Labels attached to the pull request
    pub labels: Vec<String>,
}

/// Result type for fetch operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FetchResult {
    /// Successfully fetched pull requests
    PullRequestsOk(Vec<PullRequest>),
    /// Error fetching pull requests
    PullRequestsErr(String),
}
