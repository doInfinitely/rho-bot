//! WebSocket client for communicating with the rho-bot server.

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct WsClient {
    write: futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    read: futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
}

impl WsClient {
    /// Connect to the server and authenticate with the given JWT.
    pub async fn connect(url: &str, token: &str) -> Result<Self, String> {
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| format!("WebSocket connect failed: {}", e))?;

        let (mut write, read) = ws_stream.split();

        // Send auth message
        let auth = serde_json::json!({"token": token});
        write
            .send(Message::Text(auth.to_string()))
            .await
            .map_err(|e| format!("Auth send failed: {}", e))?;

        Ok(Self { write, read })
    }

    /// Send a context payload (JSON) and wait for the server's action response.
    pub async fn send_context(&mut self, context: &Value) -> Result<Value, String> {
        self.write
            .send(Message::Text(context.to_string()))
            .await
            .map_err(|e| format!("Send failed: {}", e))?;

        // Wait for response, skipping Ping/Pong frames
        loop {
            match self.read.next().await {
                Some(Ok(Message::Text(text))) => {
                    return serde_json::from_str(&text)
                        .map_err(|e| format!("Parse error: {}", e));
                }
                Some(Ok(Message::Close(frame))) => {
                    let detail = frame
                        .map(|f| format!("code={}, reason={}", f.code, f.reason))
                        .unwrap_or_else(|| "no details".into());
                    return Err(format!("Server closed connection ({})", detail));
                }
                Some(Ok(Message::Ping(data))) => {
                    let _ = self.write.send(Message::Pong(data)).await;
                    continue;
                }
                Some(Ok(Message::Pong(_))) => continue,
                Some(Err(e)) => return Err(format!("Read error: {}", e)),
                None => return Err("Connection closed unexpectedly".into()),
                _ => continue,
            }
        }
    }

    /// Send a training pair payload and wait for the server's acknowledgment.
    pub async fn send_training_pair(&mut self, payload: &Value) -> Result<(), String> {
        self.write
            .send(Message::Text(payload.to_string()))
            .await
            .map_err(|e| format!("Send failed: {}", e))?;

        // Wait for ack, skipping Ping/Pong frames
        loop {
            match self.read.next().await {
                Some(Ok(Message::Text(_))) => return Ok(()),
                Some(Ok(Message::Close(_))) => return Err("Server closed connection".into()),
                Some(Ok(Message::Ping(data))) => {
                    let _ = self.write.send(Message::Pong(data)).await;
                    continue;
                }
                Some(Ok(Message::Pong(_))) => continue,
                Some(Err(e)) => return Err(format!("Read error: {}", e)),
                None => return Err("Connection closed".into()),
                _ => continue,
            }
        }
    }

    /// Connect without authentication (marionette server doesn't use JWT).
    pub async fn connect_no_auth(url: &str) -> Result<Self, String> {
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| format!("WebSocket connect failed: {}", e))?;

        let (write, read) = ws_stream.split();
        Ok(Self { write, read })
    }

    /// Send a start message to initiate a marionette remote agent session.
    pub async fn send_start(&mut self, task: &str, settings: &Value) -> Result<(), String> {
        let msg = serde_json::json!({
            "type": "start",
            "task": task,
            "settings": settings,
        });
        self.write
            .send(Message::Text(msg.to_string()))
            .await
            .map_err(|e| format!("Send start failed: {}", e))?;

        // Wait for 'ready' response
        loop {
            match self.read.next().await {
                Some(Ok(Message::Text(text))) => {
                    let resp: Value = serde_json::from_str(&text)
                        .map_err(|e| format!("Parse ready response: {}", e))?;
                    if resp.get("type").and_then(|t| t.as_str()) == Some("ready") {
                        return Ok(());
                    }
                    // If it's an error, return it
                    if resp.get("type").and_then(|t| t.as_str()) == Some("error") {
                        let msg = resp.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error");
                        return Err(format!("Server error: {}", msg));
                    }
                    // Otherwise keep waiting
                    continue;
                }
                Some(Ok(Message::Ping(data))) => {
                    let _ = self.write.send(Message::Pong(data)).await;
                    continue;
                }
                Some(Ok(Message::Pong(_))) => continue,
                Some(Ok(Message::Close(frame))) => {
                    let detail = frame
                        .map(|f| format!("code={}, reason={}", f.code, f.reason))
                        .unwrap_or_else(|| "no details".into());
                    return Err(format!("Server closed connection ({})", detail));
                }
                Some(Err(e)) => return Err(format!("Read error: {}", e)),
                None => return Err("Connection closed before ready".into()),
                _ => continue,
            }
        }
    }

    /// Send a context payload to the marionette agent and receive the response.
    ///
    /// The response may be: actions, done, step, ask_user, or error.
    pub async fn send_marionette_context(&mut self, context: &Value) -> Result<Value, String> {
        let msg = serde_json::json!({
            "type": "context",
            "session_id": context.get("session_id"),
            "timestamp": context.get("timestamp"),
            "screenshot_b64": context.get("screenshot_b64"),
            "accessibility_tree": context.get("accessibility_tree"),
            "recent_events": context.get("recent_events"),
            "active_app": context.get("active_app"),
            "window_bounds": context.get("window_bounds"),
        });
        self.write
            .send(Message::Text(msg.to_string()))
            .await
            .map_err(|e| format!("Send context failed: {}", e))?;

        // Wait for response
        loop {
            match self.read.next().await {
                Some(Ok(Message::Text(text))) => {
                    return serde_json::from_str(&text)
                        .map_err(|e| format!("Parse error: {}", e));
                }
                Some(Ok(Message::Close(frame))) => {
                    let detail = frame
                        .map(|f| format!("code={}, reason={}", f.code, f.reason))
                        .unwrap_or_else(|| "no details".into());
                    return Err(format!("Server closed connection ({})", detail));
                }
                Some(Ok(Message::Ping(data))) => {
                    let _ = self.write.send(Message::Pong(data)).await;
                    continue;
                }
                Some(Ok(Message::Pong(_))) => continue,
                Some(Err(e)) => return Err(format!("Read error: {}", e)),
                None => return Err("Connection closed unexpectedly".into()),
                _ => continue,
            }
        }
    }

    /// Send a chat message (for ask_user responses in marionette mode).
    pub async fn send_chat(&mut self, message: &str) -> Result<(), String> {
        let msg = serde_json::json!({
            "type": "chat",
            "message": message,
        });
        self.write
            .send(Message::Text(msg.to_string()))
            .await
            .map_err(|e| format!("Send chat failed: {}", e))
    }

    /// Gracefully close the connection.
    pub async fn close(mut self) {
        let _ = self.write.send(Message::Close(None)).await;
    }
}
