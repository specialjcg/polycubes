"""Draws which shapes live in a long game of the life of polycubes.

Usage, from the root of the repository:
    python3 docs/vie/figure.py docs/vie/formes.png docs/vie/formes.svg
"""
import csv
import sys
from collections import defaultdict

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import matplotlib.ticker

COLOURS = ["#2a78d6", "#eb6834", "#1baf7a", "#eda100", "#e87ba4"]
SURFACE, INK, SECONDARY, MUTED = "#fcfcfb", "#0b0b0b", "#52514e", "#898781"
SHOWN_SHAPE_COUNT = len(COLOURS)
# Shares are averaged over that many periods of 25 generations: a period
# alone swings too much for a trend to show.
SMOOTHED_PERIOD_COUNT = 8

rows = list(csv.DictReader(open("docs/vie/formes.csv")))
periods = sorted({int(row["period_start"]) for row in rows})
share_by_shape = defaultdict(dict)
description_by_shape = {}
for row in rows:
    share_by_shape[row["shape"]][int(row["period_start"])] = float(row["share"])
    loops = int(row["loops"])
    description_by_shape[row["shape"]] = (
        f"boîte {row['box_side_1']}×{row['box_side_2']}×{row['box_side_3']}, "
        + ("sans boucle" if loops == 0 else "une boucle" if loops == 1 else f"{loops} boucles")
    )

def mean_share(shape):
    return sum(share_by_shape[shape].values()) / len(periods)

shown_shapes = sorted(share_by_shape, key=mean_share, reverse=True)[:SHOWN_SHAPE_COUNT]

def smoothed(shape):
    shares = [share_by_shape[shape].get(period, 0.0) for period in periods]
    return [
        sum(shares[max(0, rank - SMOOTHED_PERIOD_COUNT + 1): rank + 1])
        / len(shares[max(0, rank - SMOOTHED_PERIOD_COUNT + 1): rank + 1])
        for rank in range(len(shares))
    ]

plt.rcParams.update({"font.family": "DejaVu Sans", "font.size": 11, "text.color": INK,
                     "axes.edgecolor": MUTED, "axes.labelcolor": SECONDARY,
                     "xtick.color": MUTED, "ytick.color": MUTED, "axes.titlecolor": INK})
figure, axes = plt.subplots(figsize=(10, 5.6), facecolor=SURFACE)
axes.set_facecolor(SURFACE)
for side in ("top", "right", "left"):
    axes.spines[side].set_visible(False)
axes.grid(axis="y", color=MUTED, alpha=0.2, linewidth=0.8)
axes.set_axisbelow(True)
axes.tick_params(length=0)

# Two shapes may share a box and a loop count: a letter tells them apart.
seen_descriptions = defaultdict(int)
for colour, shape in zip(COLOURS, shown_shapes):
    description = description_by_shape[shape]
    seen_descriptions[description] += 1
    twin_count = sum(1 for other in shown_shapes if description_by_shape[other] == description)
    if twin_count > 1:
        description += f" ({'abcde'[seen_descriptions[description_by_shape[shape]] - 1]})"
    axes.plot(periods, [100 * share for share in smoothed(shape)], color=colour, linewidth=2,
              label=f"{description} : {100 * mean_share(shape):.0f} % en moyenne".replace(".", ","))

axes.set_xlim(0, periods[-1])
axes.set_ylim(0, None)
axes.yaxis.set_major_formatter(matplotlib.ticker.FuncFormatter(lambda value, _: f"{value:.0f} %"))
axes.xaxis.set_major_formatter(matplotlib.ticker.FuncFormatter(lambda value, _: f"{value:,.0f}".replace(",", " ")))
axes.set_xlabel("Génération")
axes.set_ylabel("Part des polycubes vivants")
axes.set_title("Aucune forme ne l'emporte pour de bon : le classement dérive sans fin",
               loc="left", fontsize=13, fontweight="bold", pad=58)
axes.legend(loc="lower left", bbox_to_anchor=(-0.01, 1.0), ncol=2, frameon=False, labelcolor=INK,
            fontsize=10, columnspacing=1.4)
figure.text(0.012, 0.014,
            "Une partie de " + f"{periods[-1]:,}".replace(",", " ") + " générations, monde de 11 cases de côté.\n"
            + f"Les cinq formes les plus présentes sur les 29 de 5 cubes ; moyenne glissante sur {SMOOTHED_PERIOD_COUNT * 25} générations.",
            color=MUTED, fontsize=9)
figure.subplots_adjust(left=0.085, right=0.975, top=0.80, bottom=0.17)
for path in sys.argv[1:]:
    figure.savefig(path, dpi=140, facecolor=SURFACE)
