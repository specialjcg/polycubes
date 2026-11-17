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

## Ce qu'on peut en dire, et ce qu'on ne peut pas

- Une règle à la Conway pour polycubes entiers existe, sans hasard, avec une
  population bornée et des planeurs. C'est ce qui manquait au mode Vie.
- On ne sait pas si d'autres formes que le domino et le bâtonnet donnent des
  planeurs : la forme L n'en a donné aucun ici.
- Rien n'a été cherché au-delà de ces 192 règles, ni de canon à planeurs,
  l'équivalent du canon de Gosper.

## Reproduire

```bash
cargo run --release -p polycubes-automaton --example conway_sweep -- docs/vie/conway-balayage.csv
```

Une demi-heure sur 8 cœurs. Les soupes sont tirées d'une graine
fixe : la même commande redonne les mêmes lignes.
