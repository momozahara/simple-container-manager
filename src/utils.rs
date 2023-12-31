use std::sync::Arc;

use axum::Extension;
use regex::Regex;
use tokio::{fs::File, io::AsyncReadExt};

use crate::Args;

pub async fn serve_file(file_path: &str, cli: Extension<Arc<Args>>) -> String {
    let file_path = format!("{path}{file_path}", path = cli.path, file_path = file_path);

    if let Ok(mut file) = File::open(&file_path).await {
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await.unwrap();

        return String::from_utf8(buffer).unwrap();
    }

    String::new()
}

pub fn redact(text: String) -> String {
    let l_regex = Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}:\d{1,5}\b").unwrap();
    String::from(l_regex.replace_all(&text, "[REDACTED]"))
}
