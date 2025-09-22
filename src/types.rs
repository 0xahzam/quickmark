//! Core data types for markouts analysis.
//!
//! - `CandleData`: Raw OHLC data from oracle API
//! - `FillData`: Complete trade records from fills API

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CandleData {
    pub ts: u64,
    pub fill_open: f64,
    pub fill_high: f64,
    pub fill_close: f64,
    pub fill_low: f64,
    pub oracle_open: f64,
    pub oracle_high: f64,
    pub oracle_low: f64,
    pub oracle_close: f64,
    pub quote_volume: f64,
    pub base_volume: f64,
}

#[derive(serde::Deserialize)]
pub struct CandleApiResponse {
    pub records: Option<Vec<CandleData>>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FillData {
    pub ts: u64,
    pub tx_sig: String,
    pub tx_sig_index: u64,
    pub slot: u64,
    pub filler_reward: String,
    pub base_asset_amount_filled: String,
    pub quote_asset_amount_filled: String,
    pub taker_fee: String,
    pub maker_rebate: String,
    pub referrer_reward: String,
    pub quote_asset_amount_surplus: String,
    pub taker_order_base_asset_amount: String,
    pub taker_order_cumulative_base_asset_amount_filled: String,
    pub taker_order_cumulative_quote_asset_amount_filled: String,
    pub maker_order_base_asset_amount: String,
    pub maker_order_cumulative_base_asset_amount_filled: String,
    pub maker_order_cumulative_quote_asset_amount_filled: String,
    pub oracle_price: String,
    pub maker_fee: String,
    pub action: String,
    pub action_explanation: String,
    pub market_index: u64,
    pub market_type: String,
    pub filler: String,
    pub fill_record_id: String,
    pub taker: String,
    pub taker_order_id: String,
    pub taker_order_direction: String,
    pub maker: String,
    pub maker_order_id: String,
    pub maker_order_direction: String,
    pub spot_fulfillment_method_fee: String,
    pub market_filter: String,
    pub user: String,
    pub symbol: String,
    pub bit_flags: u64,
    pub taker_existing_quote_entry_amount: String,
    pub taker_existing_base_asset_amount: String,
    pub maker_existing_quote_entry_amount: String,
    pub maker_existing_base_asset_amount: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FillsMeta {
    pub next_page: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct FillsApiResponse {
    pub success: bool,
    pub records: Vec<FillData>,
    pub meta: FillsMeta,
}

#[derive(Debug)]
pub struct ProcessedFill {
    pub ts: u64,
    pub symbol: String,
    pub maker_order_direction: String,
    pub fill_price: f64,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize)]
pub struct MarkoutResult {
    pub ts: u64,
    pub symbol: String,
    pub side: i8,
    pub fill_price: f64,
    pub horizon: String,
    pub markout: f64,
}
