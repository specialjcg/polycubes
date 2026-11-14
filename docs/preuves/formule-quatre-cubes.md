# La formule fermée des 4 derniers cubes

À un nœud de l'énumération, le code compte les façons d'ajouter 1, 2, 3 ou 4 cubes sans les visiter, à partir de sommes relevées sur la frontière (`FrontierSums`, `crates/polycubes-core/src/redelmeier.rs`). Cette note prouve que ces formules comptent bien les extensions.

## Cadre

On reprend le modèle de la [preuve du tirage uniforme](tirage-uniforme.md) : un nœud $(P, Q, F)$, avec $Q$ les cases en attente et $F$ les cases libres. D'après le lemme de Redelmeier, les polycubes de $|P| + k$ cubes sous ce nœud sont en bijection avec les **extensions** de taille $k$ : les ensembles $A \subseteq Q \cup F$ de $k$ cases dont chacune est reliée, dans $A$, à une case de $A \cap Q$.

**Reformulation.** Une case de $Q$ dans $A$ ne demande rien. Une case de $F$ dans $A$ doit être reliée à une case de $A \cap Q$ ; en coupant le chemin à la première case de $Q$ rencontrée, il ne passe avant que par des cases de $F$. Donc :

> $A$ est une extension si et seulement si $A \cap Q \neq \varnothing$ et chaque case de $A \cap F$ est reliée, **à travers des cases de $A \cap F$**, à une case de $A \cap F$ voisine d'une case de $A \cap Q$ (elle-même comprise).

Les voisinages entre cases de $Q$ ne jouent aucun rôle.

**Notations.** $L = |Q|$. Pour $u \in Q$, $d(u)$ est son nombre de voisines libres. Pour $f \in F$, $m(f)$ est son nombre de voisines en attente et $e(f)$ son nombre de voisines libres. Une **arête** est un couple $(u, f)$ avec $u \in Q$, $f \in F$ voisines. Un **chemin** est un triplet $(u, f, g)$ avec $(u, f)$ une arête et $g \in F$ voisine de $f$.

**Deux faits sur le réseau cubique.**

- (B) *Biparti.* Deux voisines d'une même case ne sont jamais voisines entre elles. Plus généralement, deux cases à distance paire ne sont pas voisines.
- (C) *Voisines communes.* Deux cases distinctes ont 0, 1 ou 2 voisines communes ; elles en ont 2 exactement quand elles sont les coins opposés d'un carré unité, et les voisines communes sont alors les deux autres coins.

**Une identité.** Dans un ensemble de $L$ éléments dont $a$ sont marqués, le nombre de paires contenant au moins un élément marqué est

$$\binom{L}{2} - \binom{L - a}{2} = a(L - 1) - \binom{a}{2}. \tag{I}$$

## Les sommes

| Symbole | Définition | Champ de `FrontierSums` |
|---|---|---|
| $L$ | $\lvert Q \rvert$ | `untried_count` |
| $D$ | nombre d'arêtes $= \sum_u d(u) = \sum_f m(f)$ | `edge_count` |
| $T$ | $\sum_u \binom{d(u)}{2}$ | `free_pairs_around_untried` |
| $T_3$ | $\sum_u \binom{d(u)}{3}$ | `free_triples_around_untried` |
| $S$ | $\sum_f \binom{m(f)}{2}$ | `twice_untried_pairs_around_free / 2` |
| $S_3$ | $\sum_f \binom{m(f)}{3}$ | `thrice_untried_triples_around_free / 3` |
| $\Pi$ | nombre de chemins $= \sum_{(u,f)} e(f)$ | `path_count` |
| $\Phi$ | $\sum_{(u,f)} \binom{e(f)}{2}$ | `forked_path_count` |
| $R$ | $\sum_{(u,f)} e(f)\,(m(f) - 1)$ | `twice_doubly_rooted_path_count` |
| $E$ | $\sum_{(u,f)} d(u)\, m(f)$ | `edge_degree_product_total` |
| $X$ | $\sum_{(u,f)} d(u)\, e(f)$ | `edge_free_degree_product_total` |
| $Y$ | $\sum_{(u,f,g)} e(g)$ | `path_end_free_neighbour_total` |
| $W$ | $\sum_{(u,f,g)} m(g)$ | `path_end_untried_neighbour_total` |
| $C_1$ | carrés unité avec un coin en attente et trois coins libres | `twice_single_untried_square_count / 2` |
| $C_2$ | carrés unité avec deux coins opposés en attente et deux coins libres | `four_times_double_untried_square_count / 4` |

Les sommes sur $(u, f)$ portent sur les arêtes, celles sur $(u, f, g)$ sur les chemins. Les facteurs 2, 3 et 4 des champs viennent du balayage : sommer $m(f) - 1$ sur les arêtes donne $\sum_f m(f)(m(f) - 1) = 2S$ ; sommer $\binom{m(f) - 1}{2}$ donne $3 S_3$ ; un carré à un coin en attente $u$ est vu une fois par chemin tournant de $u$ au coin opposé, soit 2 fois ; un carré à deux coins en attente est vu pour chacun des 2 coins en attente et chacun des 2 coins libres, soit 4 fois.

