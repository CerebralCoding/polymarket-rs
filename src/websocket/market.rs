use futures_util::{SinkExt, Stream, StreamExt};
use std::pin::Pin;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::error::{Error, Result};
use crate::types::{MarketSubscription, WsEvent};

/// WebSocket client for streaming market data (order book updates)
///
/// This client connects to the Polymarket CLOB WebSocket endpoint and streams
/// real-time order book updates for specified token IDs.
///
/// # Connection Management
///
/// The Polymarket WebSocket server will disconnect idle connections after 1-2 minutes.
/// The Python client uses `ping_interval=5` to send keep-alive pings every 5 seconds.
///
/// For Rust, the recommended approach is to use [`ReconnectingStream`](crate::websocket::ReconnectingStream)
/// which automatically handles connection resets and reconnects with exponential backoff.
/// This is more robust than manual ping/pong management.
///
/// # Example with Auto-Reconnect
///
/// ```no_run
/// use polymarket_rs::websocket::{MarketWsClient, ReconnectConfig, ReconnectingStream};
/// use futures_util::StreamExt;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = MarketWsClient::new();
///     let token_ids = vec!["your_token_id".to_string()];
///
///     let config = ReconnectConfig {
///         initial_delay: Duration::from_secs(1),
///         max_delay: Duration::from_secs(30),
///         multiplier: 2.0,
///         max_attempts: None,
///     };
///
///     let token_ids_clone = token_ids.clone();
///     let mut stream = ReconnectingStream::new(config, move || {
///         let client = client.clone();
///         let token_ids = token_ids_clone.clone();
///         async move { client.subscribe(token_ids).await }
///     });
///
///     while let Some(event) = stream.next().await {
///         match event {
///             Ok(evt) => println!("Event: {:?}", evt),
///             Err(_) => continue, // Will auto-reconnect
///         }
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MarketWsClient {
    ws_url: String,
}

impl MarketWsClient {
    /// Default WebSocket URL for market data
    const DEFAULT_WS_URL: &'static str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";

    /// Create a new market WebSocket client with the default endpoint
    pub fn new() -> Self {
        Self {
            ws_url: Self::DEFAULT_WS_URL.to_string(),
        }
    }

    /// Create a new market WebSocket client with a custom endpoint
    pub fn with_url(ws_url: impl Into<String>) -> Self {
        Self {
            ws_url: ws_url.into(),
        }
    }

    /// Subscribe to market updates for the specified token IDs
    ///
    /// Returns a stream of [`WsEvent`] items. The stream will yield events as they
    /// are received from the WebSocket connection.
    ///
    /// # Arguments
    ///
    /// * `token_ids` - List of token/asset IDs to subscribe to
    ///
    /// # Events
    ///
    /// The stream will yield two types of events:
    /// - [`WsEvent::Book`]: Full order book snapshot (sent initially)
    /// - [`WsEvent::PriceChange`]: Incremental updates to the order book
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The WebSocket connection fails
    /// - The subscription message cannot be sent
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use polymarket_rs::websocket::MarketWsClient;
    /// # use futures_util::StreamExt;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = MarketWsClient::new();
    /// let token_ids = vec!["token_id_1".to_string(), "token_id_2".to_string()];
    ///
    /// let mut stream = client.subscribe(token_ids).await?;
    ///
    /// while let Some(event) = stream.next().await {
    ///     println!("Received event: {:?}", event?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe(
        &self,
        token_ids: Vec<String>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<WsEvent>> + Send>>> {
        // Connect to the WebSocket endpoint
        let (ws_stream, _) = connect_async(&self.ws_url).await?;

        let (write, read) = ws_stream.split();
        let mut write = write;

        // Create subscription message
        let subscription = MarketSubscription {
            assets_ids: token_ids,
        };

        let subscription_msg = serde_json::to_string(&subscription)?;

        // Send subscription message
        write
            .send(Message::Text(subscription_msg))
            .await
            .map_err(|e| Error::WebSocket(e.to_string()))?;

        // Return stream that parses events
        let stream = read.filter_map(|msg| async move {
            match msg {
                Ok(Message::Text(text)) => {
                    // The server can send either a single object or an array
                    // Try to parse as array first
                    if let Ok(events) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                        // Got an array, take the first event
                        if let Some(first) = events.first() {
                            match serde_json::from_value::<WsEvent>(first.clone()) {
                                Ok(event) => return Some(Ok(event)),
                                Err(e) => return Some(Err(Error::Json(e))),
                            }
                        } else {
                            // Empty array, ignore
                            return None;
                        }
                    }

                    // Try parsing as single object
                    match serde_json::from_str::<WsEvent>(&text) {
                        Ok(event) => Some(Ok(event)),
                        Err(e) => Some(Err(Error::Json(e))),
                    }
                }
                Ok(Message::Close(_)) => {
                    // Connection closed gracefully
                    Some(Err(Error::ConnectionClosed))
                }
                Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {
                    // Ignore ping/pong frames (handled automatically)
                    None
                }
                Ok(Message::Binary(_)) => {
                    // Unexpected binary message
                    Some(Err(Error::WebSocket(
                        "Unexpected binary message".to_string(),
                    )))
                }
                Ok(Message::Frame(_)) => {
                    // Raw frame (shouldn't happen)
                    None
                }
                Err(e) => {
                    // WebSocket error
                    Some(Err(Error::WebSocket(e.to_string())))
                }
            }
        });

        Ok(Box::pin(stream))
    }
}

impl Default for MarketWsClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = MarketWsClient::new();
        assert_eq!(client.ws_url, MarketWsClient::DEFAULT_WS_URL);
    }

    #[test]
    fn test_client_with_custom_url() {
        let custom_url = "wss://custom.example.com/ws";
        let client = MarketWsClient::with_url(custom_url);
        assert_eq!(client.ws_url, custom_url);
    }
}
