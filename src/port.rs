pub fn websocket_url(body: &str) -> Option<String> {
    let mut lines = body.trim().lines();
    let port = lines.next()?.trim();
    let path = lines.next()?.trim();
    if port.is_empty() || path.is_empty() {
        return None;
    }
    Some(format!("ws://127.0.0.1:{port}{path}"))
}

#[cfg(test)]
#[path = "port_tests.rs"]
mod tests;
