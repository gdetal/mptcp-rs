use std::os::fd::AsRawFd;

use crate::sys::MptcpSocketRef;

/// Represents whether to fallback to TCP in case MPTCP isn't available.
pub enum MptcpOpt {
    /// Fallback to TCP if MPTCP isn't available.
    Fallback,
    /// Do not fallback to TCP if MPTCP isn't available.
    NoFallback,
}

pub enum MptcpStatus {
    Tcp,
    Mptcp { has_fallback: bool },
}

/// A trait for extending the functionality of types that implement `AsRawFd`.
pub trait MptcpExt: AsRawFd + Sized {
    /// Returns the MPTCP status of the socket.
    ///
    /// If the socket is using MPTCP, it returns `MptcpStatus::Mptcp` along with
    /// the MPTCP information status. If the socket is not using MPTCP, it returns
    /// `MptcpStatus::Tcp`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::net::TcpStream;
    /// use mptcp::{MptcpExt, MptcpStreamExt, MptcpStatus};
    ///
    /// let stream = TcpStream::connect_mptcp("example.com:80").unwrap();
    ///
    /// match stream.mptcp_status() {
    ///     MptcpStatus::Mptcp { has_fallback } => {
    ///         println!("Stream is using MPTCP with fallback: {}", has_fallback);
    ///     }
    ///     MptcpStatus::Tcp => {
    ///         println!("Stream is using TCP.");
    ///     }
    /// }
    /// ```
    ///
    fn mptcp_status(&self) -> MptcpStatus {
        let sock: MptcpSocketRef<'_, _> = self.into();
        if sock.is_mptcp_socket() {
            return MptcpStatus::Mptcp {
                has_fallback: sock.has_fallback(),
            };
        }
        MptcpStatus::Tcp
    }
}

#[cfg(test)]
mod tests {
    use std::net::{TcpListener, TcpStream};

    use crate::sys::{has_mptcp_info, is_mptcp_enabled};
    use crate::{MptcpExt, MptcpListenerExt, MptcpSocket, MptcpStatus, MptcpStreamExt};

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

        assert!(matches!(
            stream.mptcp_status(),
            MptcpStatus::Mptcp {
                has_fallback: false
            }
        ));
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

        assert!(matches!(stream.mptcp_status(), MptcpStatus::Mptcp { .. }));

        // Can only assert on >= 5.16 kernels
        if has_mptcp_info() {
            assert!(matches!(
                stream.mptcp_status(),
                MptcpStatus::Mptcp { has_fallback: true }
            ));
        }
    }

    #[test]
    fn test_tcp() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let local_addr = listener.local_addr().unwrap();

        let stream = TcpStream::connect(local_addr).unwrap();

        assert!(matches!(stream.mptcp_status(), MptcpStatus::Tcp));
    }
}
