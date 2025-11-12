use alloy_signer_local::PrivateKeySigner;
use polymarket_rs::client::{AuthenticatedClient, TradingClient};
use polymarket_rs::orders::OrderBuilder;
use polymarket_rs::types::{CreateOrderOptions, OrderArgs, Side, SignatureType};
use polymarket_rs::Result;
use rust_decimal::Decimal;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    // IMPORTANT: Replace with your actual private key
    // NEVER commit your private key to version control!
    let private_key =
        std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY environment variable not set");

    let signer = PrivateKeySigner::from_str(&private_key).expect("Invalid private key");

    let chain_id = 137; // Polygon Mainnet
    let host = "https://clob.polymarket.com";

    println!("Wallet address: {}", signer.address());

    // Step 1: Create or derive API credentials
    println!("\n1. Creating/deriving API credentials...");
    let auth_client = AuthenticatedClient::new(host, signer.clone(), chain_id, None);

    let api_creds = auth_client.create_or_derive_api_key().await?;
    println!("API Key: {}", api_creds.api_key);
    println!("Successfully authenticated!");

    // Step 2: Create a trading client
    println!("\n2. Setting up trading client...");
    let order_builder = OrderBuilder::new(signer.clone(), Some(SignatureType::Eoa), None);

    let trading_client = TradingClient::new(
        host,
        signer.clone(),
        chain_id,
        api_creds.clone(),
        order_builder,
    );

    // Step 3: Get existing orders
    println!("\n3. Fetching existing orders...");
    let orders = trading_client.get_orders(Default::default()).await?;
    println!("Found {} open orders", orders.len());

    for order in orders.iter().take(5) {
        println!(
            "  Order {}: {:?} {} @ {}",
            order.id, order.side, order.original_size, order.price
        );
    }

    // Step 4: Create a limit order (example - NOT posted)
    println!("\n4. Creating a limit order (example)...");

    let token_id = "109648317055340591503076024421581448189531885907475125926203413622318314876012";

    let _order_args = OrderArgs::new(
        token_id,
        Decimal::from_str("0.50").unwrap(), // price
        Decimal::from_str("10.0").unwrap(), // size
        Side::Buy,
    );

    let _options = CreateOrderOptions::default()
        .tick_size(Decimal::from_str("0.01").unwrap())
        .neg_risk(false);

    // Note: This creates the order but doesn't post it
    // Uncomment the following to actually post:
    // let signed_order = trading_client.create_order(
    //     &_order_args,
    //     None,      // expiration (defaults to 0 = no expiration)
    //     None,      // extras (defaults to ExtraOrderArgs::default())
    //     _options,
    // )?;
    //
    // println!("Created signed order with salt: {}", signed_order.salt);
    //
    // // Post the order
    // let result = trading_client.post_order(signed_order, OrderType::Gtc).await?;
    // println!("Order posted: {:?}", result);

    println!("Order creation example completed (not posted)");

    // Step 5: Get trade history
    println!("\n5. Fetching trade history...");
    let trades = trading_client.get_trades(Default::default()).await?;
    println!("Trade history: {:?}", trades);

    println!("\n✓ Example completed successfully!");
    println!("\nNOTE: This example did not post any actual orders.");
    println!("Uncomment the posting code to execute real trades.");

    Ok(())
}
