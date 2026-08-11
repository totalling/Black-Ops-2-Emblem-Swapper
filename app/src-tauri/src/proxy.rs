use std::sync::Arc;

use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex as TokioMutex;

use crate::emblem::render::ShapeCache;
use crate::paths::{Paths, PROXY_HOST, PROXY_PORT, TARGET_HOST_SUBSTR};
use crate::state::Mode;

fn log(msg: &str) {
    let now = chrono::Local::now().format("%H:%M:%S");
    println!("{now} {msg}");
}

fn find_subslice(data: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || data.len() < needle.len() {
        return None;
    }
    data.windows(needle.len()).position(|w| w == needle)
}

fn find_header_value<'a>(headers: &'a [u8], name_lower: &[u8]) -> Option<&'a [u8]> {
    for raw_line in headers.split(|&b| b == b'\n') {
        let line = raw_line.strip_suffix(b"\r").unwrap_or(raw_line);
        if let Some(colon) = line.iter().position(|&b| b == b':') {
            let key = &line[..colon];
            if key.to_ascii_lowercase() == name_lower {
                return Some(line[colon + 1..].trim_ascii());
            }
        }
    }
    None
}

fn parse_userhash_slot(path: &str) -> Option<(String, u32)> {
    let bytes = path.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'u' {
            let start = i + 1;
            let mut j = start + 1;
            while j < bytes.len() && bytes[j].is_ascii_hexdigit() {
                j += 1;
            }
            if j > start + 1 {
                let rest = &path[j..];
                if let Some(rest2) = rest.strip_prefix(".slot_") {
                    let digits_end = rest2
                        .find(|c: char| !c.is_ascii_digit())
                        .unwrap_or(rest2.len());
                    if digits_end > 0 {
                        if let Ok(slot) = rest2[..digits_end].parse::<u32>() {
                            return Some((path[start..j].to_string(), slot));
                        }
                    }
                }
            }
        }
        i += 1;
    }
    None
}

fn normalize_headers_for_body(headers: &[u8], body_len: usize) -> Vec<u8> {
    let text = String::from_utf8_lossy(headers);
    let mut out_lines: Vec<String> = Vec::new();
    let mut seen_cl = false;
    for line in text.split("\r\n") {
        if line.is_empty() {
            continue;
        }
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("transfer-encoding:") {
            continue;
        }
        if lower.starts_with("content-length:") {
            out_lines.push(format!("Content-Length: {body_len}"));
            seen_cl = true;
            continue;
        }
        out_lines.push(line.to_string());
    }
    if !seen_cl {
        out_lines.push(format!("Content-Length: {body_len}"));
    }
    let mut result = out_lines.join("\r\n");
    result.push_str("\r\n\r\n");
    result.into_bytes()
}

async fn recv_full_response(stream: &mut TcpStream) -> std::io::Result<(Vec<u8>, Vec<u8>)> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];

    let header_end = loop {
        if let Some(pos) = find_subslice(&buf, b"\r\n\r\n") {
            break pos + 4;
        }
        let n = stream.read(&mut chunk).await?;
        if n == 0 {
            return Ok((buf, Vec::new()));
        }
        buf.extend_from_slice(&chunk[..n]);
    };

    let headers_raw = buf[..header_end].to_vec();
    let leftover = buf[header_end..].to_vec();

    let content_length = find_header_value(&headers_raw, b"content-length")
        .and_then(|v| std::str::from_utf8(v).ok())
        .and_then(|v| v.trim().parse::<usize>().ok());
    let is_chunked = find_header_value(&headers_raw, b"transfer-encoding")
        .map(|v| {
            String::from_utf8_lossy(v)
                .to_ascii_lowercase()
                .contains("chunked")
        })
        .unwrap_or(false);

    if is_chunked {
        let body = read_chunked_body(stream, leftover).await?;
        let headers_norm = normalize_headers_for_body(&headers_raw, body.len());
        Ok((headers_norm, body))
    } else if let Some(cl) = content_length {
        let mut body = leftover;
        while body.len() < cl {
            let n = stream.read(&mut chunk).await?;
            if n == 0 {
                break;
            }
            body.extend_from_slice(&chunk[..n]);
        }
        body.truncate(cl.min(body.len()));
        Ok((headers_raw, body))
    } else {
        let mut body = leftover;
        loop {
            match tokio::time::timeout(std::time::Duration::from_millis(500), stream.read(&mut chunk)).await {
                Ok(Ok(0)) => break,
                Ok(Ok(n)) => body.extend_from_slice(&chunk[..n]),
                Ok(Err(e)) => return Err(e),
                Err(_) => break,
            }
        }
        Ok((headers_raw, body))
    }
}

