use std::{env, io::Write};

use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::Request;
use hyper_util::rt::TokioIo;
use mptcp::tokio::MptcpStreamExt;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let uri = env::args()
        .nth(1)
        .unwrap_or_else(|| "http://127.0.0.1:8080".to_string());
    let url = uri.parse::<hyper::Uri>().unwrap();

    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(8080);

    let address = format!("{}:{}", host, port);

    let client = TcpStream::connect_mptcp(address).await.unwrap();

    let io = TokioIo::new(client.into_socket());

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await.unwrap();
    tokio::task::spawn(async move {
        let _ = conn.await;
    });

    let req = Request::builder()
        .uri(url)
        .body(Empty::<Bytes>::new())
        .unwrap();

    let mut res = sender.send_request(req).await.unwrap();
    while let Some(next) = res.frame().await {
        let frame = next.unwrap();
        if let Some(chunk) = frame.data_ref() {
            std::io::stdout().write_all(chunk).unwrap();
        }
    }
    println!("");
}
