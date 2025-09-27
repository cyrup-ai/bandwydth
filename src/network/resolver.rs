// src/network/resolver.rs
// ------------------------
use std::{net::IpAddr, sync::Mutex};

use crate::network::dns::{Client, IpTable, Resolver};
use tokio::runtime::Runtime;

/// Shared DNS client for managing resolution across threads.
pub struct SharedDns {
    client: Mutex<Client>,
    #[allow(dead_code)]
    runtime: Runtime,
    executor: crate::concurrent::ExecutorHandle,
}

impl SharedDns {
    /// Create a new shared DNS client.
    ///
    /// # Arguments
    ///
    /// * `runtime` - Tokio runtime for async operations
    /// * `upstream` - Optional upstream DNS server
    ///
    /// # Returns
    ///
    /// A new shared DNS client
    pub async fn new(
        _runtime: Runtime,
        upstream: Option<std::net::Ipv4Addr>,
        executor: crate::concurrent::ExecutorHandle,
    ) -> eyre::Result<Self> {
        // Create a resolver asynchronously
        let _resolver = Resolver::new(upstream).await?;
        let client = Client::new(upstream);

        Ok(Self {
            client: Mutex::new(client),
            runtime: Runtime::new()?, // Create a new runtime for internal use
            executor,
        })
    }

    /// Get a snapshot of the current DNS cache.
    ///
    /// # Returns
    ///
    /// Current IP to hostname mappings
    pub fn cache(&self) -> IpTable {
        self.client
            .lock()
            .map(|mut client| client.cache())
            .unwrap_or_default()
    }

    /// Submit a batch of IP addresses for background resolution.
    ///
    /// # Arguments
    ///
    /// * `ips` - Iterator of IP addresses to resolve
    pub fn resolve_batch(&self, ips: impl IntoIterator<Item = IpAddr>) {
        if let Ok(mut client) = self.client.lock() {
            client.resolve(ips.into_iter().collect(), &self.executor);
        }
    }
}
