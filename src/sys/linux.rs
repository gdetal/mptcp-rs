use std::{
    io,
    mem::{size_of, MaybeUninit},
    net::SocketAddr,
    os::fd::{AsRawFd, RawFd},
};

use semver::{Version, VersionReq};
use socket2::{Domain, Protocol, Socket, Type};
use sysctl::Sysctl;
use sysinfo::System;

lazy_static::lazy_static! {
    static ref KERNEL_VERSION_REQ: VersionReq = {
        VersionReq::parse(">=5.16").unwrap()
    };
}

#[derive(Debug)]
pub struct MptcpSocketBuilder(Socket);

impl MptcpSocketBuilder {
    fn new(domain: Domain) -> io::Result<Self> {
        Ok(Self(Socket::new(
            domain,
            Type::STREAM,
            Some(Protocol::MPTCP),
        )?))
    }

    pub fn new_v4() -> io::Result<Self> {
        Self::new(Domain::IPV4)
    }

    pub fn new_v6() -> io::Result<Self> {
        Self::new(Domain::IPV4)
    }

    pub fn new_for_addr(addr: SocketAddr) -> io::Result<Self> {
        Self::new(Domain::for_address(addr))
    }

    pub fn set_nonblocking(self) -> io::Result<Self> {
        self.0.set_nonblocking(true)?;
        Ok(self)
    }

    pub fn connect(self, addr: SocketAddr) -> io::Result<Socket> {
        let socket = self.0;

        match socket
            .connect(&addr.into())
            .map_err(|e| (e.raw_os_error(), e))
        {
            Err((Some(errno), err)) if errno != libc::EINPROGRESS => Err(err),
            _ => Ok(socket),
        }
    }

    pub fn bind(self, addr: SocketAddr) -> io::Result<Socket> {
        let socket = self.0;
        socket.bind(&addr.into())?;
        socket.listen(0)?;
        Ok(socket)
    }
}

pub struct MptcpSocketRef<'a, S>(&'a S);

impl<'a, S: AsRawFd> MptcpSocketRef<'a, S> {
    pub fn is_mptcp_socket(&self) -> bool {
        unsafe {
            getsockopt::<libc::c_int>(self.0.as_raw_fd(), libc::SOL_SOCKET, libc::SO_PROTOCOL)
                .map_or(false, |v| v == libc::IPPROTO_MPTCP)
        }
    }

    pub fn has_fallback(&self) -> bool {
        const SOL_MPTCP: libc::c_int = 0x11c;
        const MPTCP_INFO: libc::c_int = 0x1;

        if !has_mptcp_info() {
            return !self.is_mptcp_socket();
        }

        unsafe {
            getsockopt::<libc::c_int>(self.0.as_raw_fd(), SOL_MPTCP, MPTCP_INFO)
                .ok()
                .is_some()
        }
    }
}

impl<'a, S> From<&'a S> for MptcpSocketRef<'a, S> {
    fn from(socket: &'a S) -> Self {
        Self(socket)
    }
}

unsafe fn getsockopt<T>(fd: RawFd, opt: libc::c_int, val: libc::c_int) -> io::Result<T> {
    let mut payload: MaybeUninit<T> = MaybeUninit::uninit();
    let mut len = size_of::<T>() as libc::socklen_t;

    match libc::getsockopt(fd, opt, val, payload.as_mut_ptr().cast(), &mut len) {
        -1 => Err(std::io::Error::last_os_error()),
        _ => Ok(payload.assume_init()),
    }
}

pub(crate) fn has_mptcp_info() -> bool {
    if let Some(ver) = System::kernel_version() {
        if let Ok(version) = Version::parse(&ver) {
            return KERNEL_VERSION_REQ.matches(&version);
        }
    }

    false
}

pub(crate) fn is_mptcp_enabled() -> bool {
    let ctl = if cfg!(target_os = "linux") {
        sysctl::Ctl::new("net.mptcp.enabled")
    } else {
        return false;
    };

    if let Ok(ctl) = ctl {
        if let Ok(val) = ctl.value() {
            if let Some(val) = val.as_string() {
                return val == "1";
            }
        }
    }

    false
}
