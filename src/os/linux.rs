use std::collections::HashMap;
use std::sync::Arc;

use procfs::process::FDTarget;

use crate::{
    network::{LocalSocket, Protocol},
    os::ProcessInfo,
    OpenSockets,
};

pub(crate) fn get_open_sockets() -> std::io::Result<OpenSockets> {
    // Pre-size HashMaps with reasonable capacity estimates to minimize reallocations
    let mut open_sockets = HashMap::with_capacity(1024);
    let mut inode_to_proc: HashMap<u64, Arc<ProcessInfo>> = HashMap::with_capacity(4096);

    if let Ok(all_procs) = procfs::process::all_processes() {
        for process in all_procs.filter_map(Result::ok) {
            let (Ok(fds), Ok(stat)) = (process.fd(), process.stat()) else {
                continue;
            };

            // Use Arc to eliminate repeated cloning of ProcessInfo
            let proc_info = Arc::new(ProcessInfo::new(&stat.comm, stat.pid as u32));

            for fd in fds.filter_map(Result::ok) {
                if let FDTarget::Socket(inode) = fd.target {
                    inode_to_proc.insert(inode, Arc::clone(&proc_info));
                }
            }
        }
    }

    macro_rules! insert_proto {
        ($source: expr, $proto: expr) => {
            // Optimize iterator chain with direct iteration and minimal allocations
            for entries_result in $source.into_iter() {
                if let Ok(entries) = entries_result {
                    for entry in entries {
                        if let Some(proc_info) = inode_to_proc.get(&entry.inode) {
                            let socket = LocalSocket {
                                ip: entry.local_address.ip(),
                                port: entry.local_address.port(),
                                protocol: $proto,
                            };
                            open_sockets.insert(socket, Arc::clone(proc_info));
                        }
                    }
                }
            }
        };
    }

    insert_proto!([procfs::net::tcp(), procfs::net::tcp6()], Protocol::Tcp);
    insert_proto!([procfs::net::udp(), procfs::net::udp6()], Protocol::Udp);

    Ok(OpenSockets {
        sockets_to_procs: open_sockets,
    })
}
