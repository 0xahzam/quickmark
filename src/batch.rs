//! Batch processing from config files.
//!
//! - `run_batch()`: Sequential pipeline execution from TOML config

use crate::{fills, markouts, prices, save_to_csv};
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize)]
pub struct Config {
    pub global: GlobalConfig,
    pub accounts: Vec<AccountConfig>,
}

#[derive(Deserialize)]
pub struct GlobalConfig {
    pub days: u32,
    pub horizons: Vec<u32>,
    pub output_dir: String,
}

#[derive(Deserialize)]
pub struct AccountConfig {
    pub id: String,
    pub symbols: Vec<String>,
}

pub async fn run_batch(config_path: &str) -> Result<()> {
    let config_content = std::fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_content)?;

    std::fs::create_dir_all(&config.global.output_dir)?;

    let unique_symbols: HashSet<_> = config
        .accounts
        .iter()
        .flat_map(|acc| &acc.symbols)
        .collect();

    println!(
        "Batch analysis: {} symbols, {} accounts, {} days",
        unique_symbols.len(),
        config.accounts.len(),
        config.global.days
    );

    // Fetch oracle data
    for symbol in &unique_symbols {
        println!("Fetching oracle for {}", symbol);
        let records = prices::fetch_prices(symbol, 1, config.global.days, 1000).await?;
        let path = format!("{}/oracle_{}.csv", config.global.output_dir, symbol);
        save_to_csv(&records, &path)?;
    }

    // Fetch fills data
    for account in &config.accounts {
        for symbol in &account.symbols {
            println!("Fetching fills for {} {}", &account.id[..8], symbol);
            let records = fills::fetch_fills(&account.id, symbol, config.global.days).await?;
            let path = format!(
                "{}/fills_{}_{}.csv",
                config.global.output_dir,
                &account.id[..8],
                symbol
            );
            save_to_csv(&records, &path)?;
        }
    }

    // Compute markouts
    for account in &config.accounts {
        for symbol in &account.symbols {
            println!("Computing markouts for {} {}", &account.id[..8], symbol);

            let oracle_path = format!("{}/oracle_{}.csv", config.global.output_dir, symbol);
            let fills_path = format!(
                "{}/fills_{}_{}.csv",
                config.global.output_dir,
                &account.id[..8],
                symbol
            );
            let output_path = format!(
                "{}/markouts_{}_{}.csv",
                config.global.output_dir,
                &account.id[..8],
                symbol
            );

            let results =
                markouts::compute_markouts(&oracle_path, &fills_path, &config.global.horizons)?;

            save_to_csv(&results, &output_path)?;
        }
    }

    println!("Batch complete: {}", config.global.output_dir);
    Ok(())
}
