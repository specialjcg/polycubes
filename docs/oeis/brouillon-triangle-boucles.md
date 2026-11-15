# Brouillon de soumission OEIS : polycubes fixes par nombre de boucles

État : brouillon, rien n'est soumis. Les champs ci-dessous sont en anglais car
l'OEIS n'accepte que l'anglais.

## À vérifier avant de soumettre

- **Antériorité dans la littérature.** Ce triangle est celui des « site
  animals » du réseau cubique simple comptés par indice cyclomatique, étudié en
  physique statistique (compendium de Duarte, travaux de Whittington, Soteros,
  Madras). Les données y figurent probablement ; l'OEIS ne les a pas. Il faut
  retrouver la référence exacte et la citer dans REFERENCES, sinon un éditeur
  la demandera.
- **Absence dans l'OEIS.** Cinq recherches sur les lignes et sur les colonnes
  « 1 boucle » et « 2 boucles » n'ont rien trouvé le 14 novembre 2026. À refaire
  le jour de la soumission.
- **Référence Madras.** Revue, volume et pages écrits de mémoire dans LINKS ;
  à contrôler sur la page de l'éditeur (DOI 10.1007/BF01608793).
- **Lien vers le programme.** Le dépôt est public depuis le 15 novembre 2026 :
  <https://github.com/specialjcg/polycubes>. Son adresse est dans LINKS.
- **Nom de l'auteur.** L'OEIS demande le vrai nom ; il faut un compte sur
  oeis.org.

## Contrôles faits

Sur les 12 lignes : la somme de chaque ligne égale A001931(n), le premier terme
égale A118356(n), la longueur égale A007818(n) - n + 2. Les derniers termes non
nuls se lisent à la main : 3 carrés à n = 4, 1 cube 2×2×2 à n = 8, 3 pavés
2×2×3 à n = 12.

## Champs

**NAME**

Irregular triangle read by rows: T(n,k) is the number of fixed polycubes with n
cells whose cell-adjacency graph has cyclomatic number k, for n >= 1 and
0 <= k <= A007818(n) - n + 1.

**DATA**

1, 3, 15, 83, 3, 486, 48, 2967, 496, 18, 18748, 4368, 378, 8, 121725, 36027,
4854, 306, 0, 1, 807381, 288732, 51030, 5544, 159, 24, 5447203, 2280792,
488976, 72244, 5103, 396, 24, 37264974, 17866896, 4463316, 801396, 89715, 7568,
660, 24, 257896500, 139239286, 39546852, 8179476, 1197481, 132681, 12546, 1080,
0, 3

**OFFSET**

1,2

**COMMENTS**

Two cells are adjacent when they share a face. A polycube with n cells and c
pairs of adjacent cells has cyclomatic number k = c - n + 1 and 6n - 2c =
4n + 2 - 2k exposed unit faces, so row n also gives the number of fixed
polycubes with n cells by surface area, from the largest area 4n + 2 downwards.

Equivalently, T(n,k) is the number of site animals with n sites and n - 1 + k
bonds on the simple cubic lattice, counted up to translation.

Rows may contain zeros: no polycube with 8 cells has cyclomatic number 4, and
none with 12 cells has cyclomatic number 8.

**LINKS**

N. Madras, <a href="https://arxiv.org/abs/math/9902161">A pattern theorem for
lattice clusters</a>, Annals of Combinatorics 3 (1999), 357-384.

Jean-Charles Gouleau, <a href="https://github.com/specialjcg/polycubes">Rust program
counting fixed polycubes by cyclomatic number</a>.

**EXAMPLE**

Triangle begins:

         1;
         3;
        15;
        83,       3;
       486,      48;
      2967,     496,     18;
     18748,    4368,    378,    8;
    121725,   36027,   4854,  306,   0,  1;
    807381,  288732,  51030, 5544, 159, 24;
   ...

T(4,1) = 3: the 2 X 2 X 1 square in its three orientations.

T(8,5) = 1: the 2 X 2 X 2 cube, with 12 pairs of adjacent cells.

**PROG**

(Rust) // see the link to the repository; Redelmeier's algorithm, keeping the
number of adjacent pairs up to date as cells are added.

**CROSSREFS**

Row sums give A001931.

Column k=0 gives A118356.

Row n has A007818(n) - n + 2 terms.

Cf. A000162 (free polycubes), A304196 (free tree-like polycubes).

**KEYWORD**

nonn,tabf,more

**AUTHOR**

(nom de l'auteur), Nov 14 2026

## Reproduire les données

```bash
cargo run --release -p polycubes-sampling --example cycle_triangle -- 12
```

Une dizaine de minutes pour 12 cellules ; chaque cellule de plus multiplie le
temps par 7 à 8.
