//! Batch processing from config files.
//!
//! - `run_batch()`: Sequential pipeline execution from TOML config

use crate::{fills, markouts, prices, save_to_csv};
use anyhow::Result;
use futures::future::try_join_all;
use log::info;
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

async fn fetch_oracle_for_symbol(symbol: &str, days: u32, output_dir: &str) -> anyhow::Result<()> {
    let records = prices::fetch_prices(symbol, 1, days, 1000).await?;
    let path = format!("{}/oracle_{}.csv", output_dir, symbol);
    save_to_csv(&records, &path)?;
    info!("Oracle data saved for {}", symbol);
    Ok(())
}

async fn fetch_fills_for_symbol(
    account_id: &str,
    symbol: &str,
    days: u32,
    output_dir: &str,
) -> anyhow::Result<()> {
    let records = fills::fetch_fills(account_id, symbol, days).await?;
    let path = format!("{}/fills_{}_{}.csv", output_dir, &account_id[..8], symbol);
    save_to_csv(&records, &path)?;
    info!("Fills saved for {} {}", &account_id[..8], symbol);
    Ok(())
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

    info!(
        "Batch analysis: {} symbols, {} accounts, {} days",
        unique_symbols.len(),
        config.accounts.len(),
        config.global.days
    );

    let output_dir = &config.global.output_dir;
    let days = config.global.days;

    // Fetch oracle data
    let oracle_futures: Vec<_> = unique_symbols
        .iter()
        .map(|symbol| fetch_oracle_for_symbol(symbol, days, output_dir))
        .collect();

    info!(
        "Fetching oracle data for {} symbols...",
        unique_symbols.len()
    );

    // Fetch fills data
    let fills_futures: Vec<_> = config
        .accounts
        .iter()
        .flat_map(|account| {
            let account_id = &account.id;
            account
                .symbols
                .iter()
                .map(move |symbol| fetch_fills_for_symbol(account_id, symbol, days, output_dir))
        })
        .collect();

    info!(
        "Fetching fills data for {} account-symbol pairs...",
        config
            .accounts
            .iter()
            .map(|a| a.symbols.len())
            .sum::<usize>()
    );

    let (_, _) =
        futures::future::try_join(try_join_all(oracle_futures), try_join_all(fills_futures))
            .await?;

    // Compute markouts
    for (account, symbol) in config
        .accounts
        .iter()
        .flat_map(|account| account.symbols.iter().map(move |symbol| (account, symbol)))
    {
        info!("Computing markouts for {} {}", &account.id[..8], symbol);
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

    info!("Batch complete: {}", config.global.output_dir);
    Ok(())
}
