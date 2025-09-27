use std::collections::HashMap;

use netstat2::*;
use sysinfo::{Pid, ProcessesToUpdate, System};

use crate::{
    network::{LocalSocket, Protocol},
    os::ProcessInfo,
    OpenSockets,
};

#[inline]
pub(crate) fn get_open_sockets() -> std::io::Result<OpenSockets> {
    // Get socket information with optimal flags
    const AF_FLAGS: AddressFamilyFlags = AddressFamilyFlags::IPV4.union(AddressFamilyFlags::IPV6);
    const PROTO_FLAGS: ProtocolFlags = ProtocolFlags::TCP.union(ProtocolFlags::UDP);

    match get_sockets_info(AF_FLAGS, PROTO_FLAGS) {
        Ok(sockets_info) => {
            // Pre-allocate HashMap with exact capacity to eliminate reallocations
            let mut open_sockets = HashMap::with_capacity(sockets_info.len());

            // Initialize system once and reuse throughout iteration
            let mut system = System::new_all();
            system.refresh_processes(ProcessesToUpdate::All, true);

            // Process sockets with zero-copy iteration where possible
            for socket_info in sockets_info {
                // Extract process information with efficient lookup
                let process_info = socket_info
                    .associated_pids
                    .iter()
                    .find_map(|&pid| system.process(Pid::from_u32(pid)))
                    .map(|process| {
                        // Minimize string allocations by using direct access
                        ProcessInfo::new(&process.name().to_string_lossy(), process.pid().as_u32())
                    })
                    .unwrap_or_default();

                // Create socket with branch-optimized pattern matching
                let local_socket = match socket_info.protocol_socket_info {
                    ProtocolSocketInfo::Tcp(tcp_info) => LocalSocket {
                        ip: tcp_info.local_addr,
                        port: tcp_info.local_port,
                        protocol: Protocol::Tcp,
                    },
                    ProtocolSocketInfo::Udp(udp_info) => LocalSocket {
                        ip: udp_info.local_addr,
                        port: udp_info.local_port,
                        protocol: Protocol::Udp,
                    },
                };

                // Single insertion operation
                open_sockets.insert(local_socket, process_info);
            }

            Ok(OpenSockets {
                sockets_to_procs: open_sockets,
            })
        }
        Err(e) => {
            // Convert to io::Error and propagate
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to get socket info: {:?}", e),
            ))
        }
    }
}
