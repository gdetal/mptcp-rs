use std::{
    io,
    mem::{size_of, MaybeUninit},
    net::SocketAddr,
    os::fd::{AsRawFd, RawFd},
};

use semver::Version;
use socket2::{Domain, Protocol, Socket, Type};
use sysctl::Sysctl;
use sysinfo::System;

lazy_static::lazy_static! {
    static ref KERNEL_VERSION : Option<Version> = System::kernel_version().and_then(|v| Version::parse(&v).ok());
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
        const MPTCP_INFO_FLAG_FALLBACK: u32 = 0x1 << 0;

        if !has_mptcp_info() {
            return !self.is_mptcp_socket();
        }

        match unsafe { getsockopt::<MptcpInfo>(self.0.as_raw_fd(), SOL_MPTCP, MPTCP_INFO) } {
            // Error means that it is not using MPTCP:
            Err(_) => true,
            // Could be an MPTCP connection that has latter fallback to TCP:
            Ok(info) => info.mptcpi_flags & MPTCP_INFO_FLAG_FALLBACK != 0,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
struct MptcpInfo {
    mptcpi_subflows: u8,
    mptcpi_add_addr_signal: u8,
    mptcpi_add_addr_accepted: u8,
    mptcpi_subflows_max: u8,
    mptcpi_add_addr_signal_max: u8,
    mptcpi_add_addr_accepted_max: u8,
    mptcpi_flags: u32,
    mptcpi_token: u32,
    mptcpi_write_seq: u64,
    mptcpi_snd_una: u64,
    mptcpi_rcv_nxt: u64,
    mptcpi_local_addr_used: u8,
    mptcpi_local_addr_max: u8,
    mptcpi_csum_enabled: u8,
    mptcpi_retransmits: u32,
    mptcpi_bytes_retrans: u64,
    mptcpi_bytes_sent: u64,
    mptcpi_bytes_received: u64,
    mptcpi_bytes_acked: u64,
    mptcpi_subflows_total: u8,
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
    match KERNEL_VERSION.as_ref() {
        Some(version) => version.major > 5 || (version.major == 5 && version.minor >= 16),
        None => false,
    }
}

pub(crate) fn is_mptcp_enabled() -> bool {
    if let Ok(ctl) = sysctl::Ctl::new("net.mptcp.enabled") {
        if let Ok(val) = ctl.value() {
            if let Some(val) = val.as_string() {
                return val == "1";
            }
        }
    }

    false
}
