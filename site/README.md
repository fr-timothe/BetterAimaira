[← Retour au README du projet](../README.md)

# Site vitrine

Le site public de BetterAimaira : une page d'accueil et une page de téléchargement, en français et en anglais. Servi à la racine de **https://betteraimaira.montfrond.work**.

C'est un projet Astro autonome, avec ses propres dépendances. Il ne partage rien avec l'application Tauri à la racine du dépôt, à l'exception des assets de `assets/` et des valeurs de tokens de `src/app.css`.

---

## Commandes

Toutes se lancent depuis `site/`.

```bash
bun install          # dépendances du site uniquement

bun run dev          # serveur de développement, http://localhost:4321
bun run build        # génère site/dist/
bun run preview      # sert site/dist/
bun run check        # astro check (typage des composants et du contenu)
bun run media        # recopie les assets depuis assets/ vers public/media/
```

`dev`, `build` et `check` lancent `media` avant de démarrer, donc il n'y a jamais besoin de l'appeler à la main.

---

## Déploiement

Le workflow [`.github/workflows/pages.yml`](../.github/workflows/pages.yml) construit le site à chaque push sur `master` qui touche `site/`, `assets/showcase/`, les logos ou les favicons, puis **commite le résultat dans la branche `gh-pages`**.

> [!IMPORTANT]
> Ce site ne se déploie pas avec `actions/deploy-pages`, et ce n'est pas un oubli.
>
> GitHub Pages est configuré sur « Deploy from a branch » (`gh-pages`, racine), et cette même branche porte le flux de mise à jour que chaque application installée interroge (`updates/<canal>/latest.json`, `updates/<canal>/altstore.json`) ainsi que le `CNAME` qui sert le domaine. Basculer la source sur GitHub Actions couperait le flux et empêcherait toute mise à jour côté client.
>
> Le workflow supprime uniquement les fichiers que le site possède et préserve `CNAME`, `.nojekyll` et `updates/`. `release.yml` écrit sur la même branche quand un tag `v*` est publié ; aucun des deux ne force le push, et les deux touchent des chemins disjoints, donc une collision est résolue par le rebase-et-réessai en fin de job.

---

## Structure

```
site/
├── astro.config.mjs        domaine, i18n (fr par défaut, en préfixé), sitemap, Tailwind
├── scripts/copy-media.mjs  copie assets/ → public/media/ au build
├── public/
│   ├── media/              généré, ignoré par git
│   └── robots.txt
└── src/
    ├── styles/global.css   tokens FL-Theme + extension d'échelle d'affichage
    ├── i18n/content.ts     tout le texte des deux langues, sous un type partagé
    ├── lib/
    │   ├── release.ts      résolution de la release GitHub, détection d'OS, formats
    │   └── icons.ts        géométrie Lucide intégrée + marque GitHub
    ├── layouts/Base.astro  head, contrat de direction, en-tête, pied de page
    ├── components/         sections de la page d'accueil et de la page téléchargement
    └── pages/
        ├── index.astro     accueil FR      → /
        ├── download.astro  téléchargement FR → /download
        ├── en/             les deux pages EN → /en/ et /en/download
        └── 404.astro
```

---

## Contenu et traductions

Tout le texte visible vit dans [`src/i18n/content.ts`](src/i18n/content.ts), une fois par langue, sous une même interface `Content`. Une clé ajoutée en français et oubliée en anglais fait échouer `bun run check` au lieu de laisser un trou dans la page. Le site n'utilise pas Paraglide : c'est l'outil de l'application, et il n'a rien à faire dans un projet séparé de quatre pages.

Les mots entre backticks dans le contenu (`` `bun run desktop:build` ``, `` `.exe` ``) sont rendus en `<code>` par [`src/components/Rich.astro`](src/components/Rich.astro). C'est la seule marque en ligne reconnue.

---

## Design

Le site suit le système décrit dans [`DESIGN.md`](../DESIGN.md) et [`docs/DESIGN_SYSTEM.md`](../docs/DESIGN_SYSTEM.md). Deux écarts assumés, tous deux commentés dans `src/styles/global.css` :

