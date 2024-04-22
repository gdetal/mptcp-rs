use std::env;

use async_h1::client;
use async_std::net::TcpStream;
use http_types::{Method, Request, Url};
use mptcp::async_std::MptcpStreamExt;

#[tokio::main]
async fn main() {
    let uri = env::args()
        .nth(1)
        .unwrap_or_else(|| "http://127.0.0.1:8080".to_string());

    let url = uri.parse::<Url>().unwrap();

    let host = url.host().expect("uri has no host");
    let port = url.port().unwrap_or(8080);

    let address = format!("{}:{}", host, port);

    let client: TcpStream = TcpStream::connect_mptcp(address).await.unwrap().into();

    let req = Request::new(Method::Get, url);
    let mut res = client::connect(client, req).await.unwrap();
    println!("{}", res.body_string().await.unwrap());
}
