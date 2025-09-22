//! Market price data fetching.
//!
//! - `fetch_oracle()`: Backward time iteration for candle data

use crate::types::{CandleApiResponse, CandleData};
use anyhow::{Result, bail};
use lazy_static::lazy_static;

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}

pub async fn fetch_prices(
    symbol: &str,
    interval: u32,
    days: u32,
    limit: u32,
) -> Result<Vec<CandleData>> {
    let now = chrono::Utc::now().timestamp() as u64;
    let cutoff_ts = now - (days as u64 * 86400);

    let mut all_records = Vec::new();
    let mut start_ts = now;
    let end_ts = cutoff_ts;

    loop {
        let url = format!(
            "https://data.api.drift.trade/market/{}/candles/{}",
            symbol, interval
        );

        let res = HTTP_CLIENT
            .get(&url)
            .query(&[
                ("startTs", start_ts.to_string()),
                ("endTs", end_ts.to_string()),
                ("limit", limit.to_string()),
            ])
            .send()
            .await?;

        if !res.status().is_success() {
            bail!("API request failed with status: {}", res.status());
        }

        let response: CandleApiResponse = res.json().await?;
        let records = response.records.unwrap_or_default();

        if records.is_empty() {
            break;
        }

        let last_ts = records.last().unwrap().ts;

        // Filter records within our time window
        let filtered: Vec<CandleData> = records
            .into_iter()
            .filter(|record| record.ts >= cutoff_ts)
            .collect();

        all_records.extend(filtered);

        // Move start_ts to continue backwards
        if last_ts <= cutoff_ts {
            break;
        }
        start_ts = last_ts - 1;

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Sort by timestamp (oldest first)
    all_records.sort_by_key(|r| r.ts);

    Ok(all_records)
}
