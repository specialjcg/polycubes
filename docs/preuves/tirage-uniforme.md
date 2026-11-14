# Le tirage par rang est uniforme

Deux résultats. Le premier dit que l'énumération de Redelmeier visite chaque polycube fixe une fois et une seule ; la [preuve de la formule des 4 cubes](formule-quatre-cubes.md) s'appuie aussi dessus. Le second en déduit que `RankedPolycubes::polycube_at` est une bijection entre les rangs et les polycubes, donc qu'un rang uniforme donne un polycube uniforme.

Code concerné : `crates/polycubes-core/src/redelmeier.rs` (`extend`), `crates/polycubes-core/src/redelmeier/cursor.rs`, `crates/polycubes-sampling/src/ranked.rs`.

## Cadre

Une **case** est un point de $\mathbb{Z}^3$ ; deux cases sont **voisines** si elles diffèrent de 1 sur une seule coordonnée. Un **polycube fixe** de $n$ cubes est un ensemble connexe de $n$ cases, à translation près.

**Forme canonique.** On ordonne les cases par $(z, y, x)$ lexicographique. Chaque polycube fixe a un unique représentant dont la plus petite case est l'origine. Les cases **permises** sont celles qui ne précèdent pas l'origine. Un ensemble connexe de cases permises contenant l'origine a l'origine pour plus petite case. Les polycubes fixes de $n$ cubes sont donc en bijection avec les ensembles connexes de $n$ cases permises contenant l'origine.

**État d'un nœud.** Un nœud de l'arbre d'énumération est un triplet $(P, Q, F)$ :

- $P$, les cases posées ;
- $Q = (u_1, \dots, u_k)$, la file des cases en attente, distinctes ;
- $F$, les cases libres : permises, ni posées, ni en attente, ni écartées.

$P$, $Q$ et $F$ sont deux à deux disjoints. La racine est $(\varnothing, (o), F_0)$, avec $o$ l'origine et $F_0$ les cases permises autres que $o$.

**Enfants.** Le nœud $(P, Q, F)$ a $k$ enfants. Le $i$-ième pose $u_i$ :

$$P' = P \cup \{u_i\}, \qquad Q' = (u_{i+1}, \dots, u_k, v_1, \dots, v_r), \qquad F' = F \setminus N,$$

où $N = \{v_1, \dots, v_r\}$ est l'ensemble des voisines libres de $u_i$. Les cases $u_1, \dots, u_{i-1}$ ne sont ni dans $Q'$ ni dans $F'$ : elles sont écartées pour tout le sous-arbre. C'est ce que fait `extend` : la boucle ferme `u_i`, met en file ses voisines libres, descend avec la file commençant à `untried_index + 1`, et ne rouvre les cases qu'après la boucle.

## Lemme (Redelmeier)

Appelons **extension** du nœud $(P, Q, F)$ un ensemble fini non vide $A \subseteq Q \cup F$ dont chaque case est reliée, par un chemin de cases voisines contenu dans $A$, à une case de $A \cap Q$.

> **Lemme.** Les nœuds strictement sous $(P, Q, F)$ sont exactement les $P \cup A$, pour $A$ extension de ce nœud, et chaque extension y apparaît une fois et une seule.

*Preuve.* Par récurrence sur $|A|$, on montre que chaque extension $A$ est atteinte par un unique nœud du sous-arbre, et que tout nœud du sous-arbre est de cette forme.

Soit $A$ une extension et $i$ le plus petit indice tel que $u_i \in A$ ; il existe car $A \cap Q \neq \varnothing$.

*Seul l'enfant $i$ peut mener à $A$.* Un enfant $j < i$ pose $u_j \notin A$. Un enfant $j > i$ écarte $u_i$, qui ne sera plus jamais posée dans son sous-arbre.

