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

## Complete Example

```bash
mkdir -p data

# Get data
./target/release/quickmark oracle --symbol DOGE-PERP --days 3 --output data/oracle_doge.csv
./target/release/quickmark fills --account XyzRandomAccount11 --symbol DOGE-PERP --days 3 --output data/fills_doge.csv

# Analyze
./target/release/quickmark compute --oracle data/oracle_doge.csv --fills data/fills_doge.csv --output data/markouts_doge.csv

# Visualize
python scripts/plot.py data/markouts_doge.csv
```
