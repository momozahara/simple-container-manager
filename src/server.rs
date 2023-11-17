use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use regex::Regex;
use reqwest::Client;
use tokio::time::interval;

use crate::Args;

pub async fn server(cli: Arc<Args>, client: Arc<Client>, logs: Arc<Mutex<String>>) {
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
