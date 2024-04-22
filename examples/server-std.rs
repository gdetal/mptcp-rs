use std::{
    env,
    io::Write,
    net::{TcpListener, TcpStream},
    thread,
};

use mptcp::{MptcpExt, MptcpListenerExt, MptcpStatus};

fn handle_client(mut stream: TcpStream) {
    println!("handle_client()");

    let text = if let MptcpStatus::Mptcp(..) = stream.mptcp_status() {
        "You are using MPTCP\n\r"
    } else {
        "You are not using MPTCP\n\r"
    };

    println!("write_client() -> {}", text);

    stream.write_all(b"HTTP/1.1 200 OK\n\r\n\r").unwrap();
    stream.write_all(text.as_bytes()).unwrap();
    stream.shutdown(std::net::Shutdown::Write).unwrap();
}

fn main() {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let server = TcpListener::bind_mptcp_force(addr).unwrap();

    for stream in server.incoming() {
        println!("new accept()");

        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(_) => {
                println!("Error");
            }
        }
    }
}
