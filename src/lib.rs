#![allow(dead_code)]

//! This is the main module of the `mptcp-rs` crate.
//! It provides the core functionality for working with Multipath TCP (MPTCP) sockets.
//!
//! The crate is organized into several submodules:
//! - `socket`: Contains the MPTCP socket implementation.
//! - `std` (feature: "std"): Provides a standard library implementation for MPTCP.
//! - `tokio` (feature: "tokio"): Provides a Tokio-based implementation for MPTCP.
//! - `async_std` (feature: "async-std"): Provides an async-std-based implementation for MPTCP.
//!
//! Example using Tokio:
//!
//! ```rust
//! use mptcp_rs::tokio::MptcpStreamExt;
//! use std::net::SocketAddr;
//!
//! async fn connect(addr: SocketAddress) -> io::Result<TcpStream> {
//!     TcpStream::connect_mptcp(addr).await.map(|stream| stream.into())
//! }
//! ```
mod ext;
mod socket;
mod sys;

pub use ext::*;
pub use socket::*;

#[cfg(feature = "std")]
mod std;
#[cfg(feature = "std")]
pub use std::*;

#[cfg(feature = "tokio")]
pub mod tokio;

#[cfg(feature = "async-std")]
pub mod async_std;
