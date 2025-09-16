//! User trade execution data fetching.
//!
//! - `fetch_fills()`: Paginated API retrieval with time filtering

use crate::types::FillData;
use anyhow::{Result, bail};

pub async fn fetch_fills(account_id: &str, symbol: &str, days: u32) -> Result<Vec<FillData>> {
    if days > 31 {
        bail!("API only provides last 31 days of data");
    }

    let client = reqwest::Client::new();
    let cutoff_ts = (chrono::Utc::now() - chrono::Duration::days(days as i64)).timestamp() as u64;
    let base_url = format!(
        "https://data.api.drift.trade/user/{}/trades/{}",
        account_id, symbol
    );

    let mut all_records = Vec::new();
    let mut next_page: Option<String> = None;

    loop {
        let params = if let Some(page) = &next_page {
            vec![("page", page.as_str())]
        } else {
            vec![]
        };

        let url = reqwest::Url::parse_with_params(&base_url, &params)?;
        let res = client.get(url).send().await?;
        let json: serde_json::Value = res.json().await?;

        if !json["success"].as_bool().unwrap_or(false) {
            bail!("API request failed");
        }

        let records: Vec<FillData> = serde_json::from_value(json["records"].clone())?;
        let filtered: Vec<FillData> = records
            .into_iter()
            .filter(|record| record.ts >= cutoff_ts)
            .collect();

        if filtered.is_empty() {
            break;
        }

        all_records.extend(filtered);
        next_page = json["meta"]["nextPage"].as_str().map(|s| s.to_string());

        if next_page.is_none() {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    Ok(all_records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_fills_validation() {
        let result = fetch_fills("test_account", "BTC-PERP", 32).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("31 days"));
    }

    #[tokio::test]
    async fn test_fetch_fills_real_api() {
        let result = fetch_fills("", "DOGE-PERP", 1).await;

        match result {
            Ok(records) => {
                println!("Fetched {} records", records.len());
                if !records.is_empty() {
                    println!("Sample record: {:?}", &records[0]);
                }
            }
            Err(e) => panic!("API call failed: {}", e),
        }
    }
}
