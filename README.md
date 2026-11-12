# polycubes

Dénombrement des polycubes de n cubes. Aucune formule connue ; objectif à terme : un terme de plus que l'état de l'art.

## État de l'art (vérifié sur OEIS le 2026-11-12)

- [A000162](https://oeis.org/A000162) (polycubes à rotation près, miroirs distincts) : connu jusqu'à **n = 22**, a(22) = 12 766 882 202 755 783.
  - a(17)–a(20) : Stanley Dodds, décembre 2023.
  - a(21)–a(22) : Phillip Thompson, février 2024, avec l'algorithme de Dodds (port Rust référencé sur la page OEIS).
- [A038119](https://oeis.org/A038119) : polycubes libres (miroirs identifiés), déduits de A000162 et des achiraux.
- [A001931](https://oeis.org/A001931) : polycubes fixes (translation seule). Connu jusqu'à **n = 22**, a(22) = 306 405 169 166 373 418 (Thompson, février 2024, algorithme de Dodds).

## Ce qui existe

Algorithme de Redelmeier généralisé au réseau cubique, compte les polycubes **fixes**. Les 4 derniers cubes sont comptés en formule fermée, sans visiter les polycubes : un balayage de la frontière (cellules non-essayées, leurs voisins libres, chemins de 2 pas, carrés unité) fournit les sommes de `FrontierSums`, dont `extension_counts` déduit le nombre d'ajouts de 1 à 4 cubes (`src/redelmeier.rs`). Formule dérivée ici, validée seulement par les tests. Parallélisme : chaque thread parcourt l'arbre jusqu'à la taille 6 et réclame les sous-arbres (23 502) dans un registre atomique partagé ; aucune dépendance externe.

**Polycubes à rotation près (A000162)** par le lemme de Burnside sur les 24 rotations (`src/symmetry.rs`) : (fixes + Σ polycubes envoyés sur un translaté d'eux-mêmes par chaque rotation) / 24. Une rotation par classe de conjugaison (tailles 3, 6, 6, 8). Pour chaque rotation et chaque décalage possible, un Redelmeier restreint aux polycubes invariants abandonne une branche dès que les images restant à placer ne tiennent plus ou que l'une d'elles a été écartée. Mono-thread, non optimisé.

```bash
cargo run --release -- 13
```

Sortie : taille, fixes (A001931), à rotation près (A000162).

Validé contre A001931 pour n = 1 à 10 par les tests (`cargo test`), pour chaque taille maximale de 1 à 10. Les valeurs n = 11 à 16 sorties par le binaire (…, 3 322 769 321, 24 946 773 111, 188 625 900 446, 1 435 074 454 755) ne sont pas couvertes par un test.

Validé contre A000162 pour n = 1 à 10 par les tests. Le binaire redonne aussi a(11) à a(16) de la page OEIS (…, 7 859 514 470, 59 795 121 480), hors tests.

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

Référence externe (README de Thompson, M1 8 threads) : n = 15 en 10,7 s, n = 16 en 52 s.

Machine : 8 threads matériels (`nproc`). Accélération mesurée ×4,5 sur 8 threads à n = 15.

## Limite

Croissance d'environ ×7,6 par cube. n = 22 a demandé environ 10 jours sur 64 threads à Thompson avec l'algorithme de Dodds (même principe que ce dépôt) ; n = 23 en demanderait environ 7 fois plus. Reproduire Dodds ne suffit donc pas sur 8 cœurs : il faut plus de matériel ou un gain algorithmique supplémentaire (formule fermée sur plus de 3 cubes, GPU).

## Pistes

1. Alléger le balayage de frontière (jusqu'à 36 cellules lues par cellule non-essayée).
2. Paralléliser la partie symétrique (un travail par couple rotation-décalage).
3. Profiler pour de bon : `perf` est bloqué sur cette machine (`perf_event_paranoid = 4`).

## Préceptes

TDD (rouge → vert → refactor), `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` avant tout commit ; Conventional Commits ; mesurer avant d'optimiser.

## Essais rejetés

- Mise à jour des sommes sans branchement (multiplication par un masque d'état au lieu d'un `match`) : n = 15 en 58,9 s contre 35,8 s. Une seule mesure de chaque côté.
