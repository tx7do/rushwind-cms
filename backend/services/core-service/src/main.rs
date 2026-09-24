//! The core domain service entry: loads the embedded config defaults
//! (env overrides win), connects Postgres + Redis exclusively (the
//! BFFs never touch the database), runs the fresh-database bootstrap
//! (golden DDL + system seed + demo seed), and serves the domain gRPC
//! face (:6602 by default).

use core_service::config::Config;
use core_service::server;
use core_service::state::AppState;

use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Config::load()?;
    let state = Arc::new(AppState::connect(cfg.clone()).await?);
    store::bootstrap::run(&state.db).await?;

    let addr: std::net::SocketAddr = cfg
        .grpc_addr
        .parse()
        .map_err(|e| format!("grpc addr {}: {e}", cfg.grpc_addr))?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("[core-service] serving gRPC on {}", cfg.grpc_addr);
    tonic::transport::Server::builder()
        .add_routes(server::registry(state))
        .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
        .await?;
    Ok(())
}
