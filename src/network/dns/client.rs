use std::{net::IpAddr, time::Duration};

use crate::concurrent::{unbounded, OpportunisticDebouncedTicker, Receiver, Tickable};

pub use crate::network::dns::{resolver::Lookup, IpTable};

/// DNS client for asynchronous hostname resolution.
///
/// This client receives DNS results from the main executor and maintains a cache.
/// High-performance, zero-allocation DNS client
pub struct Client {
    cache_rx: Receiver<(IpAddr, String)>,
    cache: IpTable,
    dns_server: Option<std::net::Ipv4Addr>,
}

impl Client {
    /// Creates a new DNS client.
    ///
    /// # Returns
    ///
    /// A new `Client` instance
    pub fn new(dns_server: Option<std::net::Ipv4Addr>) -> Self {
        let (_cache_tx, cache_rx) = unbounded::<(IpAddr, String)>();

        Self {
            cache_rx,
            cache: IpTable::new(),
            dns_server,
        }
    }

    /// Submits a batch of IP addresses for asynchronous DNS resolution.
    ///
    /// Uses the main executor's ResolveDns task.
    ///
    /// # Arguments
    ///
    /// * `ips` - Vector of IP addresses to resolve
    /// * `executor` - Handle to the main task executor
    pub fn resolve(&mut self, ips: Vec<IpAddr>, executor: &crate::concurrent::ExecutorHandle) {
        if !ips.is_empty() {
            let _ = executor.submit(crate::concurrent::Task::ResolveDns { ips });
        }
    }

    /// Returns a copy of the current DNS cache.
    ///
    /// # Returns
    ///
    /// A hashmap mapping IP addresses to their resolved hostnames
    pub fn cache(&mut self) -> IpTable {
        // Drain any new cache entries first
        while let Ok((cached_ip, name)) = self.cache_rx.try_recv() {
            self.cache.insert(cached_ip, name);
        }
        self.cache.clone()
    }

    /// Get the configured DNS server address
    ///
    /// # Returns
    ///
    /// The optional IPv4 address of the configured DNS server
    #[inline(always)]
    pub fn dns_server(&self) -> Option<std::net::Ipv4Addr> {
        self.dns_server
    }
}

/// Tickable DNS client that implements the opportunistic event pattern
/// Tickable DNS client for use with opportunistic ticking
pub struct TickableDnsClient<F>
where
    F: FnMut(IpTable) + Send + 'static,
{
    client: Client,
    callback: F,
    period: Duration,
    active: bool,
}

impl<F> TickableDnsClient<F>
where
    F: FnMut(IpTable) + Send + 'static,
{
    /// Create a new tickable DNS client
    pub fn new(period_ms: u64, dns_server: Option<std::net::Ipv4Addr>, callback: F) -> Self {
        let period = Duration::from_millis(period_ms);
        let client = Client::new(dns_server);

        Self {
            client,
            callback,
            period,
            active: true,
        }
    }

    /// Get mutable access to the underlying client
    pub fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }

    /// Enable or disable DNS updates
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

impl<F> Tickable for TickableDnsClient<F>
where
    F: FnMut(IpTable) + Send + 'static,
{
    fn tick(&mut self) {
        if !self.active {
            return;
        }

        let cache = self.client.cache();
        (self.callback)(cache);
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn frame_interval(&self) -> Duration {
        self.period
    }
}

/// Handle for controlling a running DNS client using opportunistic ticking
pub struct DnsHandle<F>
where
    F: FnMut(IpTable) + Send + 'static,
{
    ticker: OpportunisticDebouncedTicker<TickableDnsClient<F>>,
}

impl<F> DnsHandle<F>
where
    F: FnMut(IpTable) + Send + 'static,
{
    /// Stop the DNS client
    pub fn stop(mut self) {
        self.ticker.target_mut().set_active(false);
    }

    /// Check if a tick should happen on this event
    pub fn on_any_event(&mut self) -> bool {
        self.ticker.on_any_event()
    }

    /// Schedule a fallback tick
    pub fn schedule_fallback(&mut self) {
        self.ticker.schedule_fallback();
    }

    /// Try to poll for fallback ticks (for sync contexts)
    pub fn try_poll_fallback(&mut self) -> bool {
        self.ticker.try_poll_fallback()
    }

    /// Get mutable access to the DNS client
    pub fn client_mut(&mut self) -> &mut Client {
        self.ticker.target_mut().client_mut()
    }
}

/// Start a DNS client using opportunistic ticking.
///
/// # Arguments
///
/// * `period_ms` - Update interval in milliseconds
/// * `on_update` - Callback function called with DNS cache updates
///
/// # Returns
///
/// A `DnsHandle` that can be used to control the DNS client.
pub fn start_dns_client<F>(
    period_ms: u64,
    dns_server: Option<std::net::Ipv4Addr>,
    on_update: F,
) -> DnsHandle<F>
where
    F: FnMut(IpTable) + Send + 'static,
{
    let tickable_client = TickableDnsClient::new(period_ms, dns_server, on_update);
    let ticker = OpportunisticDebouncedTicker::new(tickable_client);

    DnsHandle { ticker }
}