async fn read_chunked_body(stream: &mut TcpStream, initial: Vec<u8>) -> std::io::Result<Vec<u8>> {
    let mut raw = initial;
    let mut decoded = Vec::new();
    let mut chunk = [0u8; 4096];

    loop {
        while find_subslice(&raw, b"\r\n").is_none() {
            let n = stream.read(&mut chunk).await?;
            if n == 0 {
                return Ok(decoded);
            }
            raw.extend_from_slice(&chunk[..n]);
        }
        let line_end = find_subslice(&raw, b"\r\n").unwrap();
        let size_line = String::from_utf8_lossy(&raw[..line_end]);
        let size_str = size_line.split(';').next().unwrap_or("0").trim();
        let size = usize::from_str_radix(size_str, 16).unwrap_or(0);

        if size == 0 {
            let mut pos = line_end + 2;
            loop {
                while find_subslice(&raw[pos.min(raw.len())..], b"\r\n").is_none() {
                    let n = stream.read(&mut chunk).await?;
                    if n == 0 {
                        return Ok(decoded);
                    }
                    raw.extend_from_slice(&chunk[..n]);
                }
                let rel = find_subslice(&raw[pos..], b"\r\n").unwrap();
                if rel == 0 {
                    break;
                }
                pos += rel + 2;
            }
            break;
        }

        let needed = line_end + 2 + size + 2;
        while raw.len() < needed {
            let n = stream.read(&mut chunk).await?;
            if n == 0 {
                break;
            }
            raw.extend_from_slice(&chunk[..n]);
        }

        let data_start = line_end + 2;
        let data_end = (data_start + size).min(raw.len());
        decoded.extend_from_slice(&raw[data_start..data_end]);

        if raw.len() < needed {
            break;
        }
        raw = raw[needed..].to_vec();
    }
    Ok(decoded)
}

async fn fetch_real(host: &str, port: u16, req: &[u8]) -> std::io::Result<(Vec<u8>, Vec<u8>)> {
    let mut upstream = TcpStream::connect((host, port)).await?;
    upstream.write_all(req).await?;
    recv_full_response(&mut upstream).await
}

#[allow(clippy::too_many_arguments)]
async fn handle_target_request(
    client: &mut TcpStream,
    req: &[u8],
    host: &str,
    port: u16,
    path: &str,
    paths: &Paths,
    capture_lock: &TokioMutex<()>,
    app: &AppHandle,
) -> std::io::Result<()> {
    let mode = crate::state::read_state(paths);
    let parsed = parse_userhash_slot(path);

    if mode == Mode::Inject {
        if let Some((_, slot_num)) = &parsed {
            match crate::broadcast::read_selected_data(paths) {
                Ok(Some(selected)) => {
                    client.write_all(&selected).await?;
                    let has_conditional = find_header_value(req, b"if-none-match").is_some()
                        || find_header_value(req, b"if-modified-since").is_some()
                        || find_header_value(req, b"range").is_some();
                    if has_conditional {
                        log(&format!(
                            "  Show: sent selected emblem for slot_{slot_num}, but the console sent a cache-check request - it may keep using its cached copy instead"
                        ));
                    } else {
                        log(&format!(
                            "  Show: sent selected emblem for slot_{slot_num} ({} bytes)",
                            selected.len()
                        ));
                    }
                    return Ok(());
                }
                Ok(None) => {
                    log("  Show mode is on but no emblem is selected - passing real data through");
                }
                Err(e) => log(&format!("  error reading selected emblem: {e}")),
            }
        }
    }

    let (headers, body) = fetch_real(host, port, req).await?;

    if mode == Mode::Capture {
        if let Some((userhash, slot_num)) = &parsed {
            let _guard = capture_lock.lock().await;
            match crate::storage::get_or_create_capture_group(paths, userhash) {
                Ok(group) => {
                    let mut combined = headers.clone();
                    combined.extend_from_slice(&body);
                    let path = crate::storage::slot_path(paths, &group, *slot_num);
                    if let Err(e) = std::fs::write(&path, &combined) {
                        log(&format!("  error saving capture: {e}"));
                    } else {
                        crate::storage::invalidate_thumb_cache(paths, &group, *slot_num);
                        log(&format!(
                            "  Captured: group {group} slot_{slot_num} ({} bytes)",
                            body.len()
                        ));
                        let _ = app.emit(
                            "emblem-captured",
                            serde_json::json!({ "group": group, "slot": slot_num }),
                        );
                    }
                }
                Err(e) => log(&format!("  error creating capture group: {e}")),
            }
        }
    }

    let mut out = headers;
    out.extend_from_slice(&body);
    client.write_all(&out).await?;
    Ok(())
}

