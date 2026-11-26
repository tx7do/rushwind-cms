//! The app (C-side) BFF service assembly entry: loads the embedded
//! config defaults (env overrides win), builds the HS256 engine (the
//! secret serves both verification and minting), connects Postgres and
//! Redis, and hands the transports (the REST edge :6700 and the SSE
//! notification server :6701) to the config-driven lifecycle assembler
//! (the embedded server document, one `App`, one shutdown path).

use std::sync::Arc;

use app_api::config::Config;
use app_api::server::{rest, sse};
use app_api::state::AppState;
use rushwind_bootstrap::Bootstrap;
use rushwind_transport::StopSignal;

/// The server assembly document, compiled into the binary.
const SERVER_YAML: &str = include_str!("../assets/server.yaml");

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
    let verifier = rushwind_authn_jwt::JwtAuthenticator::new(
        rushwind_authn_jwt::JwtOptions::new()
            .with_algorithm("HS256")
            .map_err(|e| format!("jwt algorithm: {e:?}"))?
            .with_key(cfg.jwt_key.as_bytes()),
    );
    let authenticator: Arc<dyn rushwind_authn::Authenticator> = Arc::new(verifier);

    let state = Arc::new(AppState::connect(cfg, authenticator).await?);

    let assembler = Bootstrap::from_yaml_str(SERVER_YAML)?
        .route_pack("app-surface", rest::pack(Arc::clone(&state)))
        .server_factory("app-sse", sse::factory(Arc::clone(&state)));

    let booted = assembler.build().await?;
    // The lifecycle owns the OS-signal shutdown path internally; the
    // external signal here stays unfired.
    let external = StopSignal::new();
    booted.run_logged(external).await?;
    Ok(())
}
