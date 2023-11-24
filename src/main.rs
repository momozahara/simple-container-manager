mod routes;
mod utils;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use clap::Parser;
use reqwest::{Client, Method};
use time::{macros::format_description, UtcOffset};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::fmt::time::OffsetTime;

#[derive(Parser, Clone)]
#[command(disable_help_flag = true)]
pub struct Args {
    /// Print this message or the help of the given subcommand(s)
    #[arg(long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,

    /// Container name or id
    #[arg(short, long)]
    name: String,

    #[arg(short, long, default_value = "127.0.0.1")]
    host: String,

    #[arg(short, long, default_value = "2375")]
    port: String,

    /// Path to static
    #[arg(long, default_value = "static/")]
    path: String,
}

#[tokio::main]
async fn main() {
    let offset = UtcOffset::from_hms(7, 0, 0).expect("could not get offset Utc+7");
    let timer = OffsetTime::new(
        offset,
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]"),
    );
    tracing_subscriber::fmt()
        .with_target(false)
        .with_timer(timer)
        .init();

    let cli = Arc::new(Args::parse());

    let client = Arc::new(Client::new());

    tracing::info!(
        "testing remote connection {host}:{port}",
        host = &cli.host,
        port = &cli.port
    );
    let request = client.get(format!(
        "http://{host}:{port}/_ping",
        host = &cli.host,
        port = &cli.port
    ));
    match request
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        Ok(response) => {
            if !response.status().is_success() {
                tracing::error!("could not connect to server");
                std::process::exit(1);
            }
        }
        Err(_) => {
            tracing::error!("could not connect to server");
            std::process::exit(1);
        }
    }

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let app = Router::new()
        .route("/", get(routes::index_handler))
        .route("/script.js", get(routes::script_handler))
        .route("/api/json", get(routes::json_handler))
        .route("/api/start", post(routes::start_handler))
        .route("/api/stop", post(routes::stop_handler))
        .route("/api/stream", post(routes::stream_handler))
        .layer(cors)
        .layer(Extension(cli))
        .layer(Extension(client));

    let addr = &"0.0.0.0:3000".parse().unwrap();
    let bind_response = axum::Server::try_bind(addr);

    match bind_response {
        Ok(server) => {
            tracing::info!("listening on {address}", address = addr);
            server.serve(app.into_make_service()).await.unwrap();
        }
        Err(err) => {
            tracing::error!("{error}", error = err);
        }
    }
}