*Sous l'enfant $i$.* Posons $A' = A \setminus \{u_i\}$. Si $A' = \varnothing$, l'enfant lui-même est $P \cup A$, et aucun nœud plus bas ne l'égale puisqu'ils ont plus de cases. Sinon, montrons que $A'$ est une extension de l'enfant $(P', Q', F')$.

- $A' \subseteq Q' \cup F'$ : $A$ évite $u_1, \dots, u_{i-1}$ par minimalité de $i$, et $Q' \cup F' = (Q \setminus \{u_1, \dots, u_i\}) \cup F$.
- Chaque case $a \in A'$ est reliée dans $A'$ à une case de $Q'$. Prenons un chemin $a = p_0, \dots, p_t$ dans $A$ vers une case de $Q$, et $s$ le premier indice avec $p_s \in Q$. Si $p_s \neq u_i$, alors $p_s \in Q'$ et $p_0, \dots, p_s$ évite $u_i$ : c'est un chemin dans $A'$. Si $p_s = u_i$, alors $s \geq 1$, et $p_{s-1}$ est une voisine de $u_i$ qui n'est pas dans $Q$, donc dans $F$ : elle est dans $N \subseteq Q'$, et $p_0, \dots, p_{s-1}$ convient.

Par hypothèse de récurrence, $A'$ est atteinte une fois et une seule sous l'enfant $i$, donc $A$ aussi.

*Réciproque.* Un nœud sous l'enfant $i$ s'écrit $P' \cup A'$ avec $A'$ vide ou extension de l'enfant. Alors $A = A' \cup \{u_i\}$ est une extension du parent : $A \subseteq Q \cup F$ ; et une case de $A'$ reliée à une case de $Q'$ l'est soit à un $u_j$ avec $j > i$, soit à une case de $N$, voisine de $u_i \in A \cap Q$. $\blacksquare$

**Corollaire.** À la racine, $Q = (o)$ : une extension est un ensemble connexe de cases permises contenant l'origine. Les nœuds de profondeur $n$ sont donc exactement les polycubes fixes de $n$ cubes, chacun une fois.

*Réseau fini.* Le code travaille sur un réseau borné, dont le bord est interdit. Un polycube de $n$ cubes contenant l'origine reste à distance au plus $n - 1$ de celle-ci ; le réseau est dimensionné pour (`Lattice::new`). Ce dimensionnement n'est pas prouvé ici, il est couvert par les tests contre OEIS.

## Théorème (tirage uniforme)

Fixons $n$. Pour un nœud $v$ de profondeur au plus $n$, notons $s(v)$ le nombre de nœuds de profondeur $n$ dans son sous-arbre, lui compris. Si $v$ est de profondeur $n$, $s(v) = 1$ ; sinon $s(v) = \sum_i s(c_i)$, somme sur ses enfants $c_1, \dots, c_k$.

`polycube_at(rang)` part de la racine avec $r = \text{rang}$. À un nœud de profondeur inférieure à $n$, il prend le premier enfant $c_i$ tel que $r < s(c_1) + \dots + s(c_i)$, remplace $r$ par $r - s(c_1) - \dots - s(c_{i-1})$ et descend. À la profondeur $n$, il rend le polycube du nœud.

> **Théorème.** Pour tout nœud $v$ de profondeur au plus $n$, la descente depuis $v$ est une bijection de $\{0, \dots, s(v) - 1\}$ sur les nœuds de profondeur $n$ sous $v$. En particulier `polycube_at` est une bijection de $\{0, \dots, A(n) - 1\}$ sur les polycubes fixes de $n$ cubes.

*Preuve.* Par récurrence descendante sur la profondeur. À la profondeur $n$, $s(v) = 1$ et le seul rang donne $v$. Sinon, les intervalles $I_i = [\,s(c_1) + \dots + s(c_{i-1}),\ s(c_1) + \dots + s(c_i)\,[$ partitionnent $\{0, \dots, s(v) - 1\}$ ; un enfant avec $s(c_i) = 0$ a un intervalle vide et n'est jamais choisi. La descente envoie $I_i$, décalé, sur $\{0, \dots, s(c_i) - 1\}$, que l'hypothèse de récurrence met en bijection avec les nœuds de profondeur $n$ sous $c_i$. Les sous-arbres des enfants sont disjoints et couvrent tout ce qui est sous $v$. Le corollaire du lemme identifie les nœuds de profondeur $n$ aux polycubes fixes. $\blacksquare$

**Conséquence.** Si le rang est uniforme sur $\{0, \dots, A(n) - 1\}$, chaque polycube fixe de $n$ cubes sort avec probabilité $1 / A(n)$.

## Ce que la preuve suppose

- **$s(v)$ est bien calculé.** `completion_count` relance l'énumération sous le nœud. Elle utilise la formule fermée des 4 derniers cubes : voir sa [preuve](formule-quatre-cubes.md).
- **Le rang est uniforme.** Le code de l'application le tire avec `rand::random_range`, supposé uniforme.
- **Le code suit le modèle.** La preuve porte sur le modèle $(P, Q, F)$, pas sur le Rust ligne à ligne.

## Vérification par le code

- Bijection exhaustive pour $n = 1$ à $6$ : tous les rangs donnent des polycubes distincts, de la bonne taille (`crates/polycubes-sampling/tests/ranked_polycubes.rs`).
- Chaque polycube visité une fois pour $n \leq 5$ (`crates/polycubes-core/tests/enumeration_trace.rs`).

## Ce qui n'est pas uniforme

Seul le polycube final l'est. Ses préfixes (les $k$ premiers cubes dans l'ordre d'énumération) ne sont pas des polycubes uniformes de $k$ cubes : un polycube de $k$ cubes est préfixe d'autant de polycubes de $n$ cubes que son sous-arbre en contient, et ce nombre varie.
