use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::web_dashboard;

pub struct ServerOptions {
    pub addr: String,
    pub output_dir: PathBuf,
    pub refresh_ms: u64,
    pub once: bool,
}

pub fn serve(project_root: &Path, options: ServerOptions) -> Result<()> {
    let listener = TcpListener::bind(&options.addr)
        .with_context(|| format!("bind local dashboard server at {}", options.addr))?;
    println!("agenthub serve http://{}", listener.local_addr()?);
    println!("dashboard data refresh: {}ms", options.refresh_ms);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(error) = handle_stream(project_root, &options, stream) {
                    eprintln!("agenthub serve request error: {error:#}");
                }
            }
            Err(error) => eprintln!("agenthub serve accept error: {error}"),
        }
        if options.once {
            break;
        }
    }
    Ok(())
}

fn handle_stream(
    project_root: &Path,
    options: &ServerOptions,
    mut stream: TcpStream,
) -> Result<()> {
    let path = request_path(&stream)?;
    if path == "/health" {
        return respond_text(&mut stream, "ok\n", "text/plain; charset=utf-8");
    }
    web_dashboard::write_dashboard(project_root, &options.output_dir)?;
    let file = file_for_path(&options.output_dir, &path);
    if !file.starts_with(&options.output_dir) || !file.exists() || !file.is_file() {
        return respond_text(&mut stream, "not found\n", "text/plain; charset=utf-8");
    }
    let mut body = fs::read(&file).with_context(|| format!("read {}", file.display()))?;
    if file.file_name().and_then(|value| value.to_str()) == Some("index.html") {
        body = inject_live_options(&String::from_utf8_lossy(&body), options.refresh_ms).into();
    }
    respond_bytes(&mut stream, &body, content_type(&file))
}

fn request_path(stream: &TcpStream) -> Result<String> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut first = String::new();
    reader.read_line(&mut first)?;
    let path = first
        .split_whitespace()
        .nth(1)
        .unwrap_or("/")
        .split('?')
        .next()
        .unwrap_or("/");
    Ok(percent_decode(path))
}

fn file_for_path(root: &Path, request_path: &str) -> PathBuf {
    let relative = request_path.trim_start_matches('/');
    let relative = if relative.is_empty() {
        "index.html"
    } else {
        relative
    };
    let sanitized = relative
        .split('/')
        .filter(|part| !part.is_empty() && *part != "." && *part != "..")
        .collect::<PathBuf>();
    root.join(sanitized)
}

fn respond_text(stream: &mut TcpStream, body: &str, content_type: &str) -> Result<()> {
    respond_bytes(stream, body.as_bytes(), content_type)
}

fn respond_bytes(stream: &mut TcpStream, body: &[u8], content_type: &str) -> Result<()> {
    write!(
        stream,
        "HTTP/1.1 200 OK\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\ncache-control: no-store\r\nconnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)?;
    Ok(())
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|value| value.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn inject_live_options(html: &str, refresh_ms: u64) -> String {
    let script = format!(
        "<script>window.AGENTHUB_LIVE=true;window.AGENTHUB_REFRESH_MS={refresh_ms};</script>"
    );
    html.replace(
        "<script src=\"dashboard.js\"></script>",
        &format!("{script}\n    <script src=\"dashboard.js\"></script>"),
    )
}

fn percent_decode(value: &str) -> String {
    let mut out = String::new();
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(decoded) = u8::from_str_radix(&value[index + 1..index + 3], 16) {
                out.push(decoded as char);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index] as char);
        index += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_paths_are_sanitized_under_root() {
        let root = PathBuf::from("/tmp/dashboard");
        assert_eq!(file_for_path(&root, "/"), root.join("index.html"));
        assert_eq!(
            file_for_path(&root, "/../secret.txt"),
            root.join("secret.txt")
        );
    }

    #[test]
    fn live_options_are_injected_before_dashboard_script() {
        let html = r#"<script src="dashboard.js"></script>"#;
        let output = inject_live_options(html, 1500);
        assert!(output.contains("AGENTHUB_LIVE=true"));
        assert!(output.contains("AGENTHUB_REFRESH_MS=1500"));
    }
}
