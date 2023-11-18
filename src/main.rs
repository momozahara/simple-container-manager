mod routes;
mod utils;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use clap::Parser;
use reqwest::{Client, Method};
use tower_http::cors::{Any, CorsLayer};

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
    let cli = Arc::new(Args::parse());

    let client = Arc::new(Client::new());

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

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
