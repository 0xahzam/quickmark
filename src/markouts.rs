//! Markout computation and analysis.
//!
//! - `compute_markouts()`: Performance calculation at multiple horizons

use crate::types::{CandleData, FillData, MarkoutResult};
use anyhow::Result;
use std::collections::HashMap;

pub fn compute_markouts(
    oracle_path: &str,
    fills_path: &str,
    horizons: &[u32],
) -> Result<Vec<MarkoutResult>> {
    let oracle = load_oracle(oracle_path)?;
    let fills = load_fills(fills_path)?;

    let mut results = Vec::new();

    for fill in fills {
        let base_ts = floor_minute(fill.ts);

        for &horizon in horizons {
            let target_ts = base_ts + (horizon as u64 * 60);

            if let Some(&oracle_price) = oracle.get(&target_ts) {
                let side = if fill.maker_order_direction == "long" {
                    1
                } else {
                    -1
                };
                let markout = (side as f64) * (oracle_price - fill.fill_price) / fill.fill_price;

                results.push(MarkoutResult {
                    ts: fill.ts,
                    symbol: fill.symbol.clone(),
                    side,
                    fill_price: fill.fill_price,
                    horizon: format!("{}m", horizon),
                    markout,
                });
            }
        }
    }

    Ok(results)
}

fn load_oracle(path: &str) -> Result<HashMap<u64, f64>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut oracle = HashMap::new();

    for result in reader.deserialize() {
        let record: CandleData = result?;
        oracle.insert(record.ts, record.oracle_close);
    }

    Ok(oracle)
}

#[derive(Debug)]
struct ProcessedFill {
    ts: u64,
    symbol: String,
    maker_order_direction: String,
    fill_price: f64,
}

fn load_fills(path: &str) -> Result<Vec<ProcessedFill>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut fills = Vec::new();

    for result in reader.deserialize() {
        let record: FillData = result?;

        let base: f64 = record.base_asset_amount_filled.parse()?;
        let quote: f64 = record.quote_asset_amount_filled.parse()?;

        if base == 0.0 {
            continue;
        }

        let fill_price = quote / base;

        fills.push(ProcessedFill {
            ts: record.ts,
            symbol: record.symbol,
            maker_order_direction: record.maker_order_direction,
            fill_price,
        });
    }

    Ok(fills)
}

fn floor_minute(ts: u64) -> u64 {
    ts - (ts % 60)
}

pub fn parse_horizons(horizons_str: &str) -> Result<Vec<u32>> {
    horizons_str
        .split(',')
        .map(|s| s.trim().parse().map_err(Into::into))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floor_minute() {
        assert_eq!(floor_minute(1758010914), 1758010860); // 914 -> 860
        assert_eq!(floor_minute(1758010800), 1758010800); // exact minute
    }

    #[test]
    fn test_parse_horizons() {
        let horizons = parse_horizons("1,5,15").unwrap();
        assert_eq!(horizons, vec![1, 5, 15]);

        let horizons = parse_horizons("  10 , 30  ").unwrap();
        assert_eq!(horizons, vec![10, 30]);
    }
}
