"""
Generate a dark-mode UEFA Champions League (Nov 24, 2026) match intelligence card
by querying OpenScout Rust Core's live match endpoint directly.
"""

from pathlib import Path
import matplotlib.pyplot as plt
import numpy as np
import requests

API_URL = "http://localhost:8080/api/v1/live/uefa.champions/matches/401915298"
OUTPUT_PATH = (
    Path(__file__).resolve().parent.parent.parent
    / "docs"
    / "images"
    / "ucl_nov2026_rma_psv_analysis.png"
)


def build_live_card() -> Path:
    data = requests.get(API_URL, timeout=20).json()
    teams = data["teams"]
    home = next(t for t in teams if t["home_away"] == "home")
    away = next(t for t in teams if t["home_away"] == "away")

    fig, (ax1, ax2) = plt.subplots(
        1, 2, figsize=(13.8, 6.6), gridspec_kw={"width_ratios": [1, 1.25]}
    )
    fig.patch.set_facecolor("#0b1120")

    # Panel 1: Team Tactical Comparison
    ax1.set_facecolor("#0f172a")
    metrics = [
        ("Possession %", "possessionPct"),
        ("Total Shots", "totalShots"),
        ("Shots on Target", "shotsOnTarget"),
        ("Accurate Passes / 10", "accuratePasses"),
        ("Accurate Long Balls", "accurateLongBalls"),
        ("Effective Tackles", "effectiveTackles"),
    ]
    labels = [m[0] for m in reversed(metrics)]
    h_vals = []
    a_vals = []
    for _, key in reversed(metrics):
        hv = float(home["stats"].get(key, 0))
        av = float(away["stats"].get(key, 0))
        if key == "accuratePasses":
            hv /= 10.0
            av /= 10.0
        h_vals.append(hv)
        a_vals.append(av)

    y = np.arange(len(labels))
    bar_h = 0.36
    b1 = ax1.barh(y + bar_h / 2, h_vals, height=bar_h, color="#f8fafc", label=f"{home['team_name']} ({home['score']})")
    b2 = ax1.barh(y - bar_h / 2, a_vals, height=bar_h, color="#ef4444", label=f"{away['team_name']} ({away['score']})")

    ax1.set_yticks(y)
    ax1.set_yticklabels(labels, color="#e2e8f0", fontsize=10, fontweight="bold")
    ax1.set_title("Team Control & Shot Volume", color="#f8fafc", fontsize=12, fontweight="bold", pad=10)
    ax1.set_xlim(0, max(h_vals + a_vals) * 1.22)
    ax1.grid(axis="x", color="#1e293b", linestyle="--", alpha=0.7)
    ax1.tick_params(colors="#94a3b8")
    for spine in ax1.spines.values():
        spine.set_color("#1e293b")

    for idx, (bar_home, bar_away) in enumerate(zip(b1, b2)):
        key = list(reversed(metrics))[idx][1]
        h_raw = home["stats"].get(key, "0")
        a_raw = away["stats"].get(key, "0")
        ax1.text(bar_home.get_width() + 1.1, bar_home.get_y() + bar_home.get_height() / 2, h_raw, va="center", color="#f8fafc", fontsize=9, fontweight="bold")
        ax1.text(bar_away.get_width() + 1.1, bar_away.get_y() + bar_away.get_height() / 2, a_raw, va="center", color="#fca5a5", fontsize=9, fontweight="bold")

    ax1.legend(loc="lower right", facecolor="#1e293b", edgecolor="#334155", labelcolor="#f8fafc")

    # Panel 2: Top Player Shot & Goal Threat
    ax2.set_facecolor("#0f172a")
    top_players = [p for p in data["top_players"] if p["shots"] > 0][:8]
    top_players_rev = list(reversed(top_players))

    p_names = [
        f"{p['player_name']} ({'RMA' if 'Madrid' in p['team_name'] else 'PSV'})"
        for p in top_players_rev
    ]
    shots = [p["shots"] for p in top_players_rev]
    sot = [p["shots_on_target"] for p in top_players_rev]
    goals = [p["goals"] for p in top_players_rev]
    assists = [p["assists"] for p in top_players_rev]

    y2 = np.arange(len(p_names))
    ax2.barh(y2, shots, height=0.56, color="#334155", label="Total Shots")
    ax2.barh(y2, sot, height=0.56, color="#38bdf8", label="Shots on Target")

    ax2.set_yticks(y2)
    ax2.set_yticklabels(p_names, color="#e2e8f0", fontsize=10, fontweight="bold")
    ax2.set_xlabel("Shot Attempts", color="#94a3b8", fontsize=10)
    ax2.set_title("Player Shot Efficiency & Goal Contributions", color="#f8fafc", fontsize=12, fontweight="bold", pad=10)
    ax2.set_xlim(0, max(shots) + 3.2)
    ax2.grid(axis="x", color="#1e293b", linestyle="--", alpha=0.7)
    ax2.tick_params(colors="#94a3b8")
    for spine in ax2.spines.values():
        spine.set_color("#1e293b")

    for idx, (s, on_t, g, a) in enumerate(zip(shots, sot, goals, assists)):
        tags = []
        if g > 0:
            tags.append(f"{g}G")
        if a > 0:
            tags.append(f"{a}A")
        tag_str = f"  [{' + '.join(tags)}]" if tags else ""
        ax2.text(
            s + 0.18,
            y2[idx],
            f"{on_t}/{s} On Target{tag_str}",
            va="center",
            color="#f8fafc" if tags else "#cbd5e1",
            fontsize=9.2,
            fontweight="bold" if tags else "normal",
        )

    ax2.legend(loc="lower right", facecolor="#1e293b", edgecolor="#334155", labelcolor="#f8fafc")

    fig.suptitle(
        f"OpenScout Core (Rust / Axum) — UCL Live Match Breakdown: {home['team_name']} {home['score']} - {away['score']} {away['team_name']} (Nov 24, 2026)",
        color="#f8fafc",
        fontsize=13.5,
        fontweight="bold",
        y=0.97,
    )
    fig.text(
        0.5,
        0.015,
        "Live Data via OpenScout Rust Core (/api/v1/live/uefa.champions/matches/401915298)  |  Built with Axum + Tokio + serde + Matplotlib",
        ha="center",
        color="#64748b",
        fontsize=9.5,
    )

    plt.tight_layout(rect=[0, 0.04, 1, 0.92])
    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    plt.savefig(OUTPUT_PATH, dpi=220, facecolor=fig.get_facecolor())
    plt.close(fig)
    return OUTPUT_PATH


if __name__ == "__main__":
    print("Saved live UCL card to:", build_live_card())
