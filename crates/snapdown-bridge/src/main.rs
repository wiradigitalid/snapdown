use std::io::{self, BufRead, Write};

use snapdown_bridge::client::LocalApiClient;
use snapdown_bridge::mcp::McpHandler;

fn main() {
    let port: u16 = std::env::var("SNAPDOWN_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(18400);

    let client = LocalApiClient::new(port);
    let mut handler = McpHandler::new(client);

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        match line {
            Ok(content) if !content.trim().is_empty() => {
                if let Some(resp) = handler.handle_message(content.trim()) {
                    let _ = writeln!(stdout, "{resp}");
                    let _ = stdout.flush();
                }
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
}
