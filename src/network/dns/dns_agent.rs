//! Background DNS resolution thread.
//!
//! Performs DNS lookups in a dedicated thread to avoid blocking.
use std::{collections::HashMap, net::IpAddr, thread};

use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use hickory_proto::xfer::Protocol;
use hickory_resolver::{
    config::{NameServerConfig, ResolverConfig, ResolverOpts},
    name_server::TokioConnectionProvider,
    TokioResolver,
};

const QUEUE_CAPACITY: usize = 1000;
const INITIAL_CACHE_CAPACITY: usize = 2048;

/// Starts a DNS resolution thread and returns channels for communication.
///
/// # Arguments
///
/// * `dns_server` - Optional DNS server to use for resolution
///
/// # Returns
///
/// Tuple of (sender for resolution requests, receiver for resolution results)
pub fn start_dns_thread(
    dns_server: Option<std::net::Ipv4Addr>,
) -> (Sender<Vec<IpAddr>>, Receiver<(IpAddr, String)>) {
    // Channel for sending IP addresses to resolve
    let (request_tx, request_rx) = bounded::<Vec<IpAddr>>(QUEUE_CAPACITY);

    // Channel for receiving resolution results
    let (result_tx, result_rx) = unbounded::<(IpAddr, String)>();

    // Channel for internal cache updates
    let (cache_tx, cache_rx) = unbounded::<(IpAddr, String)>();

    // Spawn a dedicated thread for DNS resolution
    thread::spawn(move || {
        // Create a tokio runtime for async DNS resolution
        let rt = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(e) => {
                eprintln!("Failed to create tokio runtime: {}", e);
                return;
            }
        };

        // Initialize the DNS resolver
        let resolver = rt.block_on(async {
            // Create configuration based on the input
            let config = if let Some(dns_ip) = dns_server {
                // Create a custom configuration with DNS server
                let mut config = ResolverConfig::new();
                let socket = std::net::SocketAddr::new(IpAddr::V4(dns_ip), 53);
                let nameserver_config = NameServerConfig {
                    socket_addr: socket,
                    protocol: Protocol::Udp,
                    tls_dns_name: None,
                    trust_negative_responses: false,
                    bind_addr: None,
                    http_endpoint: None,
                };
                config.add_name_server(nameserver_config);
                config
            } else {
                // Use default configuration
                ResolverConfig::default()
            };

            // Create optimized options
            let mut opts = ResolverOpts::default();
            opts.cache_size = 2048; // Larger cache
            opts.attempts = 1; // Single attempt for faster lookups

            // Create a builder and build the resolver
            let mut builder =
                TokioResolver::builder_with_config(config, TokioConnectionProvider::default());
            *builder.options_mut() = opts;
            builder.build()
        });

        // Pre-allocate a cache with reasonable capacity
        let mut hostname_cache: HashMap<IpAddr, String> =
            HashMap::with_capacity(INITIAL_CACHE_CAPACITY);

        // Process requests until the channel is closed
        while let Ok(ip_batch) = request_rx.recv() {
            // Check for any cache updates from previous async tasks
            while let Ok((ip, hostname)) = cache_rx.try_recv() {
                hostname_cache.insert(ip, hostname);
            }

            for ip in ip_batch {
                // Skip if already in cache
                if let Some(hostname) = hostname_cache.get(&ip) {
                    let _ = result_tx.send((ip, hostname.clone()));
                    continue;
                }

                // Capture variables for the closure
                let tx = result_tx.clone();
                let cache_tx = cache_tx.clone();
                let resolver = resolver.clone();
                let ip_for_task = ip;

                // Spawn a task to resolve this IP
                rt.spawn(async move {
                    // Attempt to get hostname for this IP
                    let hostname = match resolver.reverse_lookup(ip_for_task).await {
                        Ok(response) => {
                            // Take the first name from the response
                            if let Some(name) = response.into_iter().next() {
                                name.to_string()
                            } else {
                                // No hostname found, use the IP string
                                ip_for_task.to_string()
                            }
                        }
                        Err(_) => {
                            // Resolution failed, use the IP string
                            ip_for_task.to_string()
                        }
                    };

                    // Send the result to the client
                    let _ = tx.send((ip_for_task, hostname.clone()));

                    // Also send to our cache update channel
                    let _ = cache_tx.send((ip_for_task, hostname));
                });
            }

            // No need to read from result_rx since we're returning it to the caller
        }

        // When channel closes, shutdown the runtime
        drop(rt);
    });

    (request_tx, result_rx)
}
