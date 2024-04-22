use std::ops::{Deref, DerefMut};

pub enum MptcpSocket<T> {
    Mptcp(T),
    Tcp(T),
}

impl<T> MptcpSocket<T> {
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
