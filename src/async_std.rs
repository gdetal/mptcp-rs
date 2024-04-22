use std::{future::Future, io, net::SocketAddr};

use async_std::net::{TcpListener, TcpStream, ToSocketAddrs};

use crate::{sys, MptcpExt, MptcpOpt, MptcpSocket};

/// Extension trait for async_std::net::TcpStream to support MPTCP.
#[async_trait::async_trait(?Send)]
pub trait MptcpStreamExt {
    type Output;

    /// Establishes an MPTCP connection with the given address and MptcpOpt.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to connect to.
    /// * `opt` - The MptcpOpt options for the connection.
    ///
    /// # Returns
    ///
    /// Returns an `io::Result` containing the MptcpSocket if the connection is successful,
    /// or an `io::Error` if an error occurs during the connection.
    async fn connect_mptcp_opt<A: ToSocketAddrs>(
        addr: A,
        opt: MptcpOpt,
    ) -> io::Result<MptcpSocket<Self::Output>>;

    /// Establishes an MPTCP connection with the given address. If MPTCP cannot be used
    /// the connection will fallback to a regular TCP connection.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to connect to.
    ///
    /// # Returns
    ///
    /// Returns an `io::Result` containing the MptcpSocket if the connection is successful,
    /// or an `io::Error` if an error occurs during the connection.
    async fn connect_mptcp<A: ToSocketAddrs>(addr: A) -> io::Result<MptcpSocket<Self::Output>> {
        Self::connect_mptcp_opt(addr, MptcpOpt::Fallback).await
    }

    /// Establishes an MPTCP connection with the given address. Returns an error even if
    /// MPTCP cannot be used. See `connect_mptcp` for a version that falls back to TCP.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to connect to.
    ///
    /// # Returns
    ///
    /// Returns an `io::Result` containing the MptcpSocket if the connection is successful,
    /// or an `io::Error` if an error occurs during the connection.
    async fn connect_mptcp_force<A: ToSocketAddrs>(addr: A) -> io::Result<Self::Output> {
        Ok(Self::connect_mptcp_opt(addr, MptcpOpt::NoFallback)
            .await?
            .into_socket())
    }
}

/// Extension trait for async_std::net::TcpListener.
#[async_trait::async_trait(?Send)]
pub trait MptcpListenerExt {
    type Output;

    /// Binds an MPTCP socket to the specified address with the given MptcpOpt.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to bind the socket to.
    /// * `opt` - The MptcpOpt to use for the socket.
    ///
    /// # Returns
    ///
    /// Returns an `io::Result` containing the MptcpSocket with the specified MptcpOpt.
    async fn bind_mptcp_opt<A: ToSocketAddrs>(
        addr: A,
        opt: MptcpOpt,
    ) -> io::Result<MptcpSocket<Self::Output>>;

    /// Binds an MPTCP socket to the specified address. If MPTCP cannot be used
    /// the connection will fallback to a regular TCP connection.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to bind the socket to.
    ///
    /// # Returns
    ///
    /// Returns an `io::Result` containing the MptcpSocket with the default MptcpOpt (Fallback).
    async fn bind_mptcp<A: ToSocketAddrs>(addr: A) -> io::Result<MptcpSocket<Self::Output>> {
        Self::bind_mptcp_opt(addr, MptcpOpt::Fallback).await
    }

    /// Binds an MPTCP socket to the specified address. Returns an error even if
    /// MPTCP cannot be used. See `bind_mptcp` for a version that falls back to TCP.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to bind the socket to.
    ///
    /// # Returns
    ///
    /// Returns an `io::Result` containing the MptcpSocket with the MptcpOpt set to NoFallback.
    async fn bind_mptcp_force<A: ToSocketAddrs>(addr: A) -> io::Result<Self::Output> {
        Ok(Self::bind_mptcp_opt(addr, MptcpOpt::NoFallback)
            .await?
            .into_socket())
    }
}

async fn resolve_each_addr<A: ToSocketAddrs, F, Fut, T>(addr: &A, mut f: F) -> io::Result<T>
where
    F: FnMut(SocketAddr) -> Fut,
    Fut: Future<Output = io::Result<T>>,
{
    let addrs = addr.to_socket_addrs().await?;
    let mut last_err = None;
    for addr in addrs {
        match f(addr).await {
            Ok(l) => return Ok(l),
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "could not resolve to any address",
        )
    }))
}