async fn pipe_bidirectional(client: &mut TcpStream, upstream: &mut TcpStream) {
    let _ = tokio::io::copy_bidirectional(client, upstream).await;
}

async fn handle_connection(
    mut client: TcpStream,
    paths: Arc<Paths>,
    capture_lock: Arc<TokioMutex<()>>,
    app: AppHandle,
) -> std::io::Result<()> {
    let mut req = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        if find_subslice(&req, b"\r\n\r\n").is_some() {
            break;
        }
        let n = client.read(&mut chunk).await?;
        if n == 0 {
            return Ok(());
        }
        req.extend_from_slice(&chunk[..n]);
    }

    let line_end = find_subslice(&req, b"\r\n").unwrap_or(req.len());
    let line = String::from_utf8_lossy(&req[..line_end]).to_string();
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }
    let method = parts[0].to_uppercase();

    if method == "CONNECT" {
        let target = parts[1];
        let (host, port_str) = target.split_once(':').unwrap_or((target, ""));
        if host.to_lowercase().contains("demonware") && !host.contains(TARGET_HOST_SUBSTR) {
            log(&format!(
                "  note: HTTPS traffic to {host} (not the expected emblem host '{TARGET_HOST_SUBSTR}') - tunneled untouched"
            ));
        }
        let port: u16 = port_str.parse().unwrap_or(443);
        let mut upstream = TcpStream::connect((host, port)).await?;
        client
            .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
            .await?;
        pipe_bidirectional(&mut client, &mut upstream).await;
        return Ok(());
    }

    let url = parts[1];
    let rest = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let (host_port, path) = match rest.split_once('/') {
        Some((hp, p)) => (hp, format!("/{p}")),
        None => (rest, "/".to_string()),
    };
    let (host, port) = match host_port.split_once(':') {
        Some((h, p)) => (h.to_string(), p.parse().unwrap_or(80)),
        None => (host_port.to_string(), 80u16),
    };

    let rest_of_req = if line_end + 2 <= req.len() {
        req[line_end + 2..].to_vec()
    } else {
        Vec::new()
    };
    let new_line = format!("{method} {path} HTTP/1.1\r\n");
    let mut req_rewritten = new_line.into_bytes();
    req_rewritten.extend_from_slice(&rest_of_req);

    if host.contains(TARGET_HOST_SUBSTR) {
        handle_target_request(
            &mut client,
            &req_rewritten,
            &host,
            port,
            &path,
            &paths,
            &capture_lock,
            &app,
        )
        .await?;
    } else {
        if host.to_lowercase().contains("demonware") {
            log(&format!(
                "  note: HTTP request to {host}{path} (not the expected emblem host '{TARGET_HOST_SUBSTR}') - passed through untouched"
            ));
        }
        let mut upstream = TcpStream::connect((host.as_str(), port)).await?;
        upstream.write_all(&req_rewritten).await?;
        pipe_bidirectional(&mut client, &mut upstream).await;
    }
    Ok(())
}

pub async fn serve(
    paths: Arc<Paths>,
    capture_lock: Arc<TokioMutex<()>>,
    _shapes_cache: Arc<ShapeCache>,
    app: AppHandle,
) {
    if let Err(e) = crate::state::ensure_state_file(&paths) {
        log(&format!("  error ensuring state file: {e}"));
    }

    let listener = match TcpListener::bind((PROXY_HOST, PROXY_PORT)).await {
        Ok(l) => l,
        Err(e) => {
            log(&format!(
                "  fatal: could not bind proxy on {PROXY_HOST}:{PROXY_PORT}: {e}"
            ));
            return;
        }
    };

    log(&format!(
        "Proxy listening on {}:{}  mode={}",
        PROXY_HOST,
        PROXY_PORT,
        crate::state::read_state(&paths).as_internal_str()
    ));

    loop {
        let (client, _) = match listener.accept().await {
            Ok(pair) => pair,
            Err(e) => {
                log(&format!("  accept error: {e}"));
                continue;
            }
        };
        let paths = paths.clone();
        let capture_lock = capture_lock.clone();
        let app = app.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(client, paths, capture_lock, app).await {
                log(&format!("  error: {e}"));
            }
        });
    }
}
