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
    // 全流程感知底座，先于一切装配：框架各层只发 tracing 事件——
    // 每请求 access 行（rushwind.http：method/path/request_id/status/
    // latency）、panic 汇报（rushwind.panic）、生命周期终态
    // （rushwind.bootstrap）——订阅者是应用自己的事，不装即全盲。
    // RUST_LOG 优先，缺省 info；写线非阻塞（fmt 默认同步写 stdout，
    // 部署接了不排水的管道会把 worker 逐个 park 死成"卡阻"），ANSI
    // 关闭保证日志文件可 grep。
    let (log_writer, log_guard) = tracing_appender::non_blocking(std::io::stdout());
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(log_writer)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    // 守卫必须活到进程结束（main 即进程生命周期）。
    let _log_guard = log_guard;

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
