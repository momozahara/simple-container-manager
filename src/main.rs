use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use clap::Parser;
use regex::Regex;
use reqwest::{header, Client, Method};
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncReadExt, time::interval};
use tower_http::cors::{Any, CorsLayer};

#[derive(Parser, Clone)]
#[command(disable_help_flag = true)]
struct Args {
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

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct Container {
    Name: String,
    State: State,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct State {
    Status: String,
}

async fn json_handler(cli: Extension<Arc<Args>>, client: Extension<Arc<Client>>) -> Response {
    let name = &cli.name;
    let host = &cli.host;
    let port = &cli.port;

    let request = client.get(format!(
        "http://{host}:{port}/containers/{name}/json",
        host = host,
        port = port,
        name = name
    ));

    let response = request.send().await;
    match response {
        Ok(response) => {
            if response.status().is_client_error() {
                eprintln!(
                    "ERROR: {code}: no such container",
                    code = response.status().as_u16()
                );
                return (StatusCode::NOT_FOUND).into_response();
            }

            let text = response.text().await;
            match text {
                Ok(text) => {
                    let container: Container = serde_json::from_str(&text).unwrap();
                    println!(
                        "INFO: Container {name}: {status}",
                        name = container.Name,
                        status = container.State.Status
                    );
                    (StatusCode::OK, Json(container)).into_response()
                }
                Err(err) => {
                    eprintln!("ERROR: {err}", err = err);
                    (StatusCode::INTERNAL_SERVER_ERROR).into_response()
                }
            }
        }
        Err(err) => {
            eprintln!("ERROR: {err}", err = err);
            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    }
}

async fn logs_handler(logs: Extension<Arc<Mutex<String>>>) -> Response {
    let _logs = logs.lock().unwrap();
    (StatusCode::OK, _logs.clone()).into_response()
}

async fn start_handler(cli: Extension<Arc<Args>>, client: Extension<Arc<Client>>) -> Response {
    let name = &cli.name;
    let host = &cli.host;
    let port = &cli.port;

    let request = client.post(format!(
        "http://{host}:{port}/containers/{name}/start",
        host = host,
        port = port,
        name = name
    ));

    let response = request.send().await;
    match response {
        Ok(response) => {
            if response.status().is_client_error() {
                eprintln!(
                    "ERROR: {code}: no such container",
                    code = response.status().as_u16()
                );
                return (StatusCode::NOT_FOUND).into_response();
            } else if response.status().is_redirection() {
                eprintln!(
                    "ERROR: {code}: container already started",
                    code = response.status().as_u16()
                );
                return (StatusCode::NOT_MODIFIED).into_response();
            }
            (StatusCode::NO_CONTENT).into_response()
        }
        Err(err) => {
            eprintln!("ERROR: {err}", err = err);
            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    }
}

async fn stop_handler(cli: Extension<Arc<Args>>, client: Extension<Arc<Client>>) -> Response {
    let name = &cli.name;
    let host = &cli.host;
    let port = &cli.port;

    let request = client.post(format!(
        "http://{host}:{port}/containers/{name}/stop",
        host = host,
        port = port,
        name = name
    ));

    let response = request.send().await;
    match response {
        Ok(response) => {
            if response.status().is_client_error() {
                eprintln!(
                    "ERROR: {code}: no such container",
                    code = response.status().as_u16()
                );
                return (StatusCode::NOT_FOUND).into_response();
            } else if response.status().is_redirection() {
                eprintln!(
                    "ERROR: {code}: container already stopped",
                    code = response.status().as_u16()
                );
                return (StatusCode::NOT_MODIFIED).into_response();
            }
            (StatusCode::NO_CONTENT).into_response()
        }
        Err(err) => {
            eprintln!("ERROR: {err}", err = err);
            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    }
}

async fn index_handler(cli: Extension<Arc<Args>>) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html")],
        serve_file("index.html", cli).await,
    )
        .into_response()
}

async fn script_handler(cli: Extension<Arc<Args>>) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/javascript")],
        serve_file("script.js", cli).await,
    )
        .into_response()
}

async fn serve_file(file_path: &str, cli: Extension<Arc<Args>>) -> String {
    let file_path = format!("{path}{file_path}", path = cli.path, file_path = file_path);

    if let Ok(mut file) = File::open(&file_path).await {
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await.unwrap();

        return String::from_utf8(buffer).unwrap();
    }

    String::new()
}

async fn server(cli: Arc<Args>, client: Arc<Client>, logs: Arc<Mutex<String>>) {
    let mut interval = interval(Duration::from_millis(10000));
    loop {
        let name = &cli.name;
        let host = &cli.host;
        let port = &cli.port;

        let request = client.get(format!(
            "http://{host}:{port}/containers/{name}/logs?stdout=true",
            host = host,
            port = port,
            name = name
        ));

        let response = request.send().await;
        match response {
            Ok(response) => {
                if response.status().is_client_error() {
                    eprintln!(
                        "ERROR: {code}: no such container",
                        code = response.status().as_u16()
                    );
                }

                let text = response.text().await;
                match text {
                    Ok(text) => {
                        let mut _logs = logs.lock().unwrap();
                        let l_regex = Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}:\d{1,5}\b").unwrap();
                        let redacted = l_regex.replace_all(&text, "[REDACTED]");
                        *_logs = String::from(redacted);
                        drop(_logs);
                    }
                    Err(err) => {
                        eprintln!("ERROR: {err}", err = err);
                    }
                }
            }
            Err(err) => {
                eprintln!("ERROR: {err}", err = err);
            }
        }
        interval.tick().await;
    }
}

#[tokio::main]
async fn main() {
    let cli = Arc::new(Args::parse());

    let client = Arc::new(Client::new());

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let logs = Arc::new(Mutex::new(String::new()));

    let s_cli = cli.clone();
    let s_client = client.clone();
    let s_logs = logs.clone();
    tokio::spawn(async { server(s_cli, s_client, s_logs).await });

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/script.js", get(script_handler))
        .route("/api/json", get(json_handler))
        .route("/api/logs", get(logs_handler))
        .route("/api/start", post(start_handler))
        .route("/api/stop", post(stop_handler))
        .layer(cors)
        .layer(Extension(cli))
        .layer(Extension(logs))
        .layer(Extension(client));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
