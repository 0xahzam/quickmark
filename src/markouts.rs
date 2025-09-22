//! Markout computation and analysis.
//!
//! - `compute_markouts()`: Performance calculation at multiple horizons

use crate::types::{CandleData, FillData, MarkoutResult, ProcessedFill};
use anyhow::Result;
use std::collections::HashMap;

fn load_oracle(path: &str) -> Result<HashMap<u64, f64>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut data = HashMap::new();

    for result in reader.deserialize() {
        let record: CandleData = result?;
        data.insert(record.ts, record.oracle_close);
    }

    Ok(data)
}

fn load_fills(path: &str) -> Result<Vec<ProcessedFill>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut fills = Vec::new();

    for result in reader.deserialize() {
        let record: FillData = result?;

        let base: f64 = record.base_asset_amount_filled.parse()?;
        if base == 0.0 {
            continue;
        }

        let quote: f64 = record.quote_asset_amount_filled.parse()?;
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

pub fn compute_markouts(
    oracle_path: &str,
    fills_path: &str,
    horizons: &[u32],
) -> Result<Vec<MarkoutResult>> {
    let oracle = load_oracle(oracle_path)?;
    let fills = load_fills(fills_path)?;

    let mut results = Vec::with_capacity(fills.len() * horizons.len());

    for fill in fills {
        if fill.maker_order_direction.is_empty() {
            continue;
        }

        let base_ts = floor_minute(fill.ts);
        let side = if fill.maker_order_direction == "long" {
            1
        } else {
            -1
        };

        for &horizon in horizons {
            let target_ts = base_ts + (horizon as u64 * 60);

            if let Some(oracle_price) = oracle.get(&target_ts).copied() {
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
