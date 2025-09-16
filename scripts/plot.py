import pandas as pd
import matplotlib.pyplot as plt
import sys
from pathlib import Path


def plot_markouts(csv_path):
    df = pd.read_csv(csv_path)

    # Get unique horizons and sort them
    horizons = sorted(df["horizon"].unique(), key=lambda x: int(x[:-1]))

    plt.figure(figsize=(12, 8))

    # Plot cumulative markout for each horizon
    for horizon in horizons:
        subset = df[df["horizon"] == horizon].copy()
        subset = subset.sort_values("ts")
        subset["cumulative_markout"] = subset["markout"].cumsum()

        plt.plot(
            range(len(subset)),
            subset["cumulative_markout"],
            label=f"{horizon} horizon",
            linewidth=2,
        )

    plt.xlabel("Trade Number")
    plt.ylabel("Cumulative Markout")
    plt.title("Cumulative Markout Performance by Horizon")
    plt.legend()
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    plt.show()

    # Print summary stats
    print("\nMarkout Summary:")
    print("-" * 40)
    for horizon in horizons:
        subset = df[df["horizon"] == horizon]
        avg_markout = subset["markout"].mean()
        print(f"{horizon:>6}: avg={avg_markout:>8.4f}, trades={len(subset):>3}")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python scripts/plot.py <markouts_csv>")
        sys.exit(1)

    csv_path = Path(sys.argv[1])
    if not csv_path.exists():
        print(f"File not found: {csv_path}")
        sys.exit(1)

    plot_markouts(csv_path)
