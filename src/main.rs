mod configuration;
mod models;
mod routes;

use crate::{
    configuration::LynxConfiguration,
    routes::{get_link_redirect_handler, index_handler},
};
use anyhow::{Context, Result};
use axum::{
    Router,
    extract::Request,
    http::{HeaderValue, header},
    middleware::{self as axum_middleware, Next},
    routing::get,
};
use clap::Parser;
use dotenvy::dotenv;
use notify::{Event, Watcher};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::{net::TcpListener, signal, sync::RwLock};
use tower_http::{
    catch_panic::CatchPanicLayer,
    normalize_path::NormalizePathLayer,
    trace::{self, TraceLayer},
};
use tracing::{Level, error, info, trace};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(author, version, about, long_about)]
struct Arguments {
    /// Internet socket address that the server should be ran on.
    #[arg(
        short = 'a',
        long = "address",
        env = "LYNX_ADDRESS",
        default_value = "127.0.0.1:5621"
    )]
    address: SocketAddr,

    /// Path to the configuration file. Changes will automatically trigger a reload.
    #[clap(short = 'c', long = "config", env = "LYNX_CONFIG", value_parser = validate_output_file)]
    config_file: PathBuf,
}

#[derive(Clone)]
struct AppState {
    config: Arc<RwLock<LynxConfiguration>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info")))
        .init();
    let args = Arguments::parse();

    // Load configuration file and create a watcher for changes.
    let lynx_config = Arc::new(RwLock::new(
        LynxConfiguration::from_path(&args.config_file)
            .context("your configuration file is invalid. see inner error for details")?,
    ));

    let mut watcher = {
        let loaded_config = Arc::clone(&lynx_config);
        let lynx_config_path = args.config_file.clone();
        notify::recommended_watcher(move |result: Result<Event, notify::Error>| {
            let Ok(event) = result else {
                error!(
                    "error while watching configuration file: {:?}",
                    result.err()
                );
                return;
            };
            trace!("filesystem event: {:?}", event);
            if event.kind.is_modify() || event.kind.is_create() {
                info!("change to the configuration detected - attempting reload");
                match LynxConfiguration::from_path(&lynx_config_path) {
                    Ok(config) => {
                        *loaded_config.blocking_write() = config;
                        info!("successfully reloaded configuration file");
                    }
                    Err(err) => {
                        error!(
                            "failed to reload configuration file, configuration has been left unchanged\n:Err: {:?}",
                            err
                        )
                    }
                }
            }
        })?
    };
    watcher.watch(
        &args.config_file.canonicalize()?,
        notify::RecursiveMode::NonRecursive,
    )?;

    // Prepare server.
    let state = AppState {
        config: lynx_config,
    };
    let router = Router::new()
        .route("/", get(index_handler))
        .route("/{*link_id}", get(get_link_redirect_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(NormalizePathLayer::trim_trailing_slash())
        .layer(CatchPanicLayer::new())
        .layer(axum_middleware::from_fn(
            async |req: Request, next: Next| {
                let mut res = next.run(req).await;
                let res_headers = res.headers_mut();
                res_headers.insert(
                    header::SERVER,
                    HeaderValue::from_static(env!("CARGO_PKG_NAME")),
                );
                res_headers.insert("X-Robots-Tag", HeaderValue::from_static("none"));
                res
            },
        ))
        .with_state(state);

    // Start server.
    let tcp_listener = TcpListener::bind(&args.address).await?;
    info!(
        "Starting Lynx server on http://{} with configuration from {}",
        args.address,
        args.config_file.display(),
    );
    axum::serve(tcp_listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

// https://github.com/tokio-rs/axum/blob/15917c6dbcb4a48707a20e9cfd021992a279a662/examples/graceful-shutdown/src/main.rs#L55
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

fn validate_output_file(s: &str) -> Result<PathBuf, String> {
    if s.ends_with('/') || s.ends_with('\\') {
        return Err(format!(
            "the path at '{}' must be a file, not a directory.",
            s
        ));
    }
    Ok(PathBuf::from(s))
}
