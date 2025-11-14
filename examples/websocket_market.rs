use futures_util::StreamExt;
use polymarket_rs::types::WsEvent;
use polymarket_rs::websocket::{MarketWsClient, ReconnectConfig, ReconnectingStream};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = MarketWsClient::new();

    let token_ids = vec![
        // "Yes" (outcome token) for "Fed decreases interest rates by 25 bps after December 2025 meeting?"
        "87769991026114894163580777793845523168226980076553814689875238288185044414090".to_string(),
    ];

    println!("Connecting to CLOB WebSocket...");
    println!("Subscribing to {} token(s)", token_ids.len());

    // Configure automatic reconnection
    let config = ReconnectConfig {
        initial_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(30),
        multiplier: 2.0,
        max_attempts: None, // Unlimited reconnection attempts
    };

    // Create a reconnecting stream that will automatically reconnect on disconnection
    let token_ids_clone = token_ids.clone();
    let mut stream = ReconnectingStream::new(config, move || {
        let client = client.clone();
        let token_ids = token_ids_clone.clone();
        async move {
            println!("🔌 Connecting to WebSocket...");
            let result = client.subscribe(token_ids).await;
            if result.is_ok() {
                println!("✅ Connected successfully!");
            }
            result
        }
    });

    println!("Connected! Waiting for events...\n");

    // Process events as they arrive
    let mut event_count = 0;
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                event_count += 1;
                match event {
                    WsEvent::Book(book) => {
                        println!("[Book Event #{}]", event_count);
                        println!("  Market: {}", book.market);
                        println!("  Asset ID: {}", book.asset_id);
                        println!("  Bids: {} levels", book.bids.len());
                        if let Some(best_bid) = book.bids.first() {
                            println!("    Best bid: {} @ {}", best_bid.size, best_bid.price);
                        }
                        println!("  Asks: {} levels", book.asks.len());
                        if let Some(best_ask) = book.asks.first() {
                            println!("    Best ask: {} @ {}", best_ask.size, best_ask.price);
                        }
                        println!();
                    }
                    WsEvent::PriceChange(change) => {
                        println!("[Price Change Event #{}]", event_count);
                        println!("  Market: {}", change.market);
                        println!("  Changes: {}", change.price_changes.len());
                        for price_change in &change.price_changes {
                            println!(
                                "    {:?} @ {}: {} ({})",
                                price_change.side,
                                price_change.price,
                                price_change.size,
                                if price_change.size.is_zero() {
                                    "removed"
                                } else {
                                    "updated"
                                }
                            );
                        }
                        println!();
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
                // The ReconnectingStream will automatically attempt to reconnect
                // Continue processing to allow reconnection
                eprintln!("⏳ Will attempt to reconnect...");
            }
        }
    }

    println!("WebSocket stream ended.");
    Ok(())
}
