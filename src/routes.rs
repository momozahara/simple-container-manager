use std::sync::Arc;

use async_stream::try_stream;

use axum::{
    http::{Result, StatusCode},
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Response, Sse,
    },
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
) -> Sse<impl Stream<Item = Result<Event>>> {
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
    let mut stream = response.bytes_stream();

    tracing::info!("/api/stream");

    Sse::new(try_stream! {
        loop {
            match stream.next().await.unwrap() {
                Ok(bytes) => {
                    let s = String::from_utf8_lossy(&bytes);
                    let redacted_s = redact(s.to_string());
                    let result = redacted_s.replace("\n", "<newline>").replace("\r", "");
                    yield Event::default().data(result)
                }
                Err(e) => {
                    tracing::error!("ERROR: {err}", err = e);
                    break;
                }
            }
        }
    })
    .keep_alive(KeepAlive::default())
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
                tracing::error!(
                    "ERROR: {code}: no such container",
                    code = response.status().as_u16()
                );
                return (StatusCode::NOT_FOUND).into_response();
            }

            let text = response.text().await;
            match text {
                Ok(text) => {
                    let container: Container = serde_json::from_str(&text).unwrap();
                    tracing::info!("/api/json");
                    (StatusCode::OK, Json(container)).into_response()
                }
                Err(err) => {
                    tracing::error!("ERROR: {err}", err = err);
                    (StatusCode::INTERNAL_SERVER_ERROR).into_response()
                }
            }
        }
        Err(err) => {
            tracing::error!("ERROR: {err}", err = err);
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
                tracing::error!(
                    "ERROR: {code}: no such container",
                    code = response.status().as_u16()
                );
                return (StatusCode::NOT_FOUND).into_response();
            } else if response.status().is_redirection() {
                tracing::error!(
                    "ERROR: {code}: container already started",
                    code = response.status().as_u16()
                );
                return (StatusCode::NOT_MODIFIED).into_response();
            }
            tracing::info!("/api/start");
            (StatusCode::NO_CONTENT).into_response()
        }
        Err(err) => {
            tracing::error!("ERROR: {err}", err = err);
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
                tracing::error!(
                    "ERROR: {code}: no such container",
                    code = response.status().as_u16()
                );
                return (StatusCode::NOT_FOUND).into_response();
            } else if response.status().is_redirection() {
                tracing::error!(
                    "ERROR: {code}: container already stopped",
                    code = response.status().as_u16()
                );
                return (StatusCode::NOT_MODIFIED).into_response();
            }
            tracing::info!("/api/stop");
            (StatusCode::NO_CONTENT).into_response()
        }
        Err(err) => {
            tracing::error!("ERROR: {err}", err = err);
            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    }
}

pub async fn index_handler(cli: Extension<Arc<Args>>) -> Response {
    let file = serve_file("index.html", cli).await;
    if file.is_empty() {
        return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
    }

    tracing::info!("/");

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/html")], file).into_response()
}

pub async fn script_handler(cli: Extension<Arc<Args>>) -> Response {
    let file = serve_file("script.js", cli).await;
    if file.is_empty() {
        return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
    }

    tracing::info!("/script.js");

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/javascript")],
        file,
    )
        .into_response()
}
