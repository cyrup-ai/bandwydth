use std::{collections::HashMap, net::IpAddr};

/// DNS agent for background resolution.
pub mod dns_agent;
pub use dns_agent::*;

/// DNS client for managing resolution requests.
pub mod client;
pub use client::*;

/// DNS resolver implementation.
pub mod resolver;
pub use resolver::*;

/// Mapping of IP addresses to hostnames.
pub type IpTable = HashMap<IpAddr, String>;
