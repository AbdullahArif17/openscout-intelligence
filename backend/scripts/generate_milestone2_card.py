"""
Milestone 2 Visual Card: 2022 World Cup Final (Match 3869685)
Generated from OpenScout Rust Core (Axum + Tokio + serde) player summary metrics.
"""

from pathlib import Path
import matplotlib.pyplot as plt
import numpy as np

OUTPUT_PATH = (
    Path(__file__).resolve().parent.parent.parent
    / "docs"
    / "images"
    / "wc2022_rust_analytics_card.png"
)


def build_card() -> Path:
    players = [
        ("K. Mbappé", "France", 1.783, 3, 70.4, 27),
        ("L. Messi", "Argentina", 1.455, 2, 81.7, 60),
        ("L. Martínez", "Argentina", 0.584, 0, 71.4, 7),
        ("Á. Di María", "Argentina", 0.410, 1, 86.2, 29),
        ("R. Kolo Muani", "France", 0.374, 0, 73.3, 15),
        ("R. De Paul", "Argentina", 0.103, 0, 83.8, 68),
        ("A. Rabiot", "France", 0.093, 0, 84.3, 51),
        ("E. Fernández", "Argentina", 0.072, 0, 84.0, 94),
    ]

    names = [f"{p[0]} ({p[1][:3].upper()})" for p in reversed(players)]
    xg_vals = [p[2] for p in reversed(players)]
    goals = [p[3] for p in reversed(players)]
    pass_acc = [p[4] for p in reversed(players)]
    pass_att = [p[5] for p in reversed(players)]
    teams = [p[1] for p in reversed(players)]

    colors = ["#38bdf8" if t == "Argentina" else "#f43f5e" for t in teams]

    fig, (ax1, ax2) = plt.subplots(
        1, 2, figsize=(13.5, 6.5), gridspec_kw={"width_ratios": [1.15, 1]}
    )
    fig.patch.set_facecolor("#0b1120")

    # Panel 1: Total xG & Goals
    ax1.set_facecolor("#0f172a")
    y_pos = np.arange(len(names))
    bars = ax1.barh(y_pos, xg_vals, color=colors, height=0.58, alpha=0.9)
    ax1.set_yticks(y_pos)
    ax1.set_yticklabels(names, color="#e2e8f0", fontsize=10.5, fontweight="bold")
    ax1.set_xlabel("Expected Goals (xG) — excl. Shootout", color="#94a3b8", fontsize=10)
    ax1.set_title("Shot Threat (Total xG & Goals)", color="#f8fafc", fontsize=12, fontweight="bold", pad=10)
    ax1.set_xlim(0, 2.15)
    ax1.grid(axis="x", color="#1e293b", linestyle="--", alpha=0.7)
    ax1.tick_params(colors="#94a3b8")
    for spine in ax1.spines.values():
        spine.set_color("#1e293b")

    for bar, g, xg in zip(bars, goals, xg_vals):
        label = f"{xg:.3f} xG" + (f"  ({g}G)" if g > 0 else "")
        ax1.text(
            bar.get_width() + 0.04,
            bar.get_y() + bar.get_height() / 2,
            label,
            va="center",
            color="#f8fafc",
            fontsize=9.5,
            fontweight="semibold",
        )

    # Panel 2: Passing Volume vs Accuracy
    ax2.set_facecolor("#0f172a")
    sizes = [max(pa * 6.5, 60) for pa in pass_att]
    ax2.scatter(pass_att, pass_acc, s=sizes, c=colors, alpha=0.85, edgecolors="#f8fafc", linewidth=1.2)

    # Custom (dx, dy, label) offsets so zero labels overlap
    label_specs = {
        "K. Mbappé": (0, -2.1, "Mbappé (70.4%)"),
        "L. Messi": (2, -2.4, "Messi (81.7%)"),
        "L. Martínez": (0, -2.0, "Martínez (71.4%)"),
        "Á. Di María": (0, 1.5, "Di María (86.2%)"),
        "R. Kolo Muani": (2, 1.6, "Kolo Muani (73.3%)"),
        "R. De Paul": (2, 1.9, "De Paul (83.8%)"),
        "A. Rabiot": (-5, 1.7, "Rabiot (84.3%)"),
        "E. Fernández": (-3, -2.6, "Fernández (84.0%)"),
    }

    for p in players:
        dx, dy, lbl = label_specs[p[0]]
        ax2.text(
            p[5] + dx,
            p[4] + dy,
            lbl,
            color="#e2e8f0",
            fontsize=8.8,
            ha="center",
            fontweight="bold",
        )

    ax2.set_xlabel("Passes Attempted", color="#94a3b8", fontsize=10)
    ax2.set_ylabel("Pass Completion %", color="#94a3b8", fontsize=10)
    ax2.set_title("Distribution Volume vs. Accuracy", color="#f8fafc", fontsize=12, fontweight="bold", pad=10)
    ax2.set_xlim(-4, 108)
    ax2.set_ylim(64, 92)
    ax2.grid(color="#1e293b", linestyle="--", alpha=0.7)
    ax2.tick_params(colors="#94a3b8")
    for spine in ax2.spines.values():
        spine.set_color("#1e293b")

    fig.suptitle(
        "OpenScout Core (Rust / Axum) — 2022 World Cup Final Player Impact (4,407 Events)",
        color="#f8fafc",
        fontsize=14.5,
        fontweight="bold",
        y=0.98,
    )
    fig.text(
        0.5,
        0.015,
        "Blue: Argentina  |  Red: France  |  API: Rust (Axum + Tokio + serde)  |  Data: Hudl/StatsBomb Open Data (Match 3869685)",
        ha="center",
        color="#64748b",
        fontsize=9.5,
    )

    plt.tight_layout(rect=[0, 0.04, 1, 0.93])
    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    plt.savefig(OUTPUT_PATH, dpi=220, facecolor=fig.get_facecolor())
    plt.close(fig)
    return OUTPUT_PATH


if __name__ == "__main__":
    out = build_card()
    print(f"Saved card to: {out}")
