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

        // Wait for response
        match self.read.next().await {
            Some(Ok(Message::Text(text))) => {
                serde_json::from_str(&text).map_err(|e| format!("Parse error: {}", e))
            }
            Some(Ok(Message::Close(_))) => Err("Server closed connection".into()),
            Some(Err(e)) => Err(format!("Read error: {}", e)),
            None => Err("Connection closed".into()),
            _ => Err("Unexpected message type".into()),
        }
    }

    /// Gracefully close the connection.
    pub async fn close(mut self) {
        let _ = self.write.send(Message::Close(None)).await;
    }
}
