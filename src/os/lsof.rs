use crate::{network::OpenSockets, os::lsof_utils::get_connections};

/// Retrieves currently open local sockets and their associated process information.
pub(crate) fn get_open_sockets() -> std::io::Result<OpenSockets> {
    let sockets_to_procs = get_connections()?
        .map(|raw| (raw.as_local_socket(), raw.proc_info))
        .collect();

    Ok(OpenSockets { sockets_to_procs })
}
