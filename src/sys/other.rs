use std::{
    io::{self, ErrorKind},
    net::SocketAddr,
};

use socket2::Socket;

#[derive(Debug)]
pub struct MptcpSocketBuilder;

impl MptcpSocketBuilder {
    pub fn new_v4() -> io::Result<Self> {
        Err(ErrorKind::Unsupported.into())
    }

    pub fn new_v6() -> io::Result<Self> {
        Err(ErrorKind::Unsupported.into())
    }

    pub fn new_for_addr(_addr: SocketAddr) -> io::Result<Self> {
        Err(ErrorKind::Unsupported.into())
    }

    pub fn set_nonblocking(&self) -> io::Result<Self> {
        Err(ErrorKind::Unsupported.into())
    }

    pub fn connect(self, _addr: SocketAddr) -> io::Result<Socket> {
        Err(ErrorKind::Unsupported.into())
    }

    pub fn bind(self, _addr: SocketAddr) -> io::Result<Socket> {
        Err(ErrorKind::Unsupported.into())
    }
}

pub struct MptcpSocketRef<'a, S>(&'a S);

impl<'a, S> MptcpSocketRef<'a, S> {
    pub fn is_mptcp_socket(&self) -> bool {
        false
    }

    pub fn has_fallback(&self) -> bool {
        false
    }
}

impl<'a, S> From<&'a S> for MptcpSocketRef<'a, S> {
    fn from(socket: &'a S) -> Self {
        Self(socket)
    }
}
