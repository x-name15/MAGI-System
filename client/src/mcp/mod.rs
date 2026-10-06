//! # Model Context Protocol (MCP) Server
//!
//! Exposes the MAGI System as an MCP stdio server allowing AI coding assistants
//! (e.g. Antigravity, Claude Desktop) to invoke the Evangelion Trinity consensus engine.

pub mod handler;
pub mod protocol;

use crate::config::MagiConfig;
use crate::error::MagiError;
use handler::McpHandler;
use protocol::JsonRpcRequest;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Starts the MAGI MCP server over stdio.
pub async fn run_stdio_server(
    config: MagiConfig,
    custom_skill: Option<String>,
    force_mock: bool,
) -> Result<(), MagiError> {
    let handler = McpHandler::new(config, custom_skill, force_mock);
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();
    let mut stdout = tokio::io::stdout();

    eprintln!("[MAGI MCP] Server initialized. Listening on stdio...");

    while let Ok(Some(line)) = reader.next_line().await {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[MAGI MCP] Failed to parse JSON-RPC request: {}", e);
                let err_resp = protocol::JsonRpcResponse::error(
                    serde_json::Value::Null,
                    -32700,
                    format!("Parse error: {}", e),
                );
                if let Ok(serialized) = serde_json::to_string(&err_resp) {
                    let _ = stdout.write_all(serialized.as_bytes()).await;
                    let _ = stdout.write_all(b"\n").await;
                    let _ = stdout.flush().await;
                }
                continue;
            }
        };

        if let Some(resp) = handler.handle_request(req).await {
            let serialized = serde_json::to_string(&resp).map_err(|e| {
                MagiError::Io(std::io::Error::other(format!("Serialization error: {}", e)))
            })?;
            stdout
                .write_all(serialized.as_bytes())
                .await
                .map_err(MagiError::Io)?;
            stdout.write_all(b"\n").await.map_err(MagiError::Io)?;
            stdout.flush().await.map_err(MagiError::Io)?;
        }
    }

    eprintln!("[MAGI MCP] Stdio stream closed. Shutting down.");
    Ok(())
}
