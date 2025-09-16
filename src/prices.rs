//! Market price data fetching.
//!
//! - `fetch_oracle()`: Backward time iteration for candle data

use crate::types::CandleData;
use anyhow::{Result, bail};

pub async fn fetch_prices(
    symbol: &str,
    interval: u32,
    days: u32,
    limit: u32,
) -> Result<Vec<CandleData>> {
    let client = reqwest::Client::new();
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

        let res = client
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

        let json: serde_json::Value = res.json().await?;

        if !json.get("records").is_some() {
            break;
        }

        let records: Vec<CandleData> = serde_json::from_value(json["records"].clone())?;

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

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Sort by timestamp (oldest first)
    all_records.sort_by_key(|r| r.ts);

    Ok(all_records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_prices_btc() {
        let result = fetch_prices("BTC-PERP", 1, 3, 1000).await;

        match result {
            Ok(records) => {
                println!("Fetched {} candles", records.len());
                if !records.is_empty() {
                    println!(
                        "First candle: ts={}, close={}",
                        records[0].ts, records[0].oracle_close
                    );
                    println!(
                        "Last candle: ts={}, close={}",
                        records.last().unwrap().ts,
                        records.last().unwrap().oracle_close
                    );
                }
            }
            Err(e) => panic!("Oracle fetch failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_fetch_prices_doge() {
        let result = fetch_prices("DOGE-PERP", 5, 1, 500).await;

        match result {
            Ok(records) => {
                println!("DOGE 5m candles: {}", records.len());
                assert!(!records.is_empty());
                // Verify sorting
                for i in 1..records.len() {
                    assert!(records[i].ts >= records[i - 1].ts);
                }
            }
            Err(e) => panic!("DOGE oracle fetch failed: {}", e),
        }
    }
}
