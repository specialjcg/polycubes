"""Draws the two histograms of the aggregation campaigns.

Usage, from the root of the repository:
    python3 docs/agregation/figure.py docs/agregation/resultats.png docs/agregation/resultats.svg
"""
import csv
import sys

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import matplotlib.ticker

BLUE, ORANGE, AQUA = "#2a78d6", "#eb6834", "#1baf7a"
SURFACE, INK, SECONDARY, MUTED = "#fcfcfb", "#0b0b0b", "#52514e", "#898781"

def rows(path):
    return list(csv.DictReader(open(path)))

polycubes = rows("docs/agregation/mesures.csv")
lone_cells = rows("docs/agregation/mesures-cubes-isoles.csv")

plt.rcParams.update({"font.family": "DejaVu Sans", "font.size": 11, "text.color": INK,
                     "axes.edgecolor": MUTED, "axes.labelcolor": SECONDARY,
                     "xtick.color": MUTED, "ytick.color": MUTED, "axes.titlecolor": INK})
figure, (dimension_axes, loop_axes) = plt.subplots(2, 1, figsize=(10, 9.4), facecolor=SURFACE)

def style(axes):
    axes.set_facecolor(SURFACE)
    for side in ("top", "right", "left"):
        axes.spines[side].set_visible(False)
    axes.grid(axis="y", color=MUTED, alpha=0.2, linewidth=0.8)
    axes.set_axisbelow(True)
    axes.tick_params(length=0)

def histogram(axes, values, bins, colour, label):
    axes.hist(values, bins=bins, color=colour, alpha=0.55, label=label, edgecolor=SURFACE, linewidth=0.6)

# Panel 1: fractal dimension
style(dimension_axes)
bins = [1.5 + 0.03 * step for step in range(58)]
histogram(dimension_axes, [float(row["dimension"]) for row in polycubes], bins, BLUE, "Polycubes de 5 cubes (moy. 2,25)")
histogram(dimension_axes, [float(row["dimension"]) for row in lone_cells], bins, ORANGE, "Cubes isolés (moy. 2,42)")
top = dimension_axes.get_ylim()[1]
references = [(1.82, "Suie\n1,82", "left"), (2.50, "Référence\n2,50", "left"), (2.70, "Brocoli\n≈ 2,7", "low"), (2.80, "Chou-fleur\n≈ 2,8", "left"), (3.00, "Boule\npleine 3", "left")]
for value, name, side in references:
    dimension_axes.axvline(value, color=SECONDARY, linewidth=1, linestyle=(0, (4, 3)))
    # The broccoli label sits a row below, between two close neighbours.
    height = top * (0.97 if side == "low" else 1.15)
    dimension_axes.text(value + 0.012, height, name, color=SECONDARY, fontsize=9.5, va="top", ha="left")
dimension_axes.set_ylim(0, top * 1.18)
dimension_axes.set_xlim(1.5, 3.25)
dimension_axes.set_xlabel("Dimension fractale de l'agrégat")
dimension_axes.set_ylabel("Nombre d'agrégats")
dimension_axes.set_title("Des polycubes entiers donnent des agrégats plus aérés que des cubes isolés", loc="left", fontsize=13, fontweight="bold", pad=36)
dimension_axes.legend(loc="lower left", bbox_to_anchor=(-0.01, 1.0), ncol=2, frameon=False, labelcolor=INK)
dimension_axes.xaxis.set_major_formatter(matplotlib.ticker.FuncFormatter(lambda value, _: f"{value:.1f}".replace(".", ",")))

# Panel 2: loops
style(loop_axes)
loop_bins = list(range(0, 212, 4))
histogram(loop_axes, [int(row["loops"]) for row in polycubes if row["seed_loops"] == "0"], loop_bins, BLUE, "Polycubes sans boucle (moy. 47)")
histogram(loop_axes, [int(row["loops"]) for row in lone_cells], loop_bins, ORANGE, "Cubes isolés (moy. 75)")
histogram(loop_axes, [int(row["loops"]) for row in polycubes if row["seed_loops"] == "1"], loop_bins, AQUA, "Polycubes à une boucle (moy. 170)")
loop_axes.set_xlim(0, 210)
loop_axes.set_xlabel("Nombre de boucles de l'agrégat")
loop_axes.set_ylabel("Nombre d'agrégats")
loop_axes.set_title("Le nombre de boucles dépend d'abord de ce qui s'agrège", loc="left", fontsize=13, fontweight="bold", pad=36)
loop_axes.legend(loc="lower left", bbox_to_anchor=(-0.01, 1.0), ncol=3, frameon=False, labelcolor=INK, columnspacing=1.2)

figure.text(0.012, 0.012, "20 000 agrégats de 600 cubes, agrégation limitée par diffusion sur grille cubique. Valeurs naturelles : voir resultats.md.", color=MUTED, fontsize=9)
figure.subplots_adjust(left=0.085, right=0.975, top=0.915, bottom=0.085, hspace=0.42)
for path in sys.argv[1:]:
    figure.savefig(path, dpi=140, facecolor=SURFACE)
