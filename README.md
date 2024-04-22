# mptcp-rs

A helper crate to create [Multipath TCP](https://www.mptcp.dev) (MPTCP) sockets.

## Features

The crate currently supports:

 - [x] `std::net::TcpStream` and `std::net::TcpListener`
 - [x] support Linux
 - [ ] support [tokio](https://tokio.rs)
 - [ ] support [async-std](https://async.rs)
 - [ ] support MacOS

## Usage

To create an MPTCP stream:

```rust
use mptcp::MptcpStreamExt;

let stream = TcpStream::connect_mptcp("www.google.com:443").unwrap();
```

The `connect_mptcp` method handles falling back to a TCP socket in case MPTCP
is not available on the system. Use `connect_mptcp_force` if you require to
use MPTCP.

To create an MPTCP listener:

```rust
use mptcp::MptcpListenerExt;

let listener = TcpListener::bind_mptcp("localhost:8080").unwrap();
```

Similarly to the Stream. The `bind_mptcp` method handles falling back to a
TCP socket in case MPTCP is not available on the system. Use `bind_mptcp_force`
if you require to use MPTCP.

Use the `into_socket()` to retrieve to retrieve a `TcpStream` or `TcpListener` to
be reused in existing libraries. MPTCP sockets provides the same API as TCP
sockets.

You can also check whether a `TcpStream` uses an underlying MPTCP socket using:

```rust
use mptcp::MptcpExt;

let stream : TcpStream = ...;
println!("stream uses mptcp: {}", stream.use_mptcp());
```

## License

This project is licensed under the [MIT License](LICENSE).