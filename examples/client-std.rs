use std::{
    env,
    io::{Read, Write},
    net::TcpStream,
};

use mptcp::MptcpStreamExt;

fn main() {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let mut client = TcpStream::connect_mptcp(addr).unwrap();

    client.write_all(b"GET / HTTP/1.1\n\r\n\r").unwrap();

    let mut buf = vec![0; 1024];
    client.read_to_end(&mut buf).unwrap();

    println!("{}", String::from_utf8_lossy(&buf));
}
