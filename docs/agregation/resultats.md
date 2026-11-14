# Agrégation de polycubes : 10 000 simulations comparées à des objets naturels

Campagne du 14 novembre 2026. Données brutes : [mesures.csv](mesures.csv), une
ligne par simulation.

## Ce qui a été simulé

- **Modèle** : agrégation limitée par diffusion, le modèle classique du givre,
  des dendrites minérales et de certains coraux. Des objets arrivent de loin en
  errant au hasard et se collent au premier contact. Les pointes captent les
  arrivants avant les creux, d'où les branches.
- **Particularité** : l'objet qui erre n'est pas un cube isolé mais un
  polycube entier de 5 cubes, tourné au hasard.
- **Graine** : un polycube de 5 cubes tiré uniformément parmi les 534 polycubes
  fixes de cette taille. Chaque arrivant a la forme de la graine.
- **Taille** : 120 polycubes par agrégat, soit 600 cubes.
- **Nombre** : 10 000 agrégats, tous arrivés à 600 cubes, aucun chevauchement.

## Ce qui a été mesuré

- **Dimension fractale** : l'exposant `d` tel que les `m` premiers cubes
  arrivés s'étalent sur un rayon `r` avec `m` proportionnel à `r^d`. Une boule
  pleine vaut 3, une feuille 2, un fil 1. Le rayon est le rayon de giration,
  l'exposant est une pente ajustée entre 40 et 600 cubes. La mesure est
  contrôlée par test sur une boule (3), un plan (2) et une ligne (1).
- **Boucles** : nombre cyclomatique de l'agrégat, la grandeur du triangle
  étudié ailleurs dans ce dépôt.
- **Faces exposées** par cube.

## Résultats

| Mesure | Moyenne | Écart-type | Étendue |
|---|---|---|---|
| Dimension fractale | 2,246 | 0,262 | 5 % sous 1,87, médiane 2,22, 5 % au-dessus de 2,72 |
| Boucles | 58,5 | 38,0 | de 11 à 203, médiane 47 |
| Faces exposées par cube | 3,81 | 0,13 | |
| Plus grand côté de la boîte | 35,0 | 4,4 | |

L'incertitude sur la dimension moyenne est de 0,003 : la moyenne est connue
avec précision, mais un agrégat isolé varie beaucoup (écart-type 0,26, avec
des valeurs aberrantes jusqu'à 3,7, impossibles pour un objet réel et dues au
petit nombre de points de l'ajustement).

**Forme de la graine.** Les 29 formes de 5 cubes (à rotation près) donnent
toutes une dimension moyenne entre 2,22 et 2,32. Le bâton droit est en tête
(2,315 ± 0,039 sur 62 agrégats). L'écart avec la forme la plus basse
(2,219 ± 0,018) vaut un peu plus de deux incertitudes : avec 29 formes
comparées, ce n'est pas concluant. Les graines à une boucle donnent 2,226
± 0,008, les graines sans boucle 2,249 ± 0,003 : un écart faible, à confirmer.

**Boucles et dimension.** Corrélation de −0,03 entre les deux : le nombre de
boucles d'un agrégat ne dit rien de sa dimension fractale.

## Comparaison

| Objet | Dimension fractale | Comment c'est mesuré |
|---|---|---|
| Suie de flamme | 1,82 ± 0,08 | Microscope électronique |
| Arbre bronchique humain | 1,5 à 1,7 | Comptage de boîtes sur scanner |
| **Nos agrégats de polycubes** | **2,25 ± 0,26** | **Masse contre rayon de giration** |
| Agrégation limitée par diffusion, référence | 2,50 ± 0,01 | Simulation hors réseau, très grands agrégats |
| Brocoli | environ 2,7 | Prédit à partir de coupes |
| Chou-fleur | environ 2,8 | Prédit à partir de coupes |
| Boule pleine | 3 | |

Aucune valeur chiffrée n'a été trouvée pour un corail branchu : des études
existent, mais les extraits lus n'en donnent pas.

## Ce qu'on peut en dire, et ce qu'on ne peut pas

- Nos agrégats sont nettement des objets branchus : très loin d'une boule
  pleine (3), plus denses que la suie (1,8), plus aérés qu'un chou-fleur (2,8).
- **Ils sont en dessous de la valeur de référence de leur propre modèle**
  (2,25 contre 2,50). Deux causes probables, non départagées ici : des
  agrégats trop petits (600 cubes, là où la référence en utilise des
  millions), et des arrivants de 5 cubes qui se collent plus tôt qu'un cube
  isolé, ce qui allonge les branches. Une campagne avec des cubes isolés
  permettrait de trancher.
- Les dimensions du tableau ne sont **pas mesurées de la même façon** d'une
  ligne à l'autre. Les spécialistes préviennent que ces méthodes ne sont pas
  directement comparables. Le tableau situe des ordres de grandeur, rien de
  plus.
- **Une dimension fractale proche ne prouve pas un mécanisme commun.** Le
  chou-fleur pousse par division de méristèmes, pas par collage de particules
  errantes. La ressemblance est une ressemblance de forme.

## Reproduire

```bash
cargo run --release -p polycubes-automaton --example aggregation_campaign -- 10000 docs/agregation/mesures.csv
```

Une vingtaine de minutes sur 8 cœurs. Les tirages sont aléatoires : une
nouvelle campagne donne d'autres lignes, et des moyennes proches.

## Sources

- Bowler et Ball, agrégation limitée par diffusion hors réseau en trois
  dimensions, dimension 2,50 ± 0,01 : <https://arxiv.org/abs/cond-mat/0404650>
- Köylü et al., structure fractale des agrégats de suie, dimension
  1,82 ± 0,08 : <https://scholarsmine.mst.edu/mec_aereng_facwork/1066>
- Kim, dimensions fractales d'un brocoli et d'un chou-fleur :
  <https://arxiv.org/abs/cond-mat/0411597>
- Dimension fractale des voies aériennes sur scanner (étude COPDGene) :
  <https://lmi.bwh.harvard.edu/publications/airway-fractal-dimension-predicts-respiratory-morbidity-and-mortality-copd>
- Morphologies des colonies de bactéries, d'Eden à l'agrégation :
  <https://arxiv.org/pdf/1706.05186>
