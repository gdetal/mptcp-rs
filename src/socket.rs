use std::ops::{Deref, DerefMut};

/// Represents a Multipath TCP (MPTCP) socket.
///
/// The `MptcpSocket` struct is a generic type that can hold either an MPTCP
/// socket or a TCP socket.
#[derive(Debug, Clone)]
pub enum MptcpSocket<T> {
    /// The underlying socket is an MPTCP socket.
    Mptcp(T),
    /// The underlying socket is a TCP socket.
    Tcp(T),
}

impl<T> MptcpSocket<T> {
    /// Converts the `MptcpSocket` into the underlying socket.
    pub fn into_socket(self) -> T {
        match self {
            Self::Mptcp(sock) => sock,
            Self::Tcp(sock) => sock,
        }
    }
}

impl<T> Deref for MptcpSocket<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Mptcp(sock) => sock,
            Self::Tcp(sock) => sock,
        }
    }
}

impl<T> DerefMut for MptcpSocket<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Mptcp(sock) => sock,
            Self::Tcp(sock) => sock,
        }
    }
}