async fn connect_mptcp(addr: SocketAddr) -> io::Result<TcpStream> {
    let socket = sys::mptcp_socket_for_addr(addr)?;
    socket.set_nonblocking(true)?;
    let r = socket.connect(&addr.into());
    match r.map_err(|e| (e.raw_os_error(), e)) {
        Err((Some(errno), err)) if errno != libc::EINPROGRESS => return Err(err),
        _ => {}
    }
    let socket: std::net::TcpStream = socket.into();
    Ok(socket.into())
}

#[async_trait::async_trait(?Send)]
impl MptcpStreamExt for TcpStream {
    type Output = Self;

    async fn connect_mptcp_opt<A: ToSocketAddrs>(
        addr: A,
        opt: MptcpOpt,
    ) -> io::Result<MptcpSocket<Self::Output>> {
        match resolve_each_addr(&addr, connect_mptcp).await {
            Ok(sock) => Ok(MptcpSocket::Mptcp(sock)),
            Err(_) if matches!(opt, MptcpOpt::Fallback) => {
                Ok(MptcpSocket::Tcp(Self::connect(addr).await?))
            }
            Err(err) => Err(err),
        }
    }
}

impl MptcpExt for TcpStream {}

impl From<MptcpSocket<TcpStream>> for TcpStream {
    fn from(socket: MptcpSocket<TcpStream>) -> Self {
        socket.into_socket()
    }
}

async fn bind_mptcp(addr: SocketAddr) -> io::Result<TcpListener> {
    let socket = sys::mptcp_socket_for_addr(addr)?;
    socket.set_nonblocking(true)?;
    socket.bind(&addr.into())?;
    socket.listen(0)?;
    let socket: std::net::TcpListener = socket.into();
    Ok(socket.into())
}

#[async_trait::async_trait(?Send)]
impl MptcpListenerExt for TcpListener {
    type Output = Self;

    async fn bind_mptcp_opt<A: ToSocketAddrs>(
        addr: A,
        opt: MptcpOpt,
    ) -> io::Result<MptcpSocket<Self::Output>> {
        match resolve_each_addr(&addr, bind_mptcp).await {
            Ok(sock) => Ok(MptcpSocket::Mptcp(sock)),
            Err(_) if matches!(opt, MptcpOpt::Fallback) => {
                Ok(MptcpSocket::Tcp(Self::bind(addr).await?))
            }
            Err(err) => Err(err),
        }
    }
}

impl From<MptcpSocket<TcpListener>> for TcpListener {
    fn from(socket: MptcpSocket<TcpListener>) -> Self {
        socket.into_socket()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::net::{IpAddr, Ipv4Addr};

    use crate::sys::is_mptcp_enabled;

    #[tokio::test]
    async fn test_resolve_each_addr() {
        let addr = "127.0.0.1:80";
        let result = resolve_each_addr(&addr, |addr| async move {
            assert_eq!(addr.port(), 80);
            assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
            Ok(())
        })
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_resolve_each_addr_error() {
        let addr = "thisisanerror";
        let result = resolve_each_addr(&addr, |_| async { Ok(()) }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mptcp_socket() {
        let mptcp_enabled = is_mptcp_enabled();

        let listener = TcpListener::bind_mptcp("127.0.0.1:0").await;
        if mptcp_enabled {
            assert!(matches!(listener, Ok(MptcpSocket::Mptcp(..))));
        } else {
            assert!(matches!(listener, Ok(MptcpSocket::Tcp(..))));
        }

        let listener = listener.unwrap().into_socket();
        let local_addr = listener.local_addr().unwrap();

        let stream = TcpStream::connect_mptcp(local_addr).await;
        if mptcp_enabled {
            assert!(matches!(stream, Ok(MptcpSocket::Mptcp(..))));
        } else {
            assert!(matches!(stream, Ok(MptcpSocket::Tcp(..))));
        }
    }

    #[tokio::test]
    async fn test_mptcp_no_fallback() {
        let mptcp_enabled = is_mptcp_enabled();

        if mptcp_enabled {
            // If the system supports MPTCP, we cannot test the no fallback option
            return;
        }

        let listener = TcpListener::bind_mptcp_force("127.0.0.1:0").await;
        assert!(listener.is_err());

        let stream = TcpStream::connect_mptcp_force("127.0.0.1:0").await;
        assert!(stream.is_err());
    }
}
