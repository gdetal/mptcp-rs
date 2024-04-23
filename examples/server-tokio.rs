use std::env;

use axum::{
    extract::{connect_info::Connected, ConnectInfo},
    http::Request,
    routing::get,
    Router,
};
use hyper::body::Incoming;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server,
};
use mptcp::{tokio::MptcpListenerExt, MptcpExt, MptcpStatus};
use tokio::net::{TcpListener, TcpStream};
use tower_service::Service;

#[derive(Debug, Clone)]
struct MptcpInfo {
    use_mptcp: bool,
}

impl Connected<&TcpStream> for MptcpInfo {
    fn connect_info(sock: &TcpStream) -> Self {
        let use_mptcp = matches!(
            sock.mptcp_status(),
            MptcpStatus::Mptcp {
                has_fallback: false,
            }
        );

        Self { use_mptcp }
    }
}

async fn amiusingmptcp(ConnectInfo(info): ConnectInfo<MptcpInfo>) -> &'static str {
    if info.use_mptcp {
        "You are using MPTCP"
    } else {
        "You are not using MPTCP"
    }
}

#[tokio::main]
async fn main() {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let app = Router::new().route("/", get(amiusingmptcp));

    let listener = TcpListener::bind_mptcp_force(addr).await.unwrap();

    let handle = tokio::spawn(async move {
        let mut make_service = app.into_make_service_with_connect_info::<MptcpInfo>();
        loop {
            let (socket, _remote_addr) = listener.accept().await.unwrap();
            let tower_service = make_service.call(&socket).await.unwrap();
            tokio::spawn(async move {
                let socket = TokioIo::new(socket);
                let hyper_service =
                    hyper::service::service_fn(move |request: Request<Incoming>| {
                        tower_service.clone().call(request)
                    });
                if let Err(err) = server::conn::auto::Builder::new(TokioExecutor::new())
                    .serve_connection(socket, hyper_service)
                    .await
                {
                    println!("Failed to serve connection: {}", err);
                }
            });
        }
    });
    let _ = handle.await;
}
