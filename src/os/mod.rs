#[cfg(any(target_os = "android", target_os = "linux"))]
mod linux;

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
mod lsof;

/// Utilities for parsing lsof output on macOS and FreeBSD.
#[cfg(any(target_os = "macos", target_os = "freebsd"))]
pub mod lsof_utils;

#[cfg(target_os = "windows")]
mod windows;

mod errors;
pub(crate) mod shared;

pub use shared::*;
