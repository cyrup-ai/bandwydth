use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};

use hickory_proto::xfer::Protocol;
use hickory_resolver::{
    config::{NameServerConfig, ResolverConfig, ResolverOpts},
    name_server::TokioConnectionProvider,
    TokioResolver,
};

/// Trait for performing DNS lookups on IP addresses.
pub trait Lookup {
    /// Perform a reverse DNS lookup on an IP address.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address to resolve
    ///
    /// # Returns
    ///
    /// The hostname if resolution succeeds, None if it fails
    fn lookup(&self, ip: IpAddr) -> Option<String>;
}

/// DNS resolver using hickory for performing lookups.
pub struct Resolver(TokioResolver);

impl Resolver {
    /// Create a new DNS resolver with optional custom DNS server.
    ///
    /// # Arguments
    ///
    /// * `dns_server` - Optional custom DNS server IP address
    ///
    /// # Returns
    ///
    /// A new resolver instance
    pub async fn new(dns_server: Option<Ipv4Addr>) -> eyre::Result<Self> {
        // Create configuration based on the input
        let config = if let Some(dns_server_address) = dns_server {
            // Create a custom configuration with the specified DNS server
            let mut config = ResolverConfig::new();
            let socket = SocketAddr::V4(SocketAddrV4::new(dns_server_address, 53));
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

        // Create options
        let opts = ResolverOpts::default();

        // Create a builder and build the resolver
        let mut builder =
            TokioResolver::builder_with_config(config, TokioConnectionProvider::default());
        *builder.options_mut() = opts;
        let resolver = builder.build();

        Ok(Self(resolver))
    }
}

impl Lookup for Resolver {
    fn lookup(&self, ip: IpAddr) -> Option<String> {
        // Try to use existing runtime handle first to avoid creating nested runtimes
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            // We're already in an async runtime context
            handle.block_on(self.0.reverse_lookup(ip))
        } else {
            // Create new runtime only if no current runtime exists
            let rt = tokio::runtime::Runtime::new().ok()?;
            rt.block_on(self.0.reverse_lookup(ip))
        };
        
        match result {
            Ok(names) => {
                // Take the first result and convert it to a string
                names.into_iter().next().map(|name| name.to_string())
            }
            Err(_) => {
                // If lookup fails for any reason, use the IP as string
                Some(ip.to_string())
            }
        }
    }
}
