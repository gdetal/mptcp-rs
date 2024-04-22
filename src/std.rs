use std::{
    io,
    net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
    os::fd::AsRawFd,
};

use crate::{sys, MptcpSocket};

pub enum MptcpOpt {
    Fallack,
    NoFallback,
}

pub trait MptcpExt: AsRawFd {
    fn use_mptcp(&self) -> bool {
        sys::is_mptcp_socket(self.as_raw_fd())
    }
}

pub trait MptcpStreamExt {
    type Output;

    fn connect_mptcp_opt<A: ToSocketAddrs>(
        addr: A,
        opt: MptcpOpt,
    ) -> io::Result<MptcpSocket<Self::Output>>;

    fn connect_mptcp<A: ToSocketAddrs>(addr: A) -> io::Result<MptcpSocket<Self::Output>> {
        Self::connect_mptcp_opt(addr, MptcpOpt::Fallack)
    }

    fn connect_mptcp_force<A: ToSocketAddrs>(addr: A) -> io::Result<Self::Output> {
        Ok(Self::connect_mptcp_opt(addr, MptcpOpt::NoFallback)?.into_socket())
    }
}

pub trait MptcpListenerExt {
    type Output;

    fn bind_mptcp_opt<A: ToSocketAddrs>(
        addr: A,
        opt: MptcpOpt,
    ) -> io::Result<MptcpSocket<Self::Output>>;

    fn bind_mptcp<A: ToSocketAddrs>(addr: A) -> io::Result<MptcpSocket<Self::Output>> {
        Self::bind_mptcp_opt(addr, MptcpOpt::Fallack)
    }

    fn bind_mptcp_force<A: ToSocketAddrs>(addr: A) -> io::Result<Self::Output> {
        Ok(Self::bind_mptcp_opt(addr, MptcpOpt::NoFallback)?.into_socket())
    }
}

fn resolve_each_addr<A: ToSocketAddrs, F, T>(addr: &A, mut f: F) -> io::Result<T>
where
    F: FnMut(SocketAddr) -> io::Result<T>,
{
    let addrs = addr.to_socket_addrs()?;
    let mut last_err = None;
    for addr in addrs {
        match f(addr) {
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

impl MptcpStreamExt for TcpStream {
    type Output = Self;

    fn connect_mptcp_opt<A: ToSocketAddrs>(
        addr: A,
        opt: MptcpOpt,
    ) -> io::Result<MptcpSocket<Self::Output>> {
        match resolve_each_addr(&addr, sys::mptcp_connect) {
            Ok(sock) => Ok(MptcpSocket::Mptcp(sock.into())),
            Err(_) if matches!(opt, MptcpOpt::Fallack) => {
                Ok(MptcpSocket::Tcp(Self::connect(addr)?))
            }
            Err(err) => Err(err),
        }
    }
}

impl MptcpExt for TcpStream {}

impl MptcpListenerExt for TcpListener {
    type Output = Self;

    fn bind_mptcp_opt<A: ToSocketAddrs>(
        addr: A,
        opt: MptcpOpt,
    ) -> io::Result<MptcpSocket<Self::Output>> {
        match resolve_each_addr(&addr, sys::mptcp_bind) {
            Ok(sock) => Ok(MptcpSocket::Mptcp(sock.into())),
            Err(_) if matches!(opt, MptcpOpt::Fallack) => Ok(MptcpSocket::Tcp(Self::bind(addr)?)),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod test {
    use std::net::{IpAddr, Ipv4Addr};

    use crate::sys::tests::is_mptcp_enabled;

    use super::*;

    #[test]
    fn test_resolve_each_addr() {
        let addr = "127.0.0.1:80";
        let result = resolve_each_addr(&addr, |addr| {
            assert_eq!(addr.port(), 80);
            assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
            Ok(())
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_each_addr_error() {
        let addr = "thisisanerror";
        let result = resolve_each_addr(&addr, |_| Ok(()));
        assert!(result.is_err());
    }

    #[test]
    fn test_mptcp_socket() {
        let mptcp_enabled = is_mptcp_enabled();

        let listener = TcpListener::bind_mptcp("127.0.0.1:0");
        if mptcp_enabled {
            assert!(matches!(listener, Ok(MptcpSocket::Mptcp(..))));
        } else {
            assert!(matches!(listener, Ok(MptcpSocket::Tcp(..))));
        }

        let listener = listener.unwrap().into_socket();
        let local_addr = listener.local_addr().unwrap();

        let stream = TcpStream::connect_mptcp(local_addr);
        if mptcp_enabled {
            assert!(matches!(stream, Ok(MptcpSocket::Mptcp(..))));
        } else {
            assert!(matches!(stream, Ok(MptcpSocket::Tcp(..))));
        }
    }

    #[test]
    fn test_mptcp_no_fallback() {
        let mptcp_enabled = is_mptcp_enabled();

        if mptcp_enabled {
            // If the system supports MPTCP, we cannot test the no fallback option
            return;
        }

        let listener = TcpListener::bind_mptcp_force("127.0.0.1:0");
        assert!(listener.is_err());

        let stream = TcpStream::connect_mptcp_force("127.0.0.1:0");
        assert!(stream.is_err());
    }
}
