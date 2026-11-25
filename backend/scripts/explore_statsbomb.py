"""
Milestone 1 Script: StatsBomb Open Data Explorer & Shot Map Generator.

Demonstrates:
1. Querying official Hudl/StatsBomb Open Data via `statsbombpy`.
2. Filtering match events (Shots) and handling missing values with `pandas`.
3. Plotting a publication-quality xG Shot Map using `mplsoccer`.

Run from `backend/`:
    uv run python scripts/explore_statsbomb.py
"""

from pathlib import Path
import warnings

import matplotlib.pyplot as plt
from mplsoccer import VerticalPitch
from statsbombpy import sb

# Suppress statsbombpy open-data credentials warning
warnings.filterwarnings("ignore", category=UserWarning, module="statsbombpy")

OUTPUT_DIR = Path(__file__).resolve().parent.parent.parent / "outputs"


def generate_world_cup_final_shotmap() -> Path:
    """Fetch 2022 World Cup Final events (Argentina vs France) and plot an xG shot map."""
    # Competition 43 = FIFA World Cup, Season 106 = 2022
    matches = sb.matches(competition_id=43, season_id=106)
    final_match = matches[matches["match_date"] == "2022-12-18"].iloc[0]
    match_id = int(final_match["match_id"])

    print(
        f"Loaded Match {match_id}: "
        f"{final_match['home_team']} {final_match['home_score']} - "
        f"{final_match['away_score']} {final_match['away_team']}"
    )

    events = sb.events(match_id=match_id)
    shots = events[(events["type"] == "Shot") & (events["period"] <= 4)].copy()

    # Extract StatsBomb [x, y] coordinates (pitch is 120x80)
    shots["x"] = shots["location"].apply(lambda loc: loc[0])
    shots["y"] = shots["location"].apply(lambda loc: loc[1])

    argentina_shots = shots[shots["team"] == "Argentina"]

    pitch = VerticalPitch(
        pitch_type="statsbomb",
        half=True,
        pitch_color="#0f172a",
        line_color="#475569",
    )
    fig, ax = pitch.draw(figsize=(10, 7))
    fig.patch.set_facecolor("#0f172a")

    goals = argentina_shots[argentina_shots["shot_outcome"] == "Goal"]
    non_goals = argentina_shots[argentina_shots["shot_outcome"] != "Goal"]

    pitch.scatter(
        non_goals["x"],
        non_goals["y"],
        s=(non_goals["shot_statsbomb_xg"] * 600) + 40,
        c="#38bdf8",
        alpha=0.6,
        edgecolors="#e0f2fe",
        linewidth=1,
        ax=ax,
        label="Shots (non-goal)",
    )
    pitch.scatter(
        goals["x"],
        goals["y"],
        s=(goals["shot_statsbomb_xg"] * 600) + 80,
        c="#22c55e",
        marker="*",
        edgecolors="#ffffff",
        linewidth=1.2,
        ax=ax,
        label="Goals",
        zorder=5,
    )

    ax.set_title(
        "Argentina Open-Play & Set-Piece Shots (excl. Shootout)\n2022 FIFA World Cup Final | Data: StatsBomb",
        color="#f8fafc",
        fontsize=13,
        fontweight="bold",
        pad=14,
    )
    ax.legend(
        loc="lower left",
        facecolor="#1e293b",
        edgecolor="#334155",
        labelcolor="#f8fafc",
    )

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    out_path = OUTPUT_DIR / "wc2022_final_argentina_shotmap.png"
    plt.savefig(out_path, dpi=200, bbox_inches="tight", facecolor=fig.get_facecolor())
    plt.close(fig)
    return out_path


if __name__ == "__main__":
    saved_path = generate_world_cup_final_shotmap()
    print(f"Shot map saved to: {saved_path}")
