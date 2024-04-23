#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod darwin;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use darwin::*;

#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "linux")))]
mod other;

#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "linux")))]
pub use other::*;
