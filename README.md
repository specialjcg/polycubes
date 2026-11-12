# polycubes

Dénombrement des polycubes de n cubes. Aucune formule connue ; objectif à terme : un terme de plus que l'état de l'art.

## État de l'art (vérifié sur OEIS le 2026-11-12)

- [A000162](https://oeis.org/A000162) (polycubes à rotation près, miroirs distincts) : connu jusqu'à **n = 22**, a(22) = 12 766 882 202 755 783.
  - a(17)–a(20) : Stanley Dodds, décembre 2023.
  - a(21)–a(22) : Phillip Thompson, février 2024, avec l'algorithme de Dodds (port Rust référencé sur la page OEIS).
- [A038119](https://oeis.org/A038119) : polycubes libres (miroirs identifiés), déduits de A000162 et des achiraux.
- [A001931](https://oeis.org/A001931) : polycubes fixes (translation seule). Connu jusqu'à **n = 22**, a(22) = 306 405 169 166 373 418 (Thompson, février 2024, algorithme de Dodds).

## Ce qui existe

Algorithme de Redelmeier généralisé au réseau cubique, mono-thread, compte les polycubes **fixes**. Les 3 derniers cubes sont comptés en formule fermée (dérivation dans la doc de `count_final_extensions`, `src/redelmeier.rs`), sans visiter les polycubes.

```bash
cargo run --release -- 13
```

Validé contre A001931 pour n = 1 à 10 par les tests (`cargo test`), pour chaque taille maximale de 1 à 10. Les valeurs n = 11 à 15 sorties par le binaire (…, 3 322 769 321, 24 946 773 111, 188 625 900 446) ne sont pas couvertes par un test.

## Mesure

| version | n max | polycubes comptés | temps | débit |
|---|------:|-------------------:|------:|------:|
| énumération complète | 13 | 3 839 107 918 | 33,8 s | 113,6 M/s |
| 3 derniers cubes en formule fermée | 13 | 3 839 107 918 | 6,0 s | 639,4 M/s |
| 3 derniers cubes en formule fermée | 15 | 217 411 781 475 | 350,5 s | 620,3 M/s |

Référence externe (README de Thompson, M1 8 threads) : n = 15 en 10,7 s.

Machine : 8 cœurs, un seul utilisé.

## Limite

Croissance d'environ ×7,6 par cube. n = 22 a demandé environ 10 jours sur 64 threads à Thompson avec l'algorithme de Dodds (même principe que ce dépôt) ; n = 23 en demanderait environ 7 fois plus. Reproduire Dodds ne suffit donc pas sur 8 cœurs : il faut plus de matériel ou un gain algorithmique supplémentaire (formule fermée sur plus de 3 cubes, GPU).

## Pistes

1. Paralléliser (découpe de l'arbre à profondeur fixe).
2. Profiler le comptage fermé (36 lectures par cellule non-essayée ; Dodds utilise des compteurs de voisins par octet).
3. Burnside sur les 24 rotations pour obtenir A000162 à partir des fixes.
4. Étendre la formule fermée à 4 cubes.

## Préceptes

TDD (rouge → vert → refactor), `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` avant tout commit ; Conventional Commits ; mesurer avant d'optimiser.
