# polycubes

Dénombrement des polycubes de n cubes. Aucune formule connue ; objectif à terme : un terme de plus que l'état de l'art.

## État de l'art (vérifié sur OEIS le 2026-11-12)

- [A000162](https://oeis.org/A000162) (polycubes à rotation près, miroirs distincts) : connu jusqu'à **n = 22**, a(22) = 12 766 882 202 755 783.
  - a(17)–a(20) : Stanley Dodds, décembre 2023.
  - a(21)–a(22) : Phillip Thompson, février 2024, avec l'algorithme de Dodds (port Rust référencé sur la page OEIS).
- [A038119](https://oeis.org/A038119) : polycubes libres (miroirs identifiés), déduits de A000162 et des achiraux.
- [A001931](https://oeis.org/A001931) : polycubes fixes (translation seule). Connu jusqu'à **n = 22**, a(22) = 306 405 169 166 373 418 (Thompson, février 2024, algorithme de Dodds).

## Ce qui existe

Algorithme de Redelmeier généralisé au réseau cubique, compte les polycubes **fixes**. Les 4 derniers cubes sont comptés en formule fermée, sans visiter les polycubes : un balayage de la frontière (cellules non-essayées, leurs voisins libres, chemins de 2 pas, carrés unité) fournit les sommes de `FrontierSums`, dont `extension_counts` déduit le nombre d'ajouts de 1 à 4 cubes (`crates/polycubes-core/src/redelmeier.rs`). Formule dérivée ici, prouvée dans `docs/preuves/formule-quatre-cubes.md` et validée par les tests : totaux OEIS, comparaison terme à terme avec une visite sur tous les polycubes de 1 à 4 cubes (105) et sur 72 polycubes de 1 à 6 cubes (`tests/frontier_extensions.rs`). Parallélisme : chaque thread parcourt l'arbre jusqu'à la taille 6 et réclame les sous-arbres (23 502) dans un registre atomique partagé ; aucune dépendance externe.

**Polycubes à rotation près (A000162)** par le lemme de Burnside sur les 24 rotations (`crates/polycubes-core/src/symmetry.rs`) : (fixes + Σ polycubes envoyés sur un translaté d'eux-mêmes par chaque rotation) / 24. Une rotation par classe de conjugaison (tailles 3, 6, 6, 8). Pour chaque rotation et chaque décalage possible, un Redelmeier restreint aux polycubes invariants abandonne une branche dès que les images restant à placer ne tiennent plus ou que l'une d'elles a été écartée. Mono-thread, non optimisé.

```bash
cargo run --release -p polycubes-cli -- 13
```

Sortie : taille, fixes (A001931), à rotation près (A000162).

Validé contre A001931 pour n = 1 à 10 par les tests (`cargo test`), pour chaque taille maximale de 1 à 10. Les valeurs n = 11 à 16 sorties par le binaire (…, 3 322 769 321, 24 946 773 111, 188 625 900 446, 1 435 074 454 755) ne sont pas couvertes par un test.

Validé contre A000162 pour n = 1 à 10 par les tests. Le binaire redonne aussi a(11) à a(16) de la page OEIS (…, 7 859 514 470, 59 795 121 480), hors tests.

**Tirage uniforme exact** (`crates/polycubes-sampling`) : `RankedPolycubes` associe à chaque rang de `0..count()` un polycube fixe distinct ; tirer un rang uniforme tire donc un polycube uniforme. La descente choisit à chaque nœud l'enfant dont le sous-arbre contient le rang, les tailles de sous-arbres venant du curseur `EnumerationCursor` du cœur (formule fermée comprise). Tests : pour n = 1 à 6, tous les rangs donnent des polycubes distincts, de la bonne taille, connexes, plus petite cellule à l'origine, et leur nombre est celui d'A001931 — donc une bijection.

```bash
cargo run --release -p polycubes-sampling --example draw -- 12 200000000
```

Coût d'un tirage, 1 thread, sans table de sous-arbres : n = 10 en 4 ms, n = 12 en 0,2 s, n = 13 en 1,3 s (une mesure chacun). Chaque tirage refait l'équivalent d'environ deux énumérations ; une table des sous-arbres peu profonds supprimerait ce coût.

Organisation : workspace Cargo, `crates/polycubes-core` (énumération), `crates/polycubes-cli` (binaire), `crates/polycubes-sampling` (tirage), `crates/polycubes-automaton` (automate cellulaire), `crates/polycubes-app` (coque Tauri), `frontend/` (Elm + Three.js).

**Application de bureau** (`crates/polycubes-app` + `frontend/`) : Tauri 2, interface Elm, rendu Three.js. Sept pages :

- **Croissance biaisée contre tirage uniforme** : deux polycubes apparaissent cube par cube côte à côte. À gauche, chaque cube se colle sur une case libre voisine prise au hasard (croissance d'Eden, `grow_by_random_neighbour`), ce qui favorise les formes compactes. À droite, un polycube tiré uniformément, montré dans son ordre d'énumération : seul le polycube final est uniforme, pas ses étapes. Sous chaque vue, la boîte englobante et son taux de remplissage.
- **Énumération de Redelmeier** (1 à 5 cubes) : l'algorithme polycube par polycube, dans l'ordre où il les visite. Cubes posés, cases en attente (vertes, translucides), cases écartées (cages rouges : déjà essayées plus haut dans l'arbre, interdites en dessous, d'où l'unicité), compteur par taille. Pause, pas à pas, rejouer. La trace vient de `trace_enumeration` (`polycubes-core`), bâtie sur `EnumerationCursor` ; le comptage fermé des 4 derniers cubes n'y figure pas, chaque polycube est visité.
- **Formule fermée** (polycube de 1 à 6 cubes) : un polycube pris au hasard dans l'arbre d'énumération (descente aléatoire, non uniforme), ses cases en attente, et ses extensions par 1 à 4 cubes comptées deux fois : par la formule (`FrontierSums`, lue sur la frontière) et par une visite (`EnumerationCursor`, cube par cube). Les deux comptes sont affichés terme à terme, selon le nombre de cubes ajoutés au-delà des cases en attente ; la scène fait défiler au plus 40 extensions visitées par taille. Les chemins et carrés que la formule compte ne sont pas dessinés un par un.
- **Galerie** (1 à 10 cubes) : 200 tirages uniformes et 200 croissances par voisin au hasard, comparés par le taux de remplissage de leur boîte englobante (histogrammes par dixième), puis 8 tirages uniformes répartis du plus compact au moins compact. Seconde mesure, les **faces exposées** (`exposed_face_count`) : moins il y en a, plus la forme est compacte ; un polycube sans boucle en montre 4n + 2, le maximum. Mesure large (`examples/loopless_share.rs`, une exécution, 94 s) : côté uniforme, valeur **exacte** par visite de tous les polycubes fixes ; côté croissance, 1 000 000 de tirages par taille, intervalle de Wilson à 95 %.

  | n | Polycubes fixes | Sans boucle, uniforme (exact) | Sans boucle, croissance [95 %] | Faces moyennes, uniforme | Faces moyennes, croissance | Maximum 4n + 2 |
  |---|---|---|---|---|---|---|
  | 4 | 86 | 96,51 % | 93,85 % [93,80 ; 93,90] | 17,930 | 17,877 | 18 |
  | 5 | 534 | 91,01 % | 83,08 % [83,00 ; 83,15] | 21,820 | 21,662 | 22 |
  | 6 | 3 481 | 85,23 % | 70,31 % [70,22 ; 70,40] | 25,694 | 25,374 | 26 |
  | 7 | 23 502 | 79,77 % | 57,61 % [57,51 ; 57,71] | 29,562 | 29,026 | 30 |
  | 8 | 162 913 | 74,72 % | 46,00 % [45,90 ; 46,10] | 33,427 | 32,624 | 34 |
  | 9 | 1 152 870 | 70,03 % | 36,19 % [36,09 ; 36,28] | 37,292 | 36,186 | 38 |
  | 10 | 8 294 738 | 65,67 % | 28,03 % [27,94 ; 28,11] | 41,157 | 39,703 | 42 |
  | 11 | 60 494 549 | 61,60 % | 21,45 % [21,37 ; 21,53] | 45,021 | 43,183 | 46 |

  Lecture. Les faces exposées séparent nettement les deux procédés, et l'écart grandit avec n. Chaque boucle retire 2 faces au maximum : à n = 11, un polycube uniforme a en moyenne 0,49 contact de trop pour être un arbre, un polycube grandi par voisin 1,41, soit près de trois fois plus. La part sans boucle baisse aussi côté uniforme (environ 6 % de moins par cube ajouté) : « typiquement sans boucle » n'est donc vrai qu'aux petites tailles ; ce qui tient, c'est « peu de boucles, et bien moins que la croissance ». La boîte englobante, elle, sépare à peine les deux procédés à ces tailles (aucune boîte remplie à plus de 70 % sur 200 tirages, histogrammes voisins).

  ```bash
  cargo run --release -p polycubes-sampling --example loopless_share -- 11 1000000
  ```

- **Remarquables** (1 à 9 cubes) : trois familles d'exceptions, trouvées en visitant tous les polycubes fixes et regroupées à rotation près (au plus 3 formes montrées par famille, les plus symétriques).
  - **Le plus de boucles** (`most_looped_polycubes`) : le carré 2×2 à 4 cubes, le cube 2×2×2 privé d'un coin à 7, le cube 2×2×2 à 8, seul polycube de cette taille à 5 boucles.
  - **Boucles sans carré 2×2** (`most_looped_square_free_polycubes`) : la boucle fait le tour d'un vide. Il faut 6 cubes : c'est le cube 2×2×2 privé de deux coins opposés, 4 orientations. À 8 cubes, 42 formes dont l'anneau plat 3×3. À 9 cubes, une seule forme atteint 2 boucles sans carré.
  - **Arbres les plus ramassés** (`tightest_tree_polycubes`) : sans boucle, dans la boîte au plus petit grand côté puis au plus petit volume. À 5 cubes un seul arbre tient dans 2×2×2 ; à 6 et 7 cubes aucun, les plus ramassés sont plats (1×3×3).
- **Automate** (graines de 1 à 6 cubes) : un polycube qui grossit génération après génération (`crates/polycubes-automaton`). Graine au choix : tirage uniforme, croissance par voisin, ou polycube le plus bouclé. Deux modes, rejoués en boucle dans une vue cadrée une fois pour toutes :
  - **Bourgeonnement** : le polycube entier est la cellule. À chaque génération un polycube sur deux tente de faire naître contre lui une copie de lui-même, tournée au hasard mais jamais en miroir, là où aucune case n'est prise et où elle ne touche aucune autre copie ; 16 générations, 1 000 cellules au plus. Chaque copie a sa couleur.
  - **Réplicateur** : règle de parité de Fredkin, une case vit si un nombre impair de ses six voisines vivent. Quelle que soit la graine, elle réapparaît en six exemplaires intacts à chaque puissance de deux, dès que l'écart dépasse sa largeur (vérifié par test aux générations 4 et 8). La page montre les générations 0 à 8.
  - **Colonie** : chaque cellule qui a une case libre à côté d'elle se divise avec 4 chances sur 10 vers une case libre au hasard ; 24 générations, 1 000 cellules au plus. Les couleurs suivent l'âge : les plus anciennes rouges, les plus jeunes violettes.

  Le crate sait aussi jouer les règles du type « jeu de la vie » (6 ou 26 voisines, `Rule`, `evolve`, `census`) : elles ont été retirées de la page, car partant de quelques cubes elles s'éteignent ou clignotent sans grossir. Mesure faite avant le retrait, règle à 26 voisines « survit à 4–5, naît à 5 » : à 6 cubes, 3 141 graines s'éteignent, 220 se figent, 120 entrent dans un cycle.
- **Tirage uniforme** : un polycube tiré uniformément, à faire tourner en 3D.

Les deux pages de tirage vont de 1 à 12 cubes.

```bash
frontend/build.sh
```

```bash
cargo run --release -p polycubes-app
```

- `crates/polycubes-app/src/drawing.rs` : seule logique de la coque (taille acceptée, mémoire par taille, rang fourni par l'appelant) ; la commande Tauri de `lib.rs` y ajoute le hasard.
- `crates/polycubes-app/src/gallery.rs` : tire les deux échantillons, range par dixième de remplissage, garde 8 tirages uniformes répartis selon le volume de la boîte, compte les polycubes par nombre de faces exposées.
- `crates/polycubes-app/src/growth.rs` : assemble la comparaison des deux croissances.
- `crates/polycubes-app/src/closed_form.rs` : borne la taille à 6, range les extensions visitées par taille et par terme, garde 40 exemples par taille.
- `crates/polycubes-app/src/enumeration.rs` : borne la taille tracée à 5 (639 polycubes) et joint à chaque nœud le compteur par taille.
- `frontend/src` : `Main.elm` (onglets), `Page/Growth.elm`, `Page/Enumeration.elm`, `Page/ClosedForm.elm`, `Page/Gallery.elm` et `Page/Draw.elm` (une page = état, messages et vue ensemble), `Backend.elm` (ports vers Rust), `Scene.elm` (élément `<polycube-scene>`), `CellCountSlider.elm`, `Compactness.elm` (légende de boîte englobante, commune à deux pages), `Polycube.elm`.
- `frontend/public/polycube-scene.js` : élément personnalisé Three.js, piloté par ses attributs `cells`, `shown` (nombre de cubes affichés), `candidates`, `discarded`, `added` (cubes d'une extension) et `extent` (cadrage fixe pour tous les polycubes d'au plus n cubes) ; les cubes déjà dessinés sont conservés d'un état au suivant ; Three.js est copié depuis `node_modules` par `build.sh`, sans CDN.
- `frontend/tools/capture.mjs` : capture d'écran de la page dans Chrome headless en temps réel, pour vérifier une animation sans la fenêtre Tauri.
- `frontend/public/backend.js` : relais des ports Elm vers `invoke`. Il émet `ready` une fois abonné : une demande envoyée depuis `init` serait perdue.

## Mesure

| version | n max | polycubes comptés | temps | débit |
|---|------:|-------------------:|------:|------:|
| énumération complète | 13 | 3 839 107 918 | 33,8 s | 113,6 M/s |
| 3 derniers cubes en formule fermée | 13 | 3 839 107 918 | 6,0 s | 639,4 M/s |
| 3 derniers cubes en formule fermée | 15 | 217 411 781 475 | 350,5 s | 620,3 M/s |
| idem, 8 threads | 15 | 217 411 781 475 | 78,3 s | 2 777,3 M/s |
| idem, 8 threads | 16 | 1 652 486 236 230 | 615,7 s | 2 683,7 M/s |
| sommes de frontière incrémentales, 8 threads | 15 | 217 411 781 475 | 35,8 s | 6 068,1 M/s |
| sommes de frontière incrémentales, 8 threads | 16 | 1 652 486 236 230 | 200,3 s | 8 250,6 M/s |

Comparaison en temps CPU (machine chargée par d'autres calculs, temps mur non fiables), n = 15, 8 threads, 2 passes alternées :

| version | temps CPU |
|---|------:|
| 3 cubes, sommes incrémentales | 129,3 s / 131,1 s |
| 4 cubes, balayage de frontière | 93,4 s / 93,5 s |

Gain ×1,4. n = 16 redonne 1 435 074 454 755.

Partie symétrique (Burnside), 1 thread, machine chargée : n = 14 en 11 s, n = 16 en 82 s, contre 197 s sur 8 threads pour les fixes à n = 16. Croissance d'environ ×3 par cube, donc négligeable devant les fixes quand n grandit.

Machine calme (2026-11-13), 8 threads, fixes seuls : n = 14 en 1,7 s, n = 15 en 12,8 s.

Référence externe (README de Thompson, M1 8 threads) : n = 15 en 10,7 s, n = 16 en 52 s.

Machine : 8 threads matériels (`nproc`). Accélération mesurée ×4,5 sur 8 threads à n = 15.

## Profil

`perf` est bloqué (`perf_event_paranoid = 4`). Deux mesures à la place, à n = 15 :

- Balayage neutralisé (résultats faux, temps seul) : 10,5 s CPU pour le parcours d'arbre contre 76 s de plus pour le balayage → le balayage de frontière représente environ 85 % du temps.
- Échantillonnage avec `gdb` (20 interruptions, 160 relevés de threads actifs) : environ 80 % des relevés dans la boucle interne des chemins de `frontier_sums` (lecture de la cellule en bout de chemin, tests d'état).
- Statistiques de frontière à n = 13 : 7,2 cellules non-essayées par nœud final, 3,7 voisins libres par cellule non-essayée, 1,47 voisine non-essayée par cellule libre de la frontière. Soit environ 200 itérations par nœud final.

## Limite

Croissance d'environ ×7,6 par cube. n = 22 a demandé environ 10 jours sur 64 threads à Thompson avec l'algorithme de Dodds (même principe que ce dépôt) ; n = 23 en demanderait environ 7 fois plus. Reproduire Dodds ne suffit donc pas sur 8 cœurs : il faut plus de matériel ou un gain algorithmique supplémentaire (formule fermée sur plus de 3 cubes, GPU).

## Budget n = 23 (estimation du 2026-11-13)

Unité mesurée : fixes n = 15 en 12,8 s sur 4 cœurs (i7-1185G7, machine calme), soit 51 cœur-secondes.

Le travail est proportionnel au nombre de nœuds finaux, A001931(n − 4). De n = 15 à n = 23 : A001931(19) / A001931(11) ≈ 1,08 × 10⁷ (A001931(19) = 651 459 315 795 897 cité de mémoire, non relu sur OEIS). Le coût par nœud final croît aussi avec n, d'un facteur inconnu pris entre 1 et 1,5.

→ **6 400 à 9 600 cœur-jours** avec ce code, sur des cœurs équivalents à ceux de ce portable. Recoupement : Thompson a mis environ 640 vCPU-jours pour n = 22, soit environ 4 900 vCPU-jours (≈ 2 500 cœurs physiques-jours) extrapolés à n = 23.

| matériel | cœurs | durée (8 000 cœur-jours) | coût |
|---|--:|--:|--:|
| ce portable | 4 | ~5,5 ans | — |
| `gpu-ollama` (mesuré, voir ci-dessous) | 24 | 210 à 315 jours | électricité |
| 1 × Hetzner AX162 (EPYC 9454P) | 48 | ~170 jours | ~1 300 € HT |
| 4 × Hetzner AX162 | 192 | ~42 jours | ~2 000 € HT |
| AWS c7a.48xlarge, spot | 192 | ~42 jours | 3 000 à 3 800 $ |
| AWS c7a.48xlarge, à la demande | 192 | ~42 jours | ~9 900 $ |

Prix relevés par recherche web le 2026-11-13 : Hetzner AX162 199 €/mois + 79 à 100 € de mise en service ; c7a.48xlarge 9,85 $/h à la demande, 3 à 3,8 $/h en spot. Hypothèse non vérifiée : un cœur serveur vaut un cœur de ce portable. À mesurer sur la machine cible avant tout engagement.

Mesure sur `gpu-ollama` (2026-11-13) : 2 × Xeon Gold 6146, 24 cœurs physiques, 48 threads. Fixes n = 13 sur 1 thread : 1,54 s, machine saturée par un autre calcul (charge 46). Même mesure sur ce portable chargé : 2,26 s. Un thread du Xeon vaut donc 1 à 1,5 thread du portable. Extrapolation : 1,54 s × A001931(19) / A001931(9) × (1 à 1,5) ÷ 48 threads = 210 à 315 jours, machine entièrement dédiée. Une seule mesure, courte, sous charge.

Préalables absents du code : points de reprise sur disque, découpage en lots, vérification croisée du résultat (deux exécutions indépendantes ou somme de contrôle par lot).

## Preuves

Rédigées dans `docs/preuves/`, sur un modèle de l'algorithme et non sur le Rust ligne à ligne ; chaque note liste ce qu'elle suppose.

- [Le tirage par rang est uniforme](docs/preuves/tirage-uniforme.md) : lemme de Redelmeier (chaque polycube fixe visité une fois et une seule), puis bijection entre rangs et polycubes.
- [Les formes compactes sont rares](docs/preuves/formes-compactes-rares.md) : la proportion de polycubes dont la boîte englobante est remplie à 38 % ou plus tend vers 0, à partir du seul a(16) recalculé ici (36 % avec a(22) d'OEIS). Résultat asymptotique, sans portée aux tailles affichées. Non prouvé : que le taux typique tend vers 0, et que la croissance par voisin favorise le compact (observé sur échantillons par les faces exposées, voir la galerie).
- [La formule fermée des 4 derniers cubes](docs/preuves/formule-quatre-cubes.md) : cas par cas, selon le nombre de cases libres ajoutées.

Non relu par un tiers.

**Preuve formelle (Lean 4.35.0, bibliothèque de base seule, sans Mathlib)** : `lean/RankedDraw.lean` formalise le théorème du tirage par rang sur un arbre fini abstrait. `Tree.pick` reproduit la descente de `polycube_at` ; `Tree.pick_eq` établit qu'elle rend la feuille de ce rang dans l'ordre de gauche à droite ; suivent `pick_mem`, `pick_none`, `pick_surjective` et `pick_injective` (feuilles distinctes). Aucun `sorry` ; axiomes utilisés : `propext`, `Quot.sound`.

```bash
lean lean/RankedDraw.lean
```

Ce que Lean ne couvre pas : le lien entre cet arbre abstrait et le code Rust, le lemme de Redelmeier (feuilles = polycubes fixes, tous distincts), la formule des 4 cubes, la borne de rareté, et le passage de « bijection » à « probabilité uniforme », laissé en prose.

## Pistes

1. Alléger le balayage de frontière (jusqu'à 36 cellules lues par cellule non-essayée).
2. Paralléliser la partie symétrique (un travail par couple rotation-décalage).
3. Profiler pour de bon : `perf` est bloqué sur cette machine (`perf_event_paranoid = 4`).

## Préceptes

TDD (rouge → vert → refactor), `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` avant tout commit ; Conventional Commits ; mesurer avant d'optimiser.

## Essais rejetés

- Mise à jour des sommes sans branchement (multiplication par un masque d'état au lieu d'un `match`) : n = 15 en 58,9 s contre 35,8 s. Une seule mesure de chaque côté, machine chargée.
- Balayage : lire une seule fois les 6 voisins de la cellule non-essayée, sauter les cellules sans voisin libre et les impasses : n = 15 en 13,6 s et 13,8 s contre 12,8 s. Pas de gain.
- Balayage : boucle interne des chemins sans branchement : n = 15 en 18,5 s contre 12,8 s (seconde mesure à 20,7 s, machine redevenue chargée).
