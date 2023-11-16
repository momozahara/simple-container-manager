use clap::{Parser, Subcommand};
use reqwest::Client;
use serde::Deserialize;

#[derive(Parser)]
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

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get container status
    Json,
    /// Start container
    Start,
    /// Stop container
    Stop,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct Container {
    Name: String,
    State: State,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct State {
    Status: String,
}

#[tokio::main]
async fn main() {
    let cli = Args::parse();

    let name = cli.name;
    let host = cli.host;
    let port = cli.port;
    let command = cli.command;

    let client = Client::new();

    match command {
        Commands::Json => {
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
                        std::process::exit(1);
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
                        }
                        Err(err) => {
                            eprintln!("ERROR: {err}", err = err);
                            std::process::exit(1);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("ERROR: {err}", err = err);
                    std::process::exit(1);
                }
            }
        }
        Commands::Start => {
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
                        std::process::exit(1);
                    }
                    if response.status().is_redirection() {
                        eprintln!(
                            "ERROR: {code}: container already started",
                            code = response.status().as_u16()
                        );
                        std::process::exit(1);
                    }
                }
                Err(err) => {
                    eprintln!("ERROR: {err}", err = err);
                    std::process::exit(1);
                }
            }
        }
        Commands::Stop => {
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
                        std::process::exit(1);
                    }
                    if response.status().is_redirection() {
                        eprintln!(
                            "ERROR: {code}: container already stopped",
                            code = response.status().as_u16()
                        );
                        std::process::exit(1);
                    }
                }
                Err(err) => {
                    eprintln!("ERROR: {err}", err = err);
                    std::process::exit(1);
                }
            }
        }
    }
}
