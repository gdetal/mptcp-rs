use std::{future::Future, io, net::SocketAddr};

use tokio::net::{lookup_host, TcpListener, TcpSocket, TcpStream, ToSocketAddrs};

use crate::{sys, MptcpExt, MptcpOpt, MptcpSocket};

#[async_trait::async_trait(?Send)]
pub trait MptcpStreamExt {
    type Output;

    async fn connect_mptcp_opt<A: ToSocketAddrs>(
        addr: A,
        opt: MptcpOpt,
    ) -> io::Result<MptcpSocket<Self::Output>>;

    async fn connect_mptcp<A: ToSocketAddrs>(addr: A) -> io::Result<MptcpSocket<Self::Output>> {
        Self::connect_mptcp_opt(addr, MptcpOpt::Fallack).await
    }

    async fn connect_mptcp_force<A: ToSocketAddrs>(addr: A) -> io::Result<Self::Output> {
        Ok(Self::connect_mptcp_opt(addr, MptcpOpt::NoFallback)
            .await?
            .into_socket())
    }
}

#[async_trait::async_trait(?Send)]
pub trait MptcpListenerExt {
    type Output;

    async fn bind_mptcp_opt<A: ToSocketAddrs>(
        addr: A,
        opt: MptcpOpt,
    ) -> io::Result<MptcpSocket<Self::Output>>;

    async fn bind_mptcp<A: ToSocketAddrs>(addr: A) -> io::Result<MptcpSocket<Self::Output>> {
        Self::bind_mptcp_opt(addr, MptcpOpt::Fallack).await
    }

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
    let addrs = lookup_host(addr).await?;
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
    let socket = TcpSocket::from_std_stream(socket.into());
    socket.connect(addr).await
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
            Err(_) if matches!(opt, MptcpOpt::Fallack) => {
                Ok(MptcpSocket::Tcp(Self::connect(addr).await?))
            }
            Err(err) => Err(err),
        }
    }
}

impl MptcpExt for TcpStream {}

async fn bind_mptcp(addr: SocketAddr) -> io::Result<TcpListener> {
    let socket = sys::mptcp_socket_for_addr(addr)?;
    socket.set_nonblocking(true)?;
    socket.bind(&addr.into())?;
    let socket = TcpSocket::from_std_stream(socket.into());
    socket.listen(0)
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
            Err(_) if matches!(opt, MptcpOpt::Fallack) => {
                Ok(MptcpSocket::Tcp(Self::bind(addr).await?))
            }
            Err(err) => Err(err),
        }
    }
}
