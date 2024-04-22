use std::{
    io::{self, ErrorKind},
    mem::{size_of, MaybeUninit},
    net::SocketAddr,
    os::fd::RawFd,
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

unsafe fn getsockopt<T>(fd: RawFd, opt: libc::c_int, val: libc::c_int) -> io::Result<T> {
    let mut payload: MaybeUninit<T> = MaybeUninit::uninit();
    let mut len = size_of::<T>() as libc::socklen_t;

    match libc::getsockopt(fd, opt, val, payload.as_mut_ptr().cast(), &mut len) {
        -1 => Err(std::io::Error::last_os_error()),
        _ => Ok(payload.assume_init()),
    }
}

pub(crate) fn is_mptcp_socket(fd: RawFd) -> bool {
    if cfg!(target_os = "linux") {
        unsafe {
            getsockopt::<libc::c_int>(fd, libc::SOL_SOCKET, libc::SO_PROTOCOL)
                .map_or(false, |v| v == libc::IPPROTO_MPTCP)
        }
    } else {
        false
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

pub(crate) fn has_fallback(fd: RawFd) -> bool {
    if cfg!(target_os = "linux") {
        const SOL_MPTCP: libc::c_int = 0x11c;
        const MPTCP_INFO: libc::c_int = 0x1;

        if !has_mptcp_info() {
            return !is_mptcp_socket(fd);
        }

        unsafe {
            match getsockopt::<libc::c_int>(fd, SOL_MPTCP, MPTCP_INFO) {
                Err(err) => {
                    println!("Error: {:?}", err);
                    true
                }
                Ok(_) => false,
            }
        }
    } else {
        false
    }
}

pub(crate) fn mptcp_socket(domain: Domain) -> io::Result<Socket> {
    if cfg!(target_os = "linux") {
        Socket::new(domain, Type::STREAM, Some(Protocol::MPTCP))
    } else {
        Err(ErrorKind::Unsupported.into())
    }
}

pub(crate) fn mptcp_socket_for_addr(addr: SocketAddr) -> io::Result<Socket> {
    mptcp_socket(Domain::for_address(addr))
}

pub(crate) fn mptcp_connect(addr: SocketAddr) -> io::Result<Socket> {
    let sock = mptcp_socket(Domain::for_address(addr))?;
    sock.connect(&addr.into())?;
    Ok(sock)
}

pub(crate) fn mptcp_bind(addr: SocketAddr) -> io::Result<Socket> {
    let sock = mptcp_socket(Domain::for_address(addr))?;
    sock.bind(&addr.into())?;
    sock.listen(0)?;
    Ok(sock)
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
