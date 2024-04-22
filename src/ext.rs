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
    /// # Example
    ///
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

    /// Returns whether the socket has fallback to use TCP while being
    /// created as MPTCP socket.
    ///
    /// Note: this only works on Linux kernel versions >= 5.16.
    ///
    /// Returns `true` if the socket has fallback, `false` otherwise.
    ///
    fn has_fallback(&self) -> bool {
        sys::has_fallback(self.as_raw_fd())
    }
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;
    use std::net::TcpStream;

    use crate::ext::MptcpExt;
    use crate::sys::{has_mptcp_info, is_mptcp_enabled};
    use crate::MptcpSocket;
    use crate::{MptcpListenerExt, MptcpStreamExt};

    #[test]
    fn test_mptcp() {
        let mptcp_enabled = is_mptcp_enabled();

        if !mptcp_enabled {
            // If the system does not supports MPTCP, we cannot test whether we can detect fallback
            return;
        }

        let listener = TcpListener::bind_mptcp("127.0.0.1:0").unwrap();

        let local_addr = listener.local_addr().unwrap();

        let stream = TcpStream::connect_mptcp(local_addr);
        assert!(matches!(stream, Ok(MptcpSocket::Mptcp(..))));

        let stream = stream.unwrap();

        assert!(stream.use_mptcp());
        // Can only assert on >= 5.16 kernels
        if has_mptcp_info() {
            assert!(!stream.has_fallback());
        }
    }

    #[test]
    fn test_mptcp_fallback() {
        let mptcp_enabled = is_mptcp_enabled();

        if !mptcp_enabled {
            // If the system does not supports MPTCP, we cannot test whether we can detect fallback
            return;
        }

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();

        let local_addr = listener.local_addr().unwrap();

        let stream = TcpStream::connect_mptcp(local_addr);
        assert!(matches!(stream, Ok(MptcpSocket::Mptcp(..))));

        let stream = stream.unwrap();

        assert!(stream.use_mptcp());
        assert!(stream.has_fallback());
    }

    #[test]
    fn test_tcp() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let local_addr = listener.local_addr().unwrap();

        let stream = TcpStream::connect(local_addr).unwrap();

        assert!(!stream.use_mptcp());
    }
}
