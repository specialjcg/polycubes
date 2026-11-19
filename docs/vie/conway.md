# Planeurs de dominos : un jeu de la vie à la Conway pour polycubes

Mesures du 17 novembre 2026. Données brutes, une ligne par partie :
[conway-balayage.csv](conway-balayage.csv).

## Pourquoi une autre règle

Dans le mode **Vie** de la page Automate, un polycube isolé ne meurt jamais
et chaque vivant bourgeonne une fois sur deux. La population croît donc
jusqu'à remplir le monde. Mesuré dans un grand monde, elle triple toutes les
5 générations, et chaque génération coûte environ 7 ms par polycube. Un monde
de 101 cases de côté demanderait des dizaines d'heures de calcul.

Le jeu de la vie de Conway reste borné parce que la naissance y est rare
(exactement 3 voisins) et que l'isolement tue. La règle ci-dessous transpose
ces deux idées à des polycubes entiers.

## La règle

Tous les polycubes changent en même temps.

- **Mort** : un polycube meurt si le nombre d'autres polycubes qui le
  touchent par une face sort de l'intervalle de survie.
- **Naissance** : personne ne bourgeonne et rien n'est tiré au hasard. Un
  polycube de la même forme, tourné de toutes les façons sans miroir, naît
  sur des cases libres s'il touche un nombre de vivants compris dans
  l'intervalle de naissance et s'il partage avec eux au moins F faces : il
  doit **s'emboîter**, pas seulement toucher.
- **Départage** : une naissance n'a lieu que si aucune autre naissance
  possible ne la chevauche ni ne la touche. Aucune n'est préférée à une
  autre : un départ tourné joue la même partie, tournée (vérifié par test).

## Ce que les essais ont appris

1. **Sans emboîtement, tout explose.** Avec un simple contact, 84 règles sur
   84 explosent sur toutes les soupes : un polycube est étendu, il trouve
   presque partout une place qui touche 2 vivants.
2. **Un premier vaisseau était un artefact.** Avec un départage qui
   choisissait la naissance de plus petites coordonnées, deux bâtonnets de 3
   cubes filaient en ligne droite. Avec le départage symétrique, ils
   disparaissent : leur direction venait de l'ordre de tri.
3. **De vrais planeurs existent.** Avec le départage symétrique, des motifs
   reviennent tels quels, déplacés.

## Le planeur de dominos

Règle : survie à 2 ou 3 voisins, naissance à exactement 2 voisins, au moins
2 faces partagées.

- 4 à 6 dominos selon la phase, tous dans un même plan.
- Il revient tel quel toutes les **10 générations**, **5 cases plus loin en
  diagonale** : une demi-case par génération, comme le planeur de Conway va
  en diagonale.
- Il est apparu tout seul dans 3 soupes sur 20 lors de l'exploration.
- Avec la soupe de la page (320 essais de dominos dans un cube de 16 cases),
  **une soupe sur deux** (32 sur 64) lâche quelque chose qui file à la
  vitesse du planeur : 60 à 68 cases parcourues en 120 générations.

## Balayage

192 règles (3 formes, 4 intervalles de survie, 4 de naissance, F de 2 à 5),
20 soupes chacune (40 essais dans un cube de 8 cases), 300 générations au
plus, arrêt au-delà de 1 500 polycubes.

| Fin de partie | Parties |
|---|---|
| Figée | 2 884 |
| Éteinte | 543 |
| Oscillateur | 177 |
| Explosion | 160 |
| Encore en cours | 70 |
| Planeur | 6 |

- **La population reste bornée** pour 169 règles sur 192 : aucune de leurs
  20 parties n'explose ni ne tourne sans fin.
- Les 6 planeurs sont tous faits de bâtonnets de 3 cubes : périodes 1, 6 et
  14.
- Le planeur de dominos n'apparaît pas dans ce balayage : ses soupes, petites,
  en donnent rarement, et un seul tirage change tout. Il a été trouvé lors de
  l'exploration, avec d'autres soupes, puis vérifié par un test.
- Un planeur qui laisse des débris derrière lui compte comme « encore en
  cours » : le tableau sous-estime les planeurs.

## Planeurs en relief

Le planeur de dominos tient dans un plan. Pour en trouver qui occupent
vraiment l'espace, une seconde recherche essaie 10 formes de 2 à 4 cubes
(domino, I3, L3, carré, T, S, L, vis, vis miroir, trépied), 5 intervalles de
survie, 4 de naissance, F de 2 à 6, soit 1 000 règles. Chaque règle part de
1 000 petits départs (3 à 10 essais dans une boîte de 4 à 6 cases), joués
60 générations au plus. Un motif compte s'il revient tel quel, déplacé, et
s'il s'étale sur au moins deux couches selon chaque axe.

