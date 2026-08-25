# Contribuer à BetterAimaira

Merci de l'intérêt. Ce document dit ce qu'une contribution doit respecter pour
être fusionnable, et ce que le projet refusera quelle que soit la qualité du
code.

> 🇬🇧 [English version](CONTRIBUTING.en.md)

## À lire avant d'écrire du code

| Document | Ce qu'il fixe |
| --- | --- |
| [PRODUCT.md](PRODUCT.md) | Utilisateurs, périmètre, contraintes, principes produit |
| [DESIGN.md](DESIGN.md) | Application des tokens à l'élément, primitives partagées, les six états honnêtes |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Découpage Rust, IPC Tauri, état Svelte, cache, frontières de sécurité |
| [docs/BACKEND_API.md](docs/BACKEND_API.md) | Chaque commande Tauri, son contrat de charge utile, les codes d'erreur |
| [NOTICE.md](NOTICE.md) | Ce qui, dans ce dépôt, n'appartient pas au projet |

## Les limites non négociables

Une pull request qui franchit une de ces lignes est refusée, même propre, même
demandée par des utilisateurs. Ce sont les promesses faites dans le README et
sur le site ; elles ne se négocient pas au cas par cas.

- **Lecture seule.** Aucune route d'écriture vers le portail. Pas de profil, pas
  de mot de passe, pas de questionnaire, pas de paiement, pas de démarche.
- **Aucun tiers sur le réseau.** L'application parle au portail configuré par
  l'étudiant et au flux de mise à jour du projet. Rien d'autre : pas de
  télémétrie, pas d'analytique, pas de relais, pas de service push, pas de CDN.
- **HTTPS obligatoire.** Un portail en HTTP est rejeté avant tout envoi
  d'identifiants.
- **Les secrets restent en Rust.** Cookies de session en mémoire côté backend,
  jamais au-delà de la frontière IPC. Mots de passe dans le trousseau de l'OS,
  jamais dans SQLite ni dans les préférences frontend.
- **Le HTML du portail ne franchit pas Rust.** Le contenu distant est converti
  en texte brut avant d'être renvoyé au frontend, et traité comme non fiable.
- **Le pont d'automatisation reste hors des builds distribués.** Il vit derrière
  la feature Cargo `dev-automation` et un `#[cfg(debug_assertions)]`.

## Mise en place

Prérequis et commandes complètes dans le
[README](README.md#développement). Le minimum :

```bash
bun install
bun run desktop:dev
```

## Ce qu'une PR doit vérifier

- **Chaque texte visible passe par Paraglide.** Les chaînes vivent dans
  `messages/fr.json` et `messages/en.json`, les deux, jamais en dur dans un
  composant. Une clé ajoutée dans une langue et pas dans l'autre casse la
  compilation.
- **Aucune couleur ni aucun rayon déclaré localement.** Les tokens viennent de
  `src/app.css` ; voir [DESIGN.md](DESIGN.md).
- **Chaque surface de données distingue ses états** — chargement, vide,
  obsolète, hors ligne, expiré, erreur. Une vue qui devine est un bug.
- **Aucune donnée réelle dans les fixtures.** Pas d'identifiant, pas de cookie,
  pas de nom d'étudiant, pas de contenu de PDF, pas de capture de portail. Les
  tests utilisent des hôtes d'exemple, comme le fait déjà `src-tauri/src/`.
- **Le site suit, si la contribution le touche.** `site/` a son propre
  `bun run check`.

Le bloc de vérification doit passer sans erreur ni avertissement :

```bash
bun run check
bun run build
cd src-tauri
cargo test
cargo clippy --all-targets -- -D warnings
```

## Commits et pull requests

- Messages en anglais, à l'impératif, une ligne de résumé courte.
- Types de commit conventionnels (`feat`, `fix`, `docs`, `ci`, `refactor`…) :
  les notes de release sont construites à partir d'eux, un type mal choisi se
  retrouve dans le mauvais groupe. Relecture locale avec `bun run release:notes`.
- Une PR, un sujet. Un correctif et un refactor dans le même diff se relisent
  mal et se révoquent encore plus mal.
- Capture d'écran pour tout changement visible, dans les deux langues si le
  texte bouge.

## Ajouter une école

Le répertoire d'écoles a ses propres règles, notamment de ne jamais écrire une
adresse de portail devinée : voir
[assets/schools/README.md](assets/schools/README.md). Les noms et logos qu'il
contient appartiennent aux écoles, pas au projet — [NOTICE.md](NOTICE.md).

## Signaler un bug, poser une question, remonter une faille

Voir [SUPPORT.md](SUPPORT.md). Une faille de sécurité ne s'ouvre pas en issue
publique.
