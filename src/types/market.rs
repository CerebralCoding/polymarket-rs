use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Full market information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub condition_id: String,
    pub tokens: [Token; 2],
    pub rewards: Rewards,
    pub min_incentive_size: Option<String>,
    pub max_incentive_spread: Option<String>,
    pub active: bool,
    pub closed: bool,
    pub enable_order_book: bool,
    pub archived: bool,
    pub accepting_orders: bool,
    pub question_id: String,
    pub question: String,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_decimal")]
    pub minimum_order_size: Decimal,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_decimal")]
    pub minimum_tick_size: Decimal,
    pub description: String,
    pub category: Option<String>,
    pub end_date_iso: Option<String>,
    pub game_start_time: Option<String>,
    pub market_slug: String,
    pub icon: String,
    pub fpmm: String,
    pub neg_risk: bool,
    pub neg_risk_market_id: String,
    pub neg_risk_request_id: String,
}

/// Simplified market information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedMarket {
    pub condition_id: String,
    pub tokens: [Token; 2],
    pub rewards: Rewards,
    pub active: bool,
    pub closed: bool,
    pub archived: bool,
    pub accepting_orders: bool,
}

/// Token within a market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub token_id: String,
    pub outcome: String,
}

/// Market rewards configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rewards {
    pub rates: Option<Vec<RewardsRates>>,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_decimal")]
    pub min_size: Decimal,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_decimal")]
    pub max_spread: Decimal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RewardsRates {
    pub asset_address: String,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_decimal")]
    pub rewards_daily_rate: Decimal,
}

/// Paginated markets response
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketsResponse {
    pub limit: u64,
    pub count: u64,
    pub next_cursor: Option<String>,
    pub data: Vec<Market>,
}

/// Paginated simplified markets response
#[derive(Debug, Serialize, Deserialize)]
pub struct SimplifiedMarketsResponse {
    pub limit: u64,
    pub count: u64,
    pub next_cursor: Option<String>,
    pub data: Vec<SimplifiedMarket>,
}

/// Midpoint price response
#[derive(Debug, Deserialize, Serialize)]
pub struct MidpointResponse {
    #[serde(with = "rust_decimal::serde::str")]
    pub mid: Decimal,
}

/// Price response
#[derive(Debug, Deserialize)]
pub struct PriceResponse {
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
}

/// Price history response
#[derive(Debug, Deserialize)]
pub struct PriceHistoryResponse {
    pub history: Vec<PriceHistory>,
}

/// Price at a specific timestamp
#[derive(Debug, Deserialize)]
pub struct PriceHistory {
    #[serde(
        rename = "p",
        deserialize_with = "super::serde_helpers::deserialize_decimal"
    )]
    pub price: Decimal,
    #[serde(rename = "t")]
    pub timestamp: u64,
}

/// Spread response
#[derive(Debug, Deserialize)]
pub struct SpreadResponse {
    #[serde(with = "rust_decimal::serde::str")]
    pub spread: Decimal,
}

/// Tick size response
#[derive(Debug, Deserialize)]
pub struct TickSizeResponse {
    #[serde(deserialize_with = "super::serde_helpers::deserialize_decimal")]
    pub minimum_tick_size: Decimal,
}

/// Negative risk response
#[derive(Debug, Deserialize)]
pub struct NegRiskResponse {
    pub neg_risk: bool,
}
