use std::sync::Arc;

use axum::{
    body::StreamBody,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use futures_util::{Stream, StreamExt};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

use crate::{
    utils::{redact, serve_file},
    Args,
};

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

pub async fn stream_handler(
    cli: Extension<Arc<Args>>,
    client: Extension<Arc<Client>>,
) -> StreamBody<impl Stream<Item = reqwest::Result<bytes::Bytes>>> {
    let name = &cli.name;
    let host = &cli.host;
    let port = &cli.port;

    let request = client.post(format!(
        "http://{host}:{port}/containers/{name}/attach?stdout=true&logs=true&stream=true",
        host = host,
        port = port,
        name = name
    ));

    let response = request.send().await.unwrap();
    let stream = response.bytes_stream();

    let modified_stream = stream.map(|result| match result {
        Ok(bytes) => {
            let s = String::from_utf8_lossy(&bytes);
            let redacted_s = redact(s.to_string());
            Ok(redacted_s.into_bytes().into())
        }
        Err(e) => Err(e),
    });

    StreamBody::new(modified_stream)
}

pub async fn json_handler(cli: Extension<Arc<Args>>, client: Extension<Arc<Client>>) -> Response {
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

pub async fn start_handler(cli: Extension<Arc<Args>>, client: Extension<Arc<Client>>) -> Response {
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

pub async fn stop_handler(cli: Extension<Arc<Args>>, client: Extension<Arc<Client>>) -> Response {
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

pub async fn index_handler(cli: Extension<Arc<Args>>) -> Response {
    let file = serve_file("index.html", cli).await;
    if file.is_empty() {
        return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
    }

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/html")], file).into_response()
}

pub async fn script_handler(cli: Extension<Arc<Args>>) -> Response {
    let file = serve_file("script.js", cli).await;
    if file.is_empty() {
        return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
    }

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/javascript")],
        file,
    )
        .into_response()
}
