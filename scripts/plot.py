import pandas as pd
import matplotlib.pyplot as plt
import sys
import glob
from pathlib import Path


def plot_single_markout(csv_path):
    df = pd.read_csv(csv_path)
    df["datetime"] = pd.to_datetime(df["ts"], unit="s")

    horizons = sorted(df["horizon"].unique(), key=lambda x: int(x[:-1]))

    plt.figure(figsize=(12, 8))

    for horizon in horizons:
        subset = df[df["horizon"] == horizon].copy()
        subset = subset.sort_values("datetime")
        subset["cumulative_markout"] = subset["markout"].cumsum()

        plt.plot(
            subset["datetime"],
            subset["cumulative_markout"],
            label=f"{horizon} horizon",
            linewidth=2,
        )

    plt.xlabel("Time")
    plt.ylabel("Cumulative Markout")
    plt.title("Cumulative Markout Performance by Horizon")
    plt.legend()
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    plt.show()


def plot_all_markouts(directory="data"):
    pattern = f"{directory}/markouts_*.csv"
    files = glob.glob(pattern)

    if not files:
        print(f"No files found matching: {pattern}")
        return

    n_files = len(files)
    cols = min(4, n_files)
    rows = (n_files + cols - 1) // cols

    fig, axes = plt.subplots(rows, cols, figsize=(4 * cols, 3 * rows))

    if n_files == 1:
        axes = [axes]
    elif rows == 1:
        axes = list(axes)
    else:
        axes = axes.flatten()

    for i, csv_path in enumerate(files):
        df = pd.read_csv(csv_path)
        df["datetime"] = pd.to_datetime(df["ts"], unit="s")

        parts = Path(csv_path).stem.split("_")
        account = parts[1]
        symbol = "_".join(parts[2:])
        title = f"{account[:8]} {symbol}"

        horizons = sorted(df["horizon"].unique(), key=lambda x: int(x[:-1]))

        for horizon in horizons:
            subset = df[df["horizon"] == horizon].copy()
            subset = subset.sort_values("datetime")
            subset["cumulative_markout"] = subset["markout"].cumsum()

            axes[i].plot(
                subset["datetime"],
                subset["cumulative_markout"],
                label=horizon,
                linewidth=1.5,
            )

        axes[i].set_title(title, fontsize=10)
        axes[i].legend(fontsize=8)
        axes[i].grid(True, alpha=0.3)
        axes[i].tick_params(axis="both", labelsize=8)

    for i in range(n_files, len(axes)):
        axes[i].set_visible(False)

    plt.tight_layout()
    plt.show()


if __name__ == "__main__":
    if len(sys.argv) > 1:
        arg = sys.argv[1]
        if arg.endswith(".csv"):
            # Single file mode
            csv_path = Path(arg)
            if not csv_path.exists():
                print(f"File not found: {csv_path}")
                sys.exit(1)
            plot_single_markout(csv_path)
        else:
            # Directory mode
            plot_all_markouts(arg)
    else:
        # Default directory mode
        plot_all_markouts()