On note $N_j^k$ le nombre d'extensions de $k$ cases dont $j$ sont libres (et $k - j$ en attente). Une extension a au moins une case en attente : $0 \leq j \leq k - 1$.

## Un cube

$$N_0^1 = L.$$

## Deux cubes

- $N_0^2 = \binom{L}{2}$ : toute paire de cases en attente.
- $N_1^2 = D$ : $\{u, f\}$ est une extension si et seulement si $f$ est voisine de $u$.

## Trois cubes

- $N_0^3 = \binom{L}{3}$.
- $N_1^3 = (L - 1) D - S$. L'ensemble $\{u, u', f\}$ est une extension si et seulement si $f$ est voisine de $u$ ou de $u'$. Pour $f$ fixée, l'identité (I) avec $a = m(f)$ compte $m(f)(L - 1) - \binom{m(f)}{2}$ paires ; on somme sur $f$.
- $N_2^3 = T + \Pi$. Pour $\{u, f, g\}$, deux cas disjoints.
  - $f$ et $g$ voisines de $u$ : $\binom{d(u)}{2}$ paires, soit $T$ au total.
  - Une seule, disons $f$, voisine de $u$ : $g$ doit être voisine de $f$. C'est un chemin $(u, f, g)$. Réciproquement, dans un chemin, $g$ n'est pas voisine de $u$ d'après (B), donc chaque chemin donne une extension de ce cas, et l'ensemble détermine le chemin. Total $\Pi$.

## Quatre cubes

### Aucune case libre

$$N_0^4 = \binom{L}{4}.$$

### Une case libre

