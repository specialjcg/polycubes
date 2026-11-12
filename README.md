# polycubes

Dénombrement des polycubes de n cubes. Aucune formule connue ; objectif à terme : un terme de plus que l'état de l'art.

## État de l'art (vérifié sur OEIS le 2026-11-12)

- [A000162](https://oeis.org/A000162) (polycubes à rotation près, miroirs distincts) : connu jusqu'à **n = 22**, a(22) = 12 766 882 202 755 783.
  - a(17)–a(20) : Stanley Dodds, décembre 2023.
  - a(21)–a(22) : Phillip Thompson, février 2024, avec l'algorithme de Dodds (port Rust référencé sur la page OEIS).
- [A038119](https://oeis.org/A038119) : polycubes libres (miroirs identifiés), déduits de A000162 et des achiraux.
- [A001931](https://oeis.org/A001931) : polycubes fixes (translation seule). Dernier terme connu **non vérifié** ici.

## Ce qui existe

Baseline : algorithme de Redelmeier généralisé au réseau cubique, mono-thread, compte les polycubes **fixes**.

```bash
cargo run --release -- 13
```

Validé contre A001931 pour n = 1 à 10 par les tests (`cargo test`). Les valeurs n = 11 à 13 sorties par le binaire (60 494 549, 446 205 905, 3 322 769 321) ne sont pas couvertes par un test.

## Mesure

| n max | polycubes énumérés | temps | débit |
|------:|-------------------:|------:|------:|
| 13 | 3 839 107 918 | 33,8 s | 113,6 M/s |

Machine : 8 cœurs, un seul utilisé.

## Limite de la baseline

Redelmeier énumère chaque polycube un par un. Le taux de croissance est d'environ ×8 par cube ; les polycubes fixes à n = 23 se comptent en 10^18. À 10^9/s (8 cœurs), cela représente des décennies. Paralléliser ou micro-optimiser cette baseline ne suffit pas : atteindre n = 23 exige un algorithme qui ne visite pas chaque polycube (approche de Dodds).

## Pistes

1. Lire l'algorithme de Dodds et le port Rust de Thompson, noter coût mémoire/temps de n = 22.
2. Compter par classes de symétrie pour obtenir A000162 à partir des fixes.
3. Paralléliser (découpe de l'arbre à profondeur fixe) pour servir de référence croisée jusqu'à n ≈ 16.

## Préceptes

TDD (rouge → vert → refactor), `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` avant tout commit ; Conventional Commits ; mesurer avant d'optimiser.
