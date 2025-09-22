//! User trade execution data fetching.
//!
//! - `fetch_fills()`: Paginated API retrieval with time filtering

use crate::types::{FillData, FillsApiResponse};
use anyhow::{Result, bail};
use lazy_static::lazy_static;

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}

pub async fn fetch_fills(account_id: &str, symbol: &str, days: u32) -> Result<Vec<FillData>> {
    if days > 31 {
        bail!("API only provides last 31 days of data");
    }

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
        let res = HTTP_CLIENT.get(url).send().await?;
        let response: FillsApiResponse = res.json().await?;

        if !response.success {
            bail!("API request failed");
        }

        let mut filtered_count = 0;
        for record in response.records {
            if record.ts >= cutoff_ts {
                all_records.push(record);
                filtered_count += 1;
            }
        }

        if filtered_count == 0 {
            break;
        }

        next_page = response.meta.next_page;

        if next_page.is_none() {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    Ok(all_records)
}
