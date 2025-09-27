use std::{ffi::OsStr, net::IpAddr, process::Command};

use log::warn;

use crate::{
    network::{LocalSocket, Protocol},
    os::ProcessInfo,
};

/// Raw network connection information from lsof output.
#[derive(Debug, Clone)]
pub struct RawConnection {
    /// Remote IP address of the connection.
    #[allow(dead_code)]
    pub remote_ip: IpAddr,
    /// Local IP address of the connection.
    pub local_ip: IpAddr,
    /// Local port number.
    pub local_port: u16,
    /// Remote port number.
    #[allow(dead_code)]
    pub remote_port: u16,
    /// Network protocol (TCP/UDP).
    pub protocol: Protocol,
    /// Process information for this connection.
    #[allow(dead_code)]
    pub proc_info: ProcessInfo,
}

#[inline]
const fn get_null_addr_v4() -> IpAddr {
    IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0))
}

#[inline]
const fn get_null_addr_v6() -> IpAddr {
    IpAddr::V6(std::net::Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0))
}

#[inline]
fn get_null_addr(is_ipv4: bool) -> IpAddr {
    if is_ipv4 {
        get_null_addr_v4()
    } else {
        get_null_addr_v6()
    }
}

#[inline]
fn parse_connection_string(
    connection_str: &str,
    is_ipv4: bool,
) -> Option<(IpAddr, u16, IpAddr, u16)> {
    // Find the -> separator for connected sockets
    if let Some(arrow_pos) = connection_str.find("->") {
        let (local_part, remote_part) = connection_str.split_at(arrow_pos);
        let remote_part = &remote_part[2..]; // Skip the "->"

        let (local_ip, local_port) = parse_ip_port(local_part)?;
        let (remote_ip, remote_port) = parse_ip_port(remote_part)?;

        Some((local_ip, local_port, remote_ip, remote_port))
    } else {
        // Listening socket
        let (local_ip, local_port) = parse_ip_port_listen(connection_str, is_ipv4)?;
        let remote_ip = get_null_addr(is_ipv4);
        Some((local_ip, local_port, remote_ip, 0))
    }
}

#[inline]
fn parse_ip_port(addr_str: &str) -> Option<(IpAddr, u16)> {
    // Handle IPv6 addresses in brackets
    if addr_str.starts_with('[') {
        let end_bracket = addr_str.find(']')?;
        let ip_part = &addr_str[1..end_bracket];
        let port_part = &addr_str[end_bracket + 2..]; // Skip "]:"

        let ip = ip_part.parse().ok()?;
        let port = port_part.parse().ok()?;
        Some((ip, port))
    } else {
        // IPv4 or IPv6 without brackets
        let colon_pos = addr_str.rfind(':')?;
        let ip_part = &addr_str[..colon_pos];
        let port_part = &addr_str[colon_pos + 1..];

        let ip = ip_part.parse().ok()?;
        let port = port_part.parse().ok()?;
        Some((ip, port))
    }
}

#[inline]
fn parse_ip_port_listen(addr_str: &str, is_ipv4: bool) -> Option<(IpAddr, u16)> {
    // Handle IPv6 addresses in brackets
    if addr_str.starts_with('[') {
        let end_bracket = addr_str.find(']')?;
        let ip_part = &addr_str[1..end_bracket];
        let port_part = &addr_str[end_bracket + 2..]; // Skip "]:"

        let ip = if ip_part.is_empty() || ip_part == "*" {
            get_null_addr(is_ipv4)
        } else {
            ip_part.parse().ok()?
        };

        let port = if port_part == "*" {
            0
        } else {
            port_part.parse().ok()?
        };

        Some((ip, port))
    } else {
        // IPv4 or IPv6 without brackets
        let colon_pos = addr_str.rfind(':')?;
        let ip_part = &addr_str[..colon_pos];
        let port_part = &addr_str[colon_pos + 1..];

        let ip = if ip_part.is_empty() || ip_part == "*" {
            get_null_addr(is_ipv4)
        } else {
            ip_part.parse().ok()?
        };

        let port = if port_part == "*" {
            0
        } else {
            port_part.parse().ok()?
        };

        Some((ip, port))
    }
}

#[inline]
fn decode_process_name(name: &str) -> String {
    if name.contains("\\x20") {
        name.replace("\\x20", " ")
    } else {
        name.to_string()
    }
}

