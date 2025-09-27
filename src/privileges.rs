// src/privileges.rs
// -----------------
use std::{
    env, io,
    process::{Command, Stdio},
};

#[cfg(unix)]
use nix::unistd::{geteuid, getuid, setuid};

#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

/// Check if the current process has sufficient privileges for network monitoring
pub fn check_privileges() -> io::Result<bool> {
    #[cfg(unix)]
    {
        let uid = getuid();
        let euid = geteuid();

        // Debug: Print actual UID values
        log::debug!("UID = {}, EUID = {}", uid, euid);
        log::debug!(
            "uid.is_root() = {}, euid.is_root() = {}",
            uid.is_root(),
            euid.is_root()
        );

        // Check if we're running as root or have effective root privileges
        let has_privileges = uid.is_root() || euid.is_root();
        log::debug!("check_privileges() returning: {}", has_privileges);
        Ok(has_privileges)
    }

    #[cfg(windows)]
    {
        Ok(is_elevated_windows())
    }

    #[cfg(not(any(unix, windows)))]
    {
        Ok(false)
    }
}

/// Attempt to escalate privileges if needed for network monitoring
pub fn escalate_privileges() -> io::Result<()> {
    if check_privileges()? {
        return Ok(()); // Already have privileges - SUCCESS
    }

    #[cfg(unix)]
    {
        // This will exec() and never return on success
        escalate_with_sudo()?;
        // If we reach here, escalation failed
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Privilege escalation failed",
        ))
    }

    #[cfg(windows)]
    {
        // This will exec() and never return on success
        escalate_windows()?;
        // If we reach here, escalation failed
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Privilege escalation failed",
        ))
    }

    #[cfg(not(any(unix, windows)))]
    {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Privilege escalation not supported on this platform",
        ))
    }
}

/// Drop privileges after initialization if we escalated them
pub fn drop_privileges() -> io::Result<()> {
    #[cfg(unix)]
    {
        if geteuid().is_root() && !getuid().is_root() {
            // We have effective root but real uid is not root - drop to real uid
            setuid(getuid()).map_err(|e| io::Error::new(io::ErrorKind::PermissionDenied, e))?;
        }
        Ok(())
    }

    #[cfg(windows)]
    {
        // Windows doesn't have the same privilege dropping mechanism
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    {
        Ok(())
    }
}

#[cfg(unix)]
fn escalate_with_sudo() -> io::Result<()> {
    use std::os::unix::process::CommandExt;

    // Get the current executable path
    let exe_path = env::current_exe()?;
    let args: Vec<String> = env::args().collect();

    // Check if sudo is available
    if !Command::new("which")
        .arg("sudo")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?
        .success()
    {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "sudo not found - please run as root or install sudo",
        ));
    }

    // Re-execute with sudo using exec to replace the current process
    let err = Command::new("sudo")
        .env("BANDWYDTH_SKIP_ESCALATION", "1") // Pass environment variable to new process
        .arg(&exe_path)
        .args(&args[1..]) // Skip the program name
        .exec(); // This replaces the current process and never returns on success

    // If we reach here, exec failed
    Err(err)
}

#[cfg(windows)]
fn is_elevated_windows() -> bool {
    use std::ptr;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token: HANDLE = ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut return_length = 0u32;

        let result = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut return_length,
        );

        CloseHandle(token);

        result != 0 && elevation.TokenIsElevated != 0
    }
}

#[cfg(windows)]
fn escalate_windows() -> io::Result<()> {
    use std::ptr;
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::Shell::{ShellExecuteW, SW_NORMAL};

    let exe_path = env::current_exe()?;
    let args: Vec<String> = env::args().skip(1).collect();
    let args_str = args.join(" ");

    let exe_wide: Vec<u16> = OsStr::new(&exe_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let args_wide: Vec<u16> = OsStr::new(&args_str)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let verb_wide: Vec<u16> = OsStr::new("runas")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let result = ShellExecuteW(
            ptr::null_mut() as HWND,
            verb_wide.as_ptr(),
            exe_wide.as_ptr(),
            args_wide.as_ptr(),
            ptr::null(),
            SW_NORMAL,
        );

        if result as i32 > 32 {
            // Success - new elevated process started, exit current process
            std::process::exit(0);
        } else {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Failed to escalate privileges on Windows",
            ));
        }
    }
}

/// Check if we need elevated privileges for the current operation
pub fn requires_elevation_for_operation(operation: &str) -> bool {
    match operation {
        "raw_sockets" => true,
        "pcap_capture" => true,
        "interface_stats" => {
            // Different platforms have different requirements
            #[cfg(target_os = "linux")]
            {
                // On Linux, /proc/net/dev is usually readable without root
                // but some advanced stats might need privileges
                false
            }

            #[cfg(target_os = "macos")]
            {
                // macOS requires privileges to access network statistics via sysinfo
                true
            }

            #[cfg(target_os = "windows")]
            {
                // Windows requires admin privileges for detailed network stats
                true
            }

            #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
            {
                // Default to requiring privileges on unknown platforms
                true
            }
        }
        _ => false,
    }
}
