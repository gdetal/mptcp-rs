use std::{io, net::ToSocketAddrs, os::fd::AsRawFd};

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