22 planeurs en relief sont sortis, sur 4 formes : carré, vis, vis miroir et
trépied. Les plus épais :

| Forme | Règle | Couches | Période | Déplacement | Polycubes |
|---|---|---|---|---|---|
| carré | S2-4 B3 F4 | 3 × 4 × 5 | 12 | (4, 4, 4) | 6 |
| carré | S2-3 B3 F4 | 3 × 4 × 4 | 8 | (0, 0, 4) | 5 |
| carré | S2-4 B3 F4 | 3 × 4 × 4 | 57 | (19, 19, 19) au signe près | 6 |
| carré | S2-3 B2 F3 | 2 × 3 × 5 | 6 | (3, 0, 3) au signe près | 5 |
| vis | S1-2 B2 F5 | 2 × 2 × 4 | 12 | 12 le long d'un axe | 3 |
| trépied | S1-2 B2 F4 | 2 × 2 × 4 | 6 | 6 le long d'un axe | 3 |

- La page en montre deux : le **planeur 3D** de carrés (période 57, le long
  de la grande diagonale du cube) et le **planeur rapide** de vis (une case
  par génération, la vitesse de la lumière de ce monde).
- La même règle S2-4 B3 F4 porte deux planeurs de carrés différents, de
  périodes 12 et 57, qui vont tous deux en diagonale à un tiers de case par
  génération.
- Vis et trépied, des formes sans plan de symétrie ou presque, donnent les
  planeurs les plus rapides, mais minces : un tube de 2 × 2.
- Domino, I3, L3, T, S et L ne donnent aucun planeur en relief ici.

## Galerie de motifs

Comme la vie de Conway a ses blocs, ses clignotants et ses planeurs, cette
vie a ses motifs fixes, ses oscillateurs et ses planeurs. L'exemple
`pattern_gallery` reprend les formes, les règles et les petits départs de la
recherche de planeurs (50 départs par règle) et classe chaque motif qui
revient tel quel :

- **nature** : fixe (de retour à la génération suivante, en place),
  oscillateur (de retour en place après quelques générations) ou planeur (de
  retour déplacé) ; avec sa **période** et son **déplacement** ;
- **couches** occupées selon chaque axe ;
- **symétrie** : le nombre des 24 rotations du cube qui le laissent tel quel,
  de 1 (aucune) à 24 (un cube plein).

Un motif tourné autrement est le même motif : chacun est ramené à une
**forme canonique**, la plus petite de ses 24 rotations, sans miroir, pour
n'être compté qu'une fois. Seuls restent les motifs d'au moins 3 polycubes,
en relief (2 couches au moins selon chaque axe).

Il en sort 1 634 motifs fixes, 137 oscillateurs et 5 planeurs distincts : les
trois déjà connus, plus un carré de période 6 et un trépied de période 6.
24 sont retenus pour la variété de forme, de nature, de symétrie et de
taille, rangés du plus symétrique au plus éclaté, et écrits dans
[galerie.json](galerie.json) :

- symétries 8, 6, 4, 2 puis 1 : les motifs très symétriques sont tous fixes,
  petits et serrés ; oscillateurs et planeurs n'en ont aucune ;
- oscillateurs de périodes 2, 4 et 6, sur 7 formes ;
- les 5 planeurs, de périodes 6, 12 et 57.

La page **Motifs** de l'application montre les 24 en vignettes fixes,
numérotées et étiquetées ; un clic joue le motif en grand sous sa propre
règle. Le catalogue est embarqué dans le binaire : rien n'est calculé à
l'ouverture.

## Ce qu'on peut en dire, et ce qu'on ne peut pas

- Une règle à la Conway pour polycubes entiers existe, sans hasard, avec une
  population bornée et des planeurs. C'est ce qui manquait au mode Vie.
- Le carré, la vis et le trépied donnent des planeurs en relief ; les formes
  L à 3 et 4 cubes, le T et le S n'en ont donné aucun.
- Rien n'a été cherché au-delà de ces 192 règles, ni de canon à planeurs,
  l'équivalent du canon de Gosper.

## Reproduire

```bash
cargo run --release -p polycubes-automaton --example conway_sweep -- docs/vie/conway-balayage.csv
```

Une demi-heure sur 8 cœurs. Les soupes sont tirées d'une graine
fixe : la même commande redonne les mêmes lignes.

Planeurs en relief, 1 000 départs par règle à partir de la graine 301 :

```bash
cargo run --release -p polycubes-automaton --example glider_search -- 1000 301
```

Galerie de motifs, 50 départs par règle (quelques minutes) :

```bash
cargo run --release -p polycubes-automaton --example pattern_gallery -- 50 docs/vie/galerie.json
```