- **Les tokens sont recopiés, pas importés.** Un projet Astro séparé ne peut pas importer `src/app.css`, qui porte en plus des préoccupations propres à Tauri (safe areas, `z-index` de la barre de titre, keyframes de l'application). Un token modifié dans l'application doit être modifié ici aussi.
- **Trois pas d'affichage sont ajoutés** au-dessus du `--text-4xl` de l'application : `2.75rem`, `3.5rem`, `4.25rem`. L'échelle de l'application est calibrée pour une interface dense ; un titre de page d'accueil demande plus. L'extension appartient à cette surface seulement.

Le reste est le système tel quel : `--primary` reste un aplat et jamais un trait porteur de sens, `--primary-deep` porte chaque accent lisible, l'élévation est déclarée une fois (bordure **ou** ombre), aucun état n'est signalé par la couleur seule, et la cible minimale reste `--tap-min`.

La stratégie de mouvement tient en une ligne : **un seul moment animé**, l'arrivée orchestrée du premier écran. Rien sous la ligne de flottaison n'a d'entrée au scroll ; une même révélation répétée sur chaque section est un effet, pas un moment. Tout est visible par défaut, et `prefers-reduced-motion` supprime l'animation sans jamais retirer de contenu.

---

## Téléchargements

La page `/download` résout la dernière version côté client, dans [`src/lib/release.ts`](src/lib/release.ts).

- **`/releases/latest` n'est jamais utilisé.** GitHub y résout la dernière release *non* marquée comme préversion, et ce projet ne publie que des betas : ce chemin renverrait un 404. Le point d'entrée liste est lu, et la release non-brouillon la plus récente gagne.
- **Trois plateformes ont un asset publié**, et le site ne prétend rien d'autre : `-x86_64.exe`, `-universal.apk`, `-arm64.ipa`. macOS et Linux affichent les commandes de compilation et un badge « Compilation locale », jamais un bouton de téléchargement pour un binaire qui n'existe pas.
- **Chaque lien fonctionne sans JavaScript.** L'ancre est rendue côté serveur vers la page des releases, et le script la remplace par l'URL directe de l'asset. Une API bloquée dégrade vers une page utilisable.
- **La source AltStore est l'URL permanente du flux** (`updates/beta/altstore.json`), pas l'asset d'un tag, pour qu'elle reste valable d'une version à l'autre.

---

## Assets

[`scripts/copy-media.mjs`](scripts/copy-media.mjs) ne recopie **que ce qu'une page référence réellement**. Le script échoue si un fichier de la liste est absent : une capture manquante casse le build au lieu de laisser un cadre vide en production.

| Fichier servi | Provenance |
|---|---|
| `favicon.svg`, `favicon.png` | `static/`, la marque officielle du projet |
| `screenshot-today.png` | `assets/showcase/screenshot-1.png`, capture de l'application |
| `screenshot-week.png` | `assets/showcase/screenshot-3.png`, capture de l'application |
| `screenshot-grades.png` | `assets/showcase/screenshot-4.png`, capture de l'application |
| `presentation.mp4` | `assets/showcase/`, vidéo de présentation du projet, 24 s |
| `presentation-poster.webp` | image extraite de cette vidéo à 8 s, là où le film montre l'application en usage : `ffmpeg -ss 8 -i assets/showcase/betteraimaira-presentation.mp4 -frames:v 1 -c:v libwebp -quality 82 assets/showcase/presentation-poster.webp` |

Le contenu de portail visible dans les trois captures est un **jeu de démonstration écrit à la main** — aucun relevé réel n'est dans ce dépôt, `PRODUCT.md` l'interdit. Chaque capture est rendue dans un `figure` dont la légende le dit, via la clé `demoNote` de `src/i18n/content.ts`.

`assets/showcase/betteraimaira-demo.webp` n'est délibérément pas dans la liste : 19 Mo, qu'aucune page d'accueil ne peut dépenser.

`assets/logo-lockup.svg` n'y est pas non plus. Ce fichier pose le nom du produit dans un élément `<text>` vivant ; chargé via `<img src>`, un SVG n'atteint pas le `@font-face` de la page, donc le nom s'affichait dans la police système du visiteur, et sa seconde moitié était peinte en `--primary`, soit environ 2,3:1 en texte sur fond clair. Le lockup est donc rendu par [`src/components/Lockup.astro`](src/components/Lockup.astro) : la marque en SVG inline, le nom en vrai texte HTML, en Inter, avec `--primary-deep` pour la moitié accentuée.

### Ajouter une capture d'écran

1. Déposer le fichier dans `assets/showcase/`.
2. L'ajouter à la liste de [`scripts/copy-media.mjs`](scripts/copy-media.mjs) avec son nom de destination.
3. Le référencer depuis `src/i18n/content.ts` (`features[].image`) avec son texte alternatif.

---

## Reste à faire

**Une capture au format téléphone.** Le lecteur sur mobile ne voit aujourd'hui qu'une fenêtre desktop 1440 réduite à ~374 px, où les chiffres de l'application font environ 6 px. Or `PRODUCT.md` décrit la scène principale comme « sur un téléphone entre deux cours » : la preuve devrait être au format de l'appareil qui la lit.

Pour la produire :

```bash
bun run mobile:dev      # ouvre l'application en 412x892, mode mobile forcé
```

Capturer l'écran Aujourd'hui, déposer le fichier en `assets/showcase/screenshot-mobile-today.png`, puis :

1. l'ajouter à la liste de [`scripts/copy-media.mjs`](scripts/copy-media.mjs) ;
2. le servir sous `sm` dans [`src/components/Hero.astro`](src/components/Hero.astro) et dans la première rangée de [`src/components/Features.astro`](src/components/Features.astro), avec un `<picture>` ou une paire d'images à visibilité conditionnelle ;
3. lui écrire son propre texte alternatif dans les deux langues.
