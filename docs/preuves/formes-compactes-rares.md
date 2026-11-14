# Les formes compactes sont rares

On mesure la compacité d'un polycube de $n$ cubes par le **taux de remplissage** de sa boîte englobante $a \times b \times c$ : $n / (abc)$. Un pavé plein a un taux de 1.

> **Théorème.** Soit $A(n)$ le nombre de polycubes fixes de $n$ cubes et $C_\rho(n)$ le nombre de ceux dont le taux de remplissage est au moins $\rho$. Pour tout $n_0 \geq 1$, en posant $\mu = A(n_0)^{1/n_0}$ :
>
> $$\frac{C_\rho(n)}{A(n)} \;\leq\; \left(\frac{n}{\rho}\right)^{3} \mu^{\,n_0} \left(\frac{g(\rho)}{\mu}\right)^{n}, \qquad g(\rho) = \exp\!\left(\frac{H(\rho)}{\rho}\right), \quad H(\rho) = -\rho \ln \rho - (1 - \rho)\ln(1 - \rho).$$
>
> Dès que $g(\rho) < \mu$, la proportion de polycubes de taux au moins $\rho$ tend vers 0 exponentiellement vite.

## Seuils obtenus

$g$ décroît de $+\infty$ (en $0$) à $1$ (en $1$). Le seuil est le $\rho^*$ tel que $g(\rho^*) = \mu$.

| Source de $\mu$ | $n_0$ | $A(n_0)$ | $\mu$ | Seuil $\rho^*$ |
|---|---|---|---|---|
| Recalculé par ce dépôt | 16 | 1 435 074 454 755 | 5,7518 | 0,3794 |
| OEIS A001931 | 22 | 306 405 169 166 373 418 | 6,2349 | 0,3556 |

Quelques valeurs : $g(0{,}38) = 5{,}7404$, $g(0{,}4) = 5{,}3791$, $g(0{,}5) = 4$, $g(0{,}7) = 2{,}3933$.

**Lecture.** Avec la seule valeur $A(16)$ recalculée ici : les polycubes dont la boîte englobante est remplie à 38 % ou plus forment une proportion qui tend vers 0. Avec $A(22)$ pris sur OEIS, le seuil descend à 36 %.

## Preuve

### 1. Majoration de $C_\rho(n)$

Un polycube fixe a un unique représentant dont la boîte englobante est $[0, a[ \times [0, b[ \times [0, c[$. Il est déterminé par $(a, b, c)$ et par un sous-ensemble de $n$ cases de cette boîte. Le taux est au moins $\rho$ si et seulement si $abc \leq n / \rho$. Donc

$$C_\rho(n) \;\leq \sum_{abc \,\leq\, n/\rho} \binom{abc}{n}.$$

*Nombre de boîtes.* Chaque côté vaut au plus $n / \rho$ : il y a au plus $(n / \rho)^3$ triplets.

*Chaque terme.* Pour $n \leq V$ et $p = n / V$, le développement $1 = (p + (1 - p))^V \geq \binom{V}{n} p^n (1 - p)^{V - n}$ donne

$$\binom{V}{n} \leq \exp\bigl(V \cdot H(n / V)\bigr).$$

La fonction $\varphi(V) = V \cdot H(n / V)$ est croissante sur $[n, +\infty[$ : sa dérivée vaut $-\ln(1 - n / V) > 0$. Pour $V \leq n / \rho$, on a donc $\varphi(V) \leq \varphi(n / \rho) = n \cdot H(\rho) / \rho$, soit

$$\binom{V}{n} \leq g(\rho)^n.$$

D'où $C_\rho(n) \leq (n / \rho)^3 \, g(\rho)^n$.

### 2. Minoration de $A(n)$

**Lemme (concaténation).** $A(n + m) \geq A(n) \, A(m)$.

*Preuve.* Ordonnons les cases par $(z, y, x)$ lexicographique. Soit $P$ un polycube de $n$ cubes, de plus grande case $p$, et $R$ un polycube de $m$ cubes, translaté pour que sa plus petite case soit $p + (1, 0, 0)$, voisine de $p$ et plus grande que $p$. Toute case de $R$ est alors plus grande que toute case de $P$ : les deux sont disjoints, et $P \cup R$ est connexe, donc un polycube de $n + m$ cubes. L'application $(P, R) \mapsto P \cup R$ est injective : $P$ est formé des $n$ plus petites cases de l'union, $R$ des $m$ plus grandes. $\blacksquare$

En particulier $A$ est croissante ($A(n + 1) \geq A(n) A(1) = A(n)$). Écrivons $n = q \, n_0 + r$ avec $0 \leq r < n_0$ :

$$A(n) \;\geq\; A(q \, n_0) \;\geq\; A(n_0)^q \;=\; \mu^{\,q n_0} \;\geq\; \mu^{\,n - n_0}.$$

### 3. Conclusion

Le quotient des deux bornes donne l'inégalité du théorème. Si $g(\rho) < \mu$, le facteur $(g(\rho) / \mu)^n$ l'emporte sur $n^3$. $\blacksquare$

## Portée et limites

- **Résultat asymptotique.** Près du seuil, la borne ne devient inférieure à 1 que pour des $n$ énormes : pour $\rho = 0{,}38$ et $n_0 = 16$, $g / \mu \approx 0{,}998$. Pour $\rho = 0{,}5$, $g / \mu \approx 0{,}70$ et il faut encore $n$ de l'ordre de 150. La borne ne dit rien d'utile aux tailles que l'application affiche.
- **Seuil loin d'être optimal.** Deux pertes : $\mu$ sous-estime la vraie constante de croissance $\lambda = \lim A(n)^{1/n}$ (la limite existe par le lemme de concaténation, et $\mu \leq \lambda$) ; et la borne compte tous les sous-ensembles de la boîte, connexes ou non. Un meilleur minorant de $\lambda$ abaisserait le seuil sans changer la preuve. La littérature donne des estimations de $\lambda$ nettement plus hautes que 6,23 ; non vérifiées ici, donc non utilisées.
- **Non prouvé : le taux typique tend vers 0.** C'est ce que suggère la galerie (aucune boîte remplie à plus de 70 % sur 200 tirages à $n = 8$ et $n = 10$), mais le théorème ne couvre que $\rho$ au-dessus du seuil.
- **Non prouvé : la croissance par voisin au hasard favorise les formes compactes.** Mesuré, pas démontré. La boîte englobante ne le montre pas à $n = 8$ et $n = 10$. Le nombre de faces exposées le montre : à $n = 11$, 61,60 % des polycubes fixes sont sans boucle (valeur exacte, par visite des 60 494 549 polycubes) contre 21,45 % des polycubes grandis par voisin (1 000 000 de tirages, intervalle à 95 % : 21,37 à 21,53). Tableau complet dans le README. Le théorème ci-dessus ne dit rien des faces exposées.
- **Nuance : « sans boucle » n'est pas le cas typique en grande taille.** La part exacte de polycubes sans boucle baisse d'environ 6 % par cube ajouté de $n = 4$ à $n = 11$. Ce que les mesures soutiennent : peu de boucles en moyenne (0,49 contact en trop à $n = 11$), près de trois fois moins que la croissance par voisin (1,41).

## Vérification par le code

Aucune pour le théorème lui-même. Les valeurs de $A(16)$ et des seuils se recalculent : `cargo run --release -p polycubes-cli -- 16` pour $A(16)$ ; $\mu$, $g$ et $\rho^*$ par dichotomie sur $g(\rho) = \mu$.
