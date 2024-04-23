use std::{
    io,
    mem::{size_of, MaybeUninit},
    net::SocketAddr,
    os::fd::{AsRawFd, RawFd},
    ptr,
};

use socket2::{SockAddr, Socket, Type};
use sysctl::Sysctl;

#[derive(Debug)]
pub struct MptcpSocketBuilder(Socket);

impl MptcpSocketBuilder {
    fn new() -> io::Result<Self> {
        const AF_MULTIPATH: libc::c_int = 39;

        Ok(Self(Socket::new(AF_MULTIPATH.into(), Type::STREAM, None)?))
    }

    pub fn new_v4() -> io::Result<Self> {
        Self::new()
    }

    pub fn new_v6() -> io::Result<Self> {
        Self::new()
    }

    pub fn new_for_addr(_addr: SocketAddr) -> io::Result<Self> {
        Self::new()
    }

    pub fn set_nonblocking(self) -> io::Result<Self> {
        self.0.set_nonblocking(true)?;
        Ok(self)
    }

    pub fn connect(self, addr: SocketAddr) -> io::Result<Socket> {
        let socket = self.0;
        let addr: &SockAddr = &addr.into();

        let sae = libc::sa_endpoints_t {
            sae_srcif: 0,
            sae_srcaddr: ptr::null(),
            sae_srcaddrlen: 0,
            sae_dstaddr: addr.as_ptr(),
            sae_dstaddrlen: addr.len(),
        };

        let ret = match unsafe {
            libc::connectx(
                socket.as_raw_fd(),
                &sae,
                libc::SAE_ASSOCID_ANY,
                0,
                ptr::null(),
                0,
                ptr::null_mut(),
                ptr::null_mut(),
            )
        } {
            -1 => Err(std::io::Error::last_os_error()),
            _ => Ok(()),
        };

        match ret {
            Err(err) if err.raw_os_error() != Some(libc::EINPROGRESS) => Err(err),
            _ => Ok(socket),
        }
    }

    pub fn bind(self, _addr: SocketAddr) -> io::Result<Socket> {
        // bind is not supported for AF_MULTIPATH sockets
        Err(io::ErrorKind::Unsupported.into())
    }
}

pub struct MptcpSocketRef<'a, S>(&'a S);

impl<'a, S: AsRawFd> MptcpSocketRef<'a, S> {
    pub fn is_mptcp_socket(&self) -> bool {
        const MPTCP_SERVICE_TYPE: libc::c_int = 0x213;
        unsafe {
            getsockopt::<libc::c_int>(self.0.as_raw_fd(), libc::IPPROTO_TCP, MPTCP_SERVICE_TYPE)
        }
        .is_ok()
    }

    pub fn has_fallback(&self) -> bool {
        // No way to check for fallback:
        !self.is_mptcp_socket()
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

pub(crate) fn is_mptcp_enabled() -> bool {
    if let Ok(ctl) = sysctl::Ctl::new("net.inet.mptcp.enable") {
        if let Ok(val) = ctl.value() {
            if let Some(val) = val.as_string() {
                return val == "1";
            }
        }
    }

    false
}
