//! Trade execution analysis CLI tool.
//!
//! - `oracle`: Fetch market price data to CSV
//! - `fills`: Fetch user trade data to CSV  
//! - `compute`: Calculate markouts from CSV inputs

mod batch;
mod fills;
mod markouts;
mod prices;
mod types;

use anyhow::Result;
use clap::{Parser, Subcommand};
use log::info;
use std::fs::File;

#[derive(Parser)]
#[command(name = "quickmark")]
#[command(about = "CLI tool for computing trade execution markouts on Drift Protocol")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch OHLC price data from Drift oracle
    Oracle {
        /// Trading pair symbol (e.g., BTC-PERP, DOGE-PERP)
        #[arg(long)]
        symbol: String,
        /// Candle interval in minutes
        #[arg(long, default_value_t = 1)]
        interval: u32,
        /// Number of days to fetch (max 31)
        #[arg(long, default_value_t = 3)]
        days: u32,
        /// Output CSV file path
        #[arg(long)]
        output: String,
    },
    /// Fetch user trade execution records
    Fills {
        /// Drift account public key
        #[arg(long)]
        account: String,
        /// Trading pair symbol
        #[arg(long)]
        symbol: String,
        /// Number of days to fetch (max 31)
        #[arg(long, default_value_t = 3)]
        days: u32,
        /// Output CSV file path
        #[arg(long)]
        output: String,
    },
    /// Calculate markout performance from fills and oracle data
    Compute {
        /// Path to oracle CSV file
        #[arg(long)]
        oracle: String,
        /// Path to fills CSV file  
        #[arg(long)]
        fills: String,
        /// Comma-separated markout horizons in minutes (e.g., "1,5,15")
        #[arg(long, default_value = "1,5,15")]
        horizons: String,
        /// Output CSV file path for markout results
        #[arg(long)]
        output: String,
    },
    /// Run batch analysis from config file
    Batch {
        /// Path to config file
        #[arg(long)]
        config: String,
    },
}

fn save_to_csv<T>(data: &[T], path: &str) -> Result<()>
where
    T: serde::Serialize,
{
    let file = File::create(path)?;
    let mut writer = csv::Writer::from_writer(file);

    for record in data {
        writer.serialize(record)?;
    }

    writer.flush()?;
    info!("Saved {} records to {}", data.len(), path);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_millis()
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Oracle {
            symbol,
            interval,
            days,
            output,
        } => {
            info!(
                "Fetching oracle data for {} {}m interval, {} days...",
                symbol, interval, days
            );

            let records = prices::fetch_prices(&symbol, interval, days, 1000).await?;
            save_to_csv(&records, &output)?;
        }
        Commands::Fills {
            account,
            symbol,
            days,
            output,
        } => {
            info!(
                "Fetching fills for account {} symbol {} for {} days...",
                account, symbol, days
            );
            let records = fills::fetch_fills(&account, &symbol, days).await?;
            save_to_csv(&records, &output)?;
        }
        Commands::Compute {
            oracle,
            fills,
            horizons,
            output,
        } => {
            info!(
                "Computing markouts from {} and {} with horizons {}...",
                oracle, fills, horizons
            );
            let horizon_vec = markouts::parse_horizons(&horizons)?;
            let results = markouts::compute_markouts(&oracle, &fills, &horizon_vec)?;
            save_to_csv(&results, &output)?;
        }
        Commands::Batch { config } => {
            batch::run_batch(&config).await?;
        }
    }

    Ok(())
}
