# Quickmark

Trade execution analysis tool for Drift Protocol.

## Installation

```bash
cargo build --release
```

## Usage

```bash
quickmark <COMMAND>

Commands:
  oracle   Fetch OHLC price data from Drift oracle
  fills    Fetch user trade execution records
  compute  Calculate markout performance from fills and oracle data
```

### Oracle Command

```bash
quickmark oracle --symbol DOGE-PERP --interval 1 --days 3 --output data/oracle.csv
```

- `--symbol`: Trading pair (BTC-PERP, DOGE-PERP, etc.)
- `--interval`: Candle interval in minutes (default: 1)
- `--days`: Days to fetch, max 31 (default: 3)
- `--output`: Output CSV file path

### Fills Command

```bash
quickmark fills --account <PUBKEY> --symbol DOGE-PERP --days 3 --output data/fills.csv
```

- `--account`: Drift account public key
- `--symbol`: Trading pair
- `--days`: Days to fetch, max 31 (default: 3)
- `--output`: Output CSV file path

### Compute Command

```bash
quickmark compute --oracle data/oracle.csv --fills data/fills.csv --horizons 1,5,15 --output data/markouts.csv
```

- `--oracle`: Path to oracle CSV file
- `--fills`: Path to fills CSV file
- `--horizons`: Comma-separated horizons in minutes (default: "1,5,15")
- `--output`: Output CSV file path

### Batch Command

```bash
quickmark batch --config analysis.toml
```

- `--config`: Path to TOML config file

Runs complete oracle → fills → markouts pipeline for multiple accounts and symbols.

**Config format:**

```toml
[global]
days = 3
horizons = [1, 5, 15]
output_dir = "data"

[[accounts]]
id = "AccountPubkey123..."
symbols = ["BTC-PERP", "ETH-PERP", "SOL-PERP"]

[[accounts]]
id = "AnotherAccountPubkey456..."
symbols = ["XRP-PERP", "WIF-PERP", "DOGE-PERP"]
```

**Example:**

```bash
./target/release/quickmark batch --config analysis.toml
```

## Complete Example

```bash
mkdir -p data

# Get data
./target/release/quickmark oracle --symbol DOGE-PERP --days 3 --output data/oracle_doge.csv
./target/release/quickmark fills --account XyzRandomAccount11 --symbol DOGE-PERP --days 3 --output data/fills_doge.csv

# Analyze
./target/release/quickmark compute --oracle data/oracle_doge.csv --fills data/fills_doge.csv --output data/markouts_doge.csv

# Visualize single markout file
uv run scripts/plot.py data/markouts_XyzRandomAccount11_HYPE-PERP.csv

# Visualize all markout files in grid
uv run scripts/plot.py

# Visualize all markout files in in results/ directory
uv run scripts/plot.py results
```