$\{u, u', u'', f\}$ est une extension si et seulement si $f$ est voisine d'au moins une des trois. Pour $f$ fixée avec $a = m(f)$, le nombre de triplets de $Q$ contenant au moins une des $a$ voisines est, par inclusion-exclusion, $a\binom{L - 1}{2} - \binom{a}{2}(L - 2) + \binom{a}{3}$. En sommant :

$$N_1^4 = \binom{L - 1}{2} D - (L - 2)\, S + S_3.$$

### Deux cases libres

$\{u, u', f, g\}$. Appelons une case libre **accrochée** si elle est voisine de $u$ ou de $u'$. Si aucune n'est accrochée, ce n'est pas une extension. Deux cas disjoints restent.

**(i) $f$ et $g$ accrochées.** C'est toujours une extension. Pour une paire $\{f, g\}$ de cases libres, posons $a = m(f)$, $b = m(g)$ et $c$ leur nombre de voisines en attente communes. Le nombre de paires $\{u, u'\}$ touchant à la fois les voisines de $f$ et celles de $g$ est

$$\binom{L}{2} - \binom{L - a}{2} - \binom{L - b}{2} + \binom{L - a - b + c}{2} \;=\; ab - c(a + b) + \frac{c\,(2L - 1 + c)}{2}.$$

(L'égalité est une identité polynomiale ; vérifiée aussi numériquement pour $L \leq 8$.) On somme chaque terme sur les paires $\{f, g\}$.

- $\sum ab = \frac{1}{2}\bigl(D^2 - \sum_f m(f)^2\bigr)$, et $\sum_f m(f)^2 = 2S + D$.
- $\sum c\,(a + b)$ : $c$ compte les $u$ voisines des deux, donc $\sum_{\{f,g\}} c\,(a + b) = \sum_u \sum_{\{f, g\} \subseteq N(u)} (m(f) + m(g)) = \sum_{(u,f)} (d(u) - 1)\, m(f) = E - (2S + D)$.
- $\sum \frac{c(2L - 1 + c)}{2} = L \sum c + \sum \binom{c}{2}$. D'une part $\sum c = T$. D'autre part, d'après (C), $c \leq 2$, et $c = 2$ exactement quand $f$ et $g$ sont les coins libres opposés d'un carré dont les deux autres coins sont en attente : $\sum \binom{c}{2} = C_2$.

Total du cas (i) : $\dfrac{D^2 - 2S - D}{2} - (E - 2S - D) + L\,T + C_2$.

**(ii) Une seule accrochée, disons $f$.** Alors $g$ doit être voisine de $f$. On compte, pour chaque couple ordonné $(f, g)$ de cases libres voisines, les paires $\{u, u'\}$ qui touchent les voisines de $f$ et évitent celles de $g$. D'après (B), $f$ et $g$ n'ont aucune voisine commune. Parmi les $L - m(g)$ cases en attente non voisines de $g$, $m(f)$ sont voisines de $f$ ; l'identité (I) donne $m(f)(L - m(g) - 1) - \binom{m(f)}{2}$. L'ensemble détermine $(f, g)$, donc pas de double compte. En sommant sur les couples $(f, g)$ :

- $\sum_{(f,g)} m(f) = \Pi$ (chaque couple compté une fois par voisine en attente de $f$) ;
- $\sum_{(f,g)} m(f)\, m(g) = W$ ;
- $\sum_{(f,g)} \binom{m(f)}{2} = \frac{1}{2}\sum_{(u,f)} e(f)(m(f) - 1) = R / 2$.

Total du cas (ii) : $(L - 1)\,\Pi - R/2 - W$.

$$N_2^4 = \frac{D^2 - 2S - D}{2} - (E - 2S - D) + L\,T + C_2 + (L - 1)\,\Pi - \frac{R}{2} - W.$$

### Trois cases libres

$\{u, f, g, h\}$. Soit $\rho$ le nombre de cases libres voisines de $u$. Si $\rho = 0$, ce n'est pas une extension. Trois cas disjoints.

**$\rho = 3$.** Toujours une extension : $T_3$.

**$\rho = 2$** : $f, g$ voisines de $u$, $h$ non. Alors $h$ doit être voisine de $f$ ou de $g$ ; et toute voisine libre de $f$ ou de $g$ est non voisine de $u$ d'après (B). Pour une paire $\{f, g\}$ de voisines libres de $u$, le nombre de $h$ est $e(f) + e(g) - \lvert N_F(f) \cap N_F(g) \rvert$. D'après (C), $f$ et $g$ ont pour voisines communes $u$, qui n'est pas libre, et le quatrième coin du carré quand elles sont perpendiculaires. L'intersection vaut donc 1 si ce quatrième coin existe et est libre, 0 sinon. En sommant sur $u$ et $\{f, g\}$ :

- $\sum (e(f) + e(g)) = \sum_{(u,f)} (d(u) - 1)\, e(f) = X - \Pi$ ;
- le terme correctif compte les carrés ayant $u$ pour seul coin en attente et trois coins libres : $C_1$.

Total : $X - \Pi - C_1$.

**$\rho = 1$** : $f$ voisine de $u$, $g$ et $h$ non. Deux sous-cas disjoints, selon le nombre de cases parmi $g, h$ voisines de $f$ (aucune n'est exclue car il faut les relier à $f$).

- *Fourche* : $g$ et $h$ voisines de $f$. D'après (B), aucune n'est voisine de $u$. Total $\sum_{(u,f)} \binom{e(f)}{2} = \Phi$.
- *Chaîne* : $g$ voisine de $f$, $h$ voisine de $g$ mais pas de $f$. Il faut $h \neq f$ et $h$ non voisine de $u$ (sinon $\rho = 2$) ; $h$ n'est pas voisine de $f$ d'après (B). Pour un chemin $(u, f, g)$, les voisines libres de $g$ sont au nombre de $e(g)$ ; on retire $f$, et on retire la voisine commune de $u$ et $g$ autre que $f$ quand elle existe et est libre. D'après (C), elle existe quand le chemin tourne : c'est le quatrième coin du carré. Un carré à un coin en attente et trois coins libres est retiré une fois par chemin tournant, soit 2 fois. L'ensemble détermine $f$, puis $g$, puis $h$. Total $Y - \Pi - 2\,C_1$.

$$N_3^4 = T_3 + (X - \Pi - C_1) + \Phi + (Y - \Pi - 2\,C_1).$$

## Récapitulatif

| $k$ | Nombre d'extensions |
|---|---|
| 1 | $L$ |
| 2 | $\binom{L}{2} + D$ |
| 3 | $\binom{L}{3} + (L - 1)D - S + T + \Pi$ |
| 4 | $\binom{L}{4} + N_1^4 + N_2^4 + N_3^4$ |

Ce sont les expressions de `extension_counts_by_free_cell_count` : ligne $k - 1$, colonne $j$.

## Ce que la preuve suppose

- **Les sommes sont bien relevées.** Le balayage `frontier_sums` et les compteurs $d$, $m$, $e$ tenus à jour dans chaque case (`free_neighbour_count`, `untried_neighbour_count`) ne sont pas prouvés ici.
- **La place ne manque pas.** Les cases à moins de 4 pas de la frontière doivent être dans le réseau ; le code n'applique la formule que quand il reste exactement 4 cubes à poser, sur un réseau dimensionné pour la taille maximale.
- **Le code suit le modèle** $(P, Q, F)$.

## Vérification par le code

Ces tests couvrent ensemble la formule, le balayage et les compteurs.

- Formule contre visite, terme à terme ($N_j^k$ pour tous $k, j$), sur **tous** les polycubes de 1 à 4 cubes, soit 105 nœuds (`formula_matches_the_visit_on_every_polycube_of_at_most_four_cells`).
- Même comparaison sur 72 polycubes de 1 à 6 cubes pris par descentes pseudo-aléatoires (`formula_counts_what_the_visit_finds_term_by_term`). Les deux tests sont dans `crates/polycubes-core/tests/frontier_extensions.rs`.
- Totaux : les nombres de polycubes fixes obtenus avec la formule coïncident avec OEIS A001931 jusqu'à $n = 16$ ; à cette taille la formule est appliquée à chaque nœud de 12 cubes.

Un détail d'histoire : la première version comptait pour $C_2$ les carrés à deux coins en attente **adjacents** au lieu d'opposés, et donnait 3484 au lieu de 3481 à $n = 6$. Le fait (C) est exactement ce qui fixe le bon cas.