impl RawConnection {
    /// Parse a raw lsof output line into a RawConnection.
    #[inline]
    pub fn new(raw_line: &str) -> Option<RawConnection> {
        let mut columns = raw_line.split_ascii_whitespace();

        let process_name_raw = columns.next()?;
        let pid_str = columns.next()?;
        let _username = columns.next()?; // Skip username
        let _fd = columns.next()?; // Skip file descriptor
        let ip_type = columns.next()?;
        let _device = columns.next()?; // Skip device
        let _size = columns.next()?; // Skip size
        let protocol_str = columns.next()?;
        let connection_str = columns.next()?;

        let is_ipv4 = ip_type.contains('4');

        let protocol = match protocol_str {
            "TCP" => Protocol::Tcp,
            "UDP" => Protocol::Udp,
            // Ignore protocols that don't represent trackable network connections
            "NODE" => {
                // NODE represents filesystem nodes, not network connections
                return None;
            }
            "ICMPV6" | "ICMP" => {
                // ICMP/ICMPV6 are control protocols, not data connections we want to track
                return None;
            }
            "PIPE" | "FIFO" => {
                // Named pipes and FIFOs are not network connections
                return None;
            }
            "unix" | "UNIX" => {
                // Unix domain sockets are not network connections
                return None;
            }
            "RAW" => {
                // Raw sockets are typically not user data connections
                return None;
            }
            unknown => {
                warn!("Unknown protocol in lsof output: {unknown}");
                return None;
            }
        };

        let pid = match pid_str.parse() {
            Ok(pid) => pid,
            Err(e) => {
                warn!("Failed to parse PID '{pid_str}': {e}");
                return None;
            }
        };
        let process_name = decode_process_name(process_name_raw);
        let proc_info = ProcessInfo::new(&process_name, pid);

        let (local_ip, local_port, remote_ip, remote_port) =
            parse_connection_string(connection_str, is_ipv4)?;

        Some(RawConnection {
            local_ip,
            local_port,
            remote_ip,
            remote_port,
            protocol,
            proc_info,
        })
    }

    /// Get the protocol of this connection.
    #[allow(dead_code)]
    #[inline]
    pub const fn get_protocol(&self) -> Protocol {
        self.protocol
    }

    /// Get the local IP address.
    #[allow(dead_code)]
    #[inline]
    pub const fn get_local_ip(&self) -> IpAddr {
        self.local_ip
    }

    /// Get the local port number.
    #[allow(dead_code)]
    #[inline]
    pub const fn get_local_port(&self) -> u16 {
        self.local_port
    }

    /// Get the remote IP address.
    #[allow(dead_code)]
    #[inline]
    pub const fn get_remote_ip(&self) -> IpAddr {
        self.remote_ip
    }

    /// Get the remote port number.
    #[allow(dead_code)]
    #[inline]
    pub const fn get_remote_port(&self) -> u16 {
        self.remote_port
    }

    /// Convert to a LocalSocket representation.
    #[inline]
    pub const fn as_local_socket(&self) -> LocalSocket {
        LocalSocket {
            ip: self.local_ip,
            port: self.local_port,
            protocol: self.protocol,
        }
    }
}

/// Get all network connections from the system using lsof.
#[inline]
pub fn get_connections() -> std::io::Result<OwnedRawConnections> {
    let content = run(["-n", "-P", "-i4", "-i6", "+c", "0"])?;
    Ok(create_owned_connections(content))
}

#[inline]
fn run<I, S>(args: I) -> std::io::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("lsof").args(args).output()?;

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Iterator over raw connections from lsof output.
#[allow(dead_code)]
pub struct RawConnections<'a> {
    lines: std::str::Lines<'a>,
}

impl<'a> RawConnections<'a> {
    /// Create a new iterator over lsof output lines.
    #[allow(dead_code)]
    #[inline]
    pub fn new(content: &'a str) -> Self {
        Self {
            lines: content.lines(),
        }
    }
}

impl<'a> Iterator for RawConnections<'a> {
    type Item = RawConnection;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line = self.lines.next()?;
            if let Some(connection) = RawConnection::new(line) {
                return Some(connection);
            }
        }
    }
}

/// Factory function that returns owned iterator for cases where content ownership is needed.
#[inline]
pub fn create_owned_connections(content: String) -> OwnedRawConnections {
    OwnedRawConnections::new(content)
}

/// Owned iterator over raw connections that owns the lsof output string.
pub struct OwnedRawConnections {
    content: String,
    position: usize,
}

impl OwnedRawConnections {
    /// Create a new owned connection iterator.
    #[inline]
    pub fn new(content: String) -> Self {
        Self {
            content,
            position: 0,
        }
    }
}

impl Iterator for OwnedRawConnections {
    type Item = RawConnection;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.position < self.content.len() {
            let remaining = &self.content[self.position..];
            if let Some(newline_pos) = remaining.find('\n') {
                let line = &remaining[..newline_pos];
                self.position += newline_pos + 1;

                if let Some(connection) = RawConnection::new(line) {
                    return Some(connection);
                }
            } else {
                // Last line without newline
                let line = remaining;
                self.position = self.content.len();

                if !line.is_empty() {
                    if let Some(connection) = RawConnection::new(line) {
                        return Some(connection);
                    }
                }
                break;
            }
        }
        None
    }
}
