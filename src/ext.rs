use std::os::fd::AsRawFd;

use crate::sys;

/// Represents whether to fallback to TCP in case MPTCP isn't available.
pub enum MptcpOpt {
    /// Fallback to TCP if MPTCP isn't available.
    Fallback,
    /// Do not fallback to TCP if MPTCP isn't available.
    NoFallback,
}

/// A trait for extending the functionality of types that implement `AsRawFd`.
pub trait MptcpExt: AsRawFd {
    /// Checks if the socket is using Multipath TCP (MPTCP).
    ///
    /// Returns `true` if the socket is using MPTCP, `false` otherwise.
    ///
    /// Example:
    /// ```rust
    /// use std::net::TcpStream;
    /// use mptcp::{MptcpExt, MptcpStreamExt};
    ///
    /// let stream = TcpStream::connect_mptcp("example.com:80").unwrap();
    /// println!("Stream is using Mptcp: {}", stream.use_mptcp());
    /// ```
    ///
    fn use_mptcp(&self) -> bool {
        sys::is_mptcp_socket(self.as_raw_fd())
    }
}
