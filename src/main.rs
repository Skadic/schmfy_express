use axum::{extract::Query, response::Html, routing::get, Json, Router};
use dotenvy::dotenv;
use miette::{Context, IntoDiagnostic};
use std::net::Ipv4Addr;
use tokio::{net::TcpListener, signal};
use tracing::{error, info, info_span};

const SCHMFY_HOST: &str = "SCHMFY_HOST";
const SCHMFY_PORT: &str = "SCHMFY_PORT";

#[tokio::main]
async fn main() -> miette::Result<()> {
    tracing_subscriber::fmt().pretty().init();

    let setup_span = info_span!("setup").entered();
    if dotenv().is_err() {
        info!("could not find .env file");
    }
    let router = Router::<()>::new()
        .route("/", get(home))
        .route("/schmfy", get(schmfy).post(schmfy_post));

    let address = std::env::var(SCHMFY_HOST)
        .into_diagnostic()
        .unwrap_or_else(|_| "0.0.0.0".to_string())
        .parse::<Ipv4Addr>()
        .into_diagnostic()
        .wrap_err("could not parse HOST_ADDRESS as IPv4 address")?;

    let port = std::env::var(SCHMFY_PORT)
        .into_diagnostic()
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .into_diagnostic()
        .wrap_err("could not parse HOST_PORT as port number")?;

    let listener = match TcpListener::bind((address, port)).await {
        Ok(l) => l,
        Err(e) => {
            error!("could not bind to port {address}:{port}");
            return Err(e).into_diagnostic();
        }
    };
    drop(setup_span);

    let _schmfy_span = info_span!("schmfy").entered();
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .into_diagnostic()
}

#[derive(serde::Deserialize)]
struct Input {
    input: String,
}

async fn home() -> Html<&'static str> {
    Html(include_str!("home.html"))
}

async fn schmfy(Query(Input { input }): Query<Input>) -> String {
    schmfy::schmfy(input.as_str())
}

async fn schmfy_post(Json(Input { input }): Json<Input>) -> String {
    schmfy::schmfy(input.as_str())
}

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
