use crate::error::{Error, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

pub struct Connection {
    pub ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl Connection {
    //Connecting to the Discord Gateway
    pub async fn connect(url: &str) -> Result<Self> {
        tracing::info!("Connecting to Discord Gateway at {}", url);

        let (ws, _response) = connect_async(url)
            .await
            .map_err(|e| Error::GatewayConnection(e.to_string()))?;

        tracing::info!("Successfully connected!");
        Ok(Self { ws })
    }

    pub async fn receive(&mut self) -> Result<Option<Value>> {
        while let Some(msg) = self.ws.next().await {
            let msg = msg?;

            match msg {
                Message::Text(text) => {
                    let payload: Value = serde_json::from_str(&text)?;
                    let redacted = redact_gateway_payload(&payload);
                    let redacted_text = serde_json::to_string(&redacted)?;
                    tracing::debug!("Received: {}", redacted_text);
                    return Ok(Some(payload));
                }
                Message::Close(frame) => {
                    tracing::warn!("WebSocket closed: {:?}", frame);
                    return Ok(None);
                }
                _ => {
                    //ignore other message types (binary, ping, pong)
                    continue;
                }
            }
        }
        Ok(None)
    }

    pub async fn send(&mut self, payload: &Value) -> Result<()> {
        let text = serde_json::to_string(payload)?;
        let redacted = redact_gateway_payload(payload);
        let redacted_text = serde_json::to_string(&redacted)?;
        tracing::debug!("Sending: {}", redacted_text);

        self.ws.send(Message::Text(text)).await?;
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        self.ws.close(None).await?;
        Ok(())
    }
}

fn redact_gateway_payload(payload: &Value) -> Value {
    let mut out = payload.clone();
    redact_sensitive_keys(&mut out);
    out
}

fn redact_sensitive_keys(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, child) in map.iter_mut() {
                let lower = key.to_ascii_lowercase();
                if lower == "token"
                    || lower == "access_token"
                    || lower == "refresh_token"
                    || lower.ends_with("_token")
                {
                    *child = Value::String("[REDACTED]".to_string());
                } else {
                    redact_sensitive_keys(child);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                redact_sensitive_keys(item);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::redact_gateway_payload;
    use serde_json::json;

    #[test]
    fn redact_gateway_payload_masks_tokens_recursively() {
        let payload = json!({
            "op": 0,
            "d": {
                "token": "abc",
                "nested": {
                    "access_token": "def"
                },
                "items": [
                    {
                        "refresh_token": "ghi"
                    }
                ]
            }
        });

        let redacted = redact_gateway_payload(&payload);
        assert_eq!(redacted["d"]["token"], "[REDACTED]");
        assert_eq!(redacted["d"]["nested"]["access_token"], "[REDACTED]");
        assert_eq!(redacted["d"]["items"][0]["refresh_token"], "[REDACTED]");
    }
}
