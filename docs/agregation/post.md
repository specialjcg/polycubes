# Brouillon de post : un agrégat de 10 000 cubes

Image : [agregation-10000.gif](agregation-10000.gif)

## Version courte

Du givre en cubes.

Prenez une forme de 5 cubes collés, tirée au hasard. Lâchez-en 2 000 copies,
une par une, loin d'elle. Chacune erre au hasard, case par case, et se colle
au premier contact.

Aucune règle ne dit « fais des branches ». Elles apparaissent seules : les
pointes attrapent les arrivants avant les creux, donc elles poussent plus
vite. C'est le mécanisme du givre sur une vitre et des dendrites dans la
roche.

10 000 cubes au total. Les couleurs suivent l'ordre d'arrivée : rouge pour
les premiers, violet pour les derniers.

Écrit en Rust, affiché avec Tauri, Elm et Three.js.

## Version longue (à ajouter si la place le permet)

Ce modèle s'appelle l'agrégation limitée par diffusion. D'ordinaire on y fait
errer des particules isolées. Ici ce sont des polycubes entiers, et ça change
la forme de l'amas.

Je l'ai mesuré sur 20 000 simulations de 600 cubes. La dimension fractale dit
comment un objet remplit l'espace : 3 pour une boule pleine, 2 pour une
feuille, 1 pour un fil.

- Agrégats de cubes isolés : 2,42
- Agrégats de polycubes de 5 cubes : 2,25
- Suie de flamme, mesurée au microscope : environ 1,8
- Chou-fleur : environ 2,8

Des polycubes entiers donnent donc des amas plus aérés que des cubes seuls :
un polycube touche l'amas plus tôt et dépasse de là où il se colle.

Une réserve : une dimension fractale proche ne veut pas dire un mécanisme
commun. Un chou-fleur ne pousse pas par collage de particules errantes.

## À vérifier avant de publier

- Le lien vers le dépôt : il n'a pas de dépôt distant pour l'instant.
- Les chiffres naturels et leurs sources sont dans [resultats.md](resultats.md).
- L'agrégat du GIF a une dimension mesurée de 2,52 : c'est un seul tirage,
  pas une moyenne, à ne pas citer comme résultat.
- Taille du GIF selon la plateforme (certaines limitent à 5 ou 15 Mo).
