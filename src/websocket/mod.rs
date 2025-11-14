//! WebSocket clients for streaming market data and user events.
//!
//! This module provides two WebSocket clients:
//! - [`MarketWsClient`]: Streams real-time order book updates for markets
//! - [`UserWsClient`]: Streams authenticated user events (trades and order updates)
//!
//! # Connection Management
//!
//! The Polymarket WebSocket server may disconnect idle connections after 1-2 minutes.
//! For production use, it's recommended to use [`ReconnectingStream`] to automatically
//! handle disconnections and reconnect with exponential backoff.
//!
//! # Authentication with UserWsClient
//!
//! The [`UserWsClient`] requires API credentials for authentication. You can provide
//! credentials in two ways:
//!
//! 1. **Using [`ApiCreds`] directly**:
//!    ```no_run
//!    # use polymarket_rs::websocket::UserWsClient;
//!    # use polymarket_rs::types::ApiCreds;
//!    # use futures_util::StreamExt;
//!    # #[tokio::main]
//!    # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!    let creds = ApiCreds::new(
//!        "your_api_key".to_string(),
//!        "your_api_secret".to_string(),
//!        "your_api_passphrase".to_string(),
//!    );
//!
//!    let client = UserWsClient::new();
//!    let mut stream = client.subscribe_with_creds(&creds).await?;
//!    # Ok(())
//!    # }
//!    ```
//!
//! 2. **Using [`AuthenticatedClient`](crate::AuthenticatedClient)**:
//!    ```no_run
//!    # use polymarket_rs::{AuthenticatedClient, ApiCreds};
//!    # use polymarket_rs::websocket::UserWsClient;
//!    # use alloy_signer_local::PrivateKeySigner;
//!    # #[tokio::main]
//!    # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!    # let signer = PrivateKeySigner::random();
//!    # let creds = ApiCreds::new("key".into(), "secret".into(), "pass".into());
//!    let auth_client = AuthenticatedClient::new(
//!        "https://clob.polymarket.com",
//!        signer,
//!        137,
//!        Some(creds),
//!        None,
//!    );
//!
//!    // Reuse credentials from AuthenticatedClient
//!    if let Some(api_creds) = auth_client.api_creds() {
//!        let ws_client = UserWsClient::new();
//!        let mut stream = ws_client.subscribe_with_creds(api_creds).await?;
//!        // Process events...
//!    }
//!    # Ok(())
//!    # }
//!    ```
//!
//! # Example: Basic Market Streaming
//!
//! ```no_run
//! use polymarket_rs::websocket::MarketWsClient;
//! use futures_util::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = MarketWsClient::new();
//!     let token_ids = vec!["your_token_id".to_string()];
//!
//!     let mut stream = client.subscribe(token_ids).await?;
//!
//!     while let Some(event) = stream.next().await {
//!         println!("Event: {:?}", event?);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Example: Resilient Streaming with Auto-Reconnect
//!
//! ```no_run
//! use polymarket_rs::websocket::{MarketWsClient, ReconnectConfig, ReconnectingStream};
//! use futures_util::StreamExt;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = MarketWsClient::new();
//!     let token_ids = vec!["your_token_id".to_string()];
//!
//!     let config = ReconnectConfig {
//!         initial_delay: Duration::from_secs(1),
//!         max_delay: Duration::from_secs(30),
//!         multiplier: 2.0,
//!         max_attempts: None, // Unlimited reconnections
//!     };
//!
//!     let token_ids_clone = token_ids.clone();
//!     let mut stream = ReconnectingStream::new(config, move || {
//!         let client = client.clone();
//!         let token_ids = token_ids_clone.clone();
//!         async move { client.subscribe(token_ids).await }
//!     });
//!
//!     while let Some(event) = stream.next().await {
//!         match event {
//!             Ok(evt) => println!("Event: {:?}", evt),
//!             Err(e) => eprintln!("Error (will reconnect): {}", e),
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

mod market;
mod stream;
mod user;

pub use market::MarketWsClient;
pub use stream::{ReconnectConfig, ReconnectingStream};
pub use user::UserWsClient;

// Re-export commonly used types for convenience
pub use crate::types::{
    BookEvent, MarketSubscription, OrderEvent, PriceChange, PriceChangeEvent, PriceLevel,
    TradeEvent, UserAuthentication, UserWsEvent, WsEvent,
};
