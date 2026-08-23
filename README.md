<div align="center">
  <img src="assets/logo.svg" height="200">
<h1>BetterAimaira</h1>
<p><strong>Un client local qui remplace le portail étudiant Aimaira par une application adaptative Tauri.</strong>
<br>
<strong>Compatible <code>Windows</code>, <code>macOS</code>, <code>Linux</code>, <code>Android</code> et <code>iOS</code></strong></p>
<br>
<p>
<a href="https://github.com/fr-timothe/BetterAimaira/actions/workflows/release.yml"><img src="https://img.shields.io/github/actions/workflow/status/fr-timothe/BetterAimaira/release.yml?style=for-the-badge&label=Release&color=white&labelColor=black" alt="Workflow de release"></a>
<a href="https://github.com/fr-timothe/BetterAimaira/releases/latest"><img src="https://img.shields.io/github/v/release/fr-timothe/BetterAimaira?style=for-the-badge&label=Derni%C3%A8re%20version&color=white&labelColor=black" alt="Dernière version"></a>
<a href="https://github.com/fr-timothe/BetterAimaira/releases"><img src="https://img.shields.io/github/downloads/fr-timothe/BetterAimaira/total?style=for-the-badge&label=T%C3%A9l%C3%A9chargements&color=white&labelColor=black" alt="Téléchargements"></a>
<a href="LICENSE"><img src="https://img.shields.io/badge/Licence-GPL--3.0-white?style=for-the-badge&labelColor=black" alt="Licence"></a>
</p>
<br>
<p>
  <a href="README.md"><strong>🇫🇷 Français</strong></a> &bull;
  <a href="README.en.md">🇬🇧 English</a>
</p>
<br>
<p><strong>Liens rapides</strong></p>
<p>
<a href="#captures-décran"><img src="https://img.shields.io/badge/Captures_d'%C3%A9cran-000000?style=for-the-badge" alt=""></a>
<a href="#principales-fonctionnalités"><img src="https://img.shields.io/badge/Fonctionnalit%C3%A9s-000000?style=for-the-badge" alt=""></a>
<a href="#téléchargement"><img src="https://img.shields.io/badge/T%C3%A9l%C3%A9chargement-000000?style=for-the-badge" alt=""></a>
<a href="docs/README.md"><img src="https://img.shields.io/badge/Documentation-000000?style=for-the-badge" alt=""></a>
</p>
<p>
<a href="#développement"><img src="https://img.shields.io/badge/D%C3%A9veloppement-000000?style=for-the-badge" alt=""></a>
<a href="#confidentialité"><img src="https://img.shields.io/badge/Confidentialit%C3%A9-000000?style=for-the-badge" alt=""></a>
<a href="#stack-technique"><img src="https://img.shields.io/badge/Stack_Technique-000000?style=for-the-badge" alt=""></a>
<a href="#licence"><img src="https://img.shields.io/badge/Licence-000000?style=for-the-badge" alt=""></a>
</p>

<p align="center">
  <img src="assets/showcase/betteraimaira-demo.webp" alt="Démonstration animée de BetterAimaira" width="100%" style="border-radius: 12px; box-shadow: 0 20px 40px rgba(0,0,0,0.3);" />
</p>
<p align="center">
  <small>🎥 <a href="assets/showcase/betteraimaira-presentation.mp4">Télécharger la vidéo de présentation 1080p (MP4)</a></small>
</p>
</div>

## Introduction

Aimaira est un intranet web utilisé par les établissements d'enseignement pour les emplois du temps, les notes, les absences et les documents administratifs.
BetterAimaira en est un client natif : l'étudiant colle n'importe quelle page de son propre portail, le cœur en Rust la normalise vers son origine HTTPS, s'authentifie, et l'interface affiche l'essentiel en priorité — le cours actuel ou suivant, sa salle et la fraîcheur des données.

L'application communique exclusivement avec le portail configuré et rien d'autre. Aucun relais cloud intermédiaire n'est utilisé, les identifiants résident dans le trousseau sécurisé du système d'exploitation, et chaque vue indique explicitement son état (en chargement, vide, obsolète, hors ligne, expiré ou en erreur) sans jamais deviner. Cette première version est strictement en lecture seule : les actions administratives, de facturation et d'écriture distante sont hors périmètre.

## Captures d'écran

<p align="center">
  <em>(Captures d'écran issues de l'interface de l'application en anglais)</em>
</p>

<div align="center">
 <table>
  <tr>
   <td align="center"><strong>Aujourd'hui — Cours en direct & Métriques</strong></td>
   <td align="center"><strong>Emploi du temps — Vue Jour</strong></td>
  </tr>
  <tr>
   <td><img src="assets/showcase/screenshot-1.png" width="100%"></td>
   <td><img src="assets/showcase/screenshot-2.png" width="100%"></td>
  </tr>
  <tr>
   <td align="center"><strong>Emploi du temps — Grille Semaine</strong></td>
   <td align="center"><strong>Notes & Évaluations</strong></td>
  </tr>
  <tr>
   <td><img src="assets/showcase/screenshot-3.png" width="100%"></td>
   <td><img src="assets/showcase/screenshot-4.png" width="100%"></td>
  </tr>
 </table>
</div>

## Principales fonctionnalités

<details>
  <summary>Session et portail</summary>

- `Normalisation de l'URL du portail` (tout lien profond collé est réduit à son origine HTTPS)
- `HTTPS uniquement` (un portail en HTTP est rejeté avant tout envoi d'identifiants)
- `Authentification par formulaire en Rust` (jeton anti-falsification, gestionnaire privé de cookies)
- `Persistance optionnelle du mot de passe` (Windows Credential Manager, Trousseau macOS, Keystore Android, Secret Service Linux)
- `Restauration automatique de session` au lancement
- `Codes d'erreur stables` (traduits dans l'interface, jamais de diagnostics bruts)
- `Interface en français et en anglais`, complète dès la première version
</details>

<details>
  <summary>Emploi du temps et planning</summary>

- `Semaine ancrée le lundi` (correspondant à la fenêtre de requête de 7 jours du portail)
- `Cours actuel ou suivant` avec salle, enseignant et note du portail
- `Compte à rebours et progression` pour le cours en cours
- `Sélecteur de jour` sur fenêtres compactes, `Grille hebdomadaire` dès que la largeur le permet
- `Paramètres de planning du portail` (`urlTempoSeance`, `tempoLinkVisible`, `sundaysVisible`)
- `Lien de séance Tempo`, uniquement lorsque le portail le signale comme visible
- `Nettoyage du texte du portail` (le HTML du portail est converti en texte brut avant de quitter Rust)
</details>

<details>
  <summary>Notes, présences et documents</summary>

- `Notes` (lecture seule `/Note`, adaptateur sémantique en Rust)
- `Synchronisation des notes au lancement` (empreintes SQLite, premier état silencieux)
- `Alertes de notes dans l'application` (bannière d'accueil et tiroir de notifications, sans service push)
- `Présences et absences` (lecture seule `/Absence`)
- `Profil` (lecture seule `/Profil`)
- `Documents` (lecture seule `/Document`)
- `Questionnaires` (lecture seule, avec détail des réponses)
- `Téléchargement sécurisé de PDF` (liste blanche same-origin, vérification de signature PDF, limite à 25 Mio)
</details>

<details>
  <summary>Interface</summary>

- `Mise en page adaptative` dictée par la largeur de fenêtre, et non par le nom du terminal
- `Cinq destinations` (Aujourd'hui, Planning, Notes, Présences, Plus)
- `Barre inférieure flottante` sur fenêtres compactes, `Tiroir latéral et barre supérieure` sur bureau
- `États explicites et honnêtes` (chargement, vide, erreur, expiré, hors ligne, obsolète — sans faux-semblant)
- `Indicateurs de fraîcheur` sur chaque vue en cache
- `Cible tactile interactive minimale de 44px`, focus visible, navigation complète au clavier
- `Support de Reduced Motion` et des zones de sécurité (`Safe Area`)
- `Barre de titre personnalisée sans bordure` sur bureau
</details>

<details>
  <summary>Mises à jour et distribution</summary>

- `Flux de mise à jour unifié` (une seule release GitHub lue par toutes les plateformes)
- `Installation en place signée` sur bureau (vérification minisign, NSIS passif)
- `Passage de relais à PackageInstaller` sur Android
- `Vérification de source AltStore` sur iOS
- `Version installée visible` dans l'onglet Profil de la vue Plus
</details>

<details>
  <summary>Prévu — pas encore implémenté</summary>

- `Mode sombre` (les tokens existent, la version actuelle est livrée en mode clair)
- `Analyses des notes` (courbes de tendances, distribution de classe, simulateur de moyenne)
- `Analyses des présences` (jauge de quota et indicateurs ECTS)
- `Annuaire du campus` (listes du corps enseignant et des étudiants)
- `Export iCal` (génération `.ics` et serveur local d'abonnement)
- `Alertes Webhook` (Discord, Telegram)
- `Widgets et barre système` (écran d'accueil Android/iOS, zone de notification Windows, barre de menus macOS)
- `Déverrouillage biométrique` (Face ID, Touch ID, Windows Hello)
</details>

Le groupe des fonctionnalités prévues est détaillé dans [docs/INTEGRATIONS.md](docs/INTEGRATIONS.md).

## Téléchargement

| Plateforme | Fichier | Installation |
|---|---|---|
| Windows | `*-setup.exe` | Installeur NSIS, se met à jour en place |
| macOS, Linux | `.dmg`, `.app`, `.AppImage`, `.deb` | Compilé localement pour l'instant, même système de mise à jour |
| Android | `*.apk` | APK release universel, puis invite d'installation du système |
| iOS | `*.ipa` | Source AltStore/SideStore, publié manuellement |

<a href="https://github.com/fr-timothe/BetterAimaira/releases/latest"><img src="https://img.shields.io/badge/T%C3%A9l%C3%A9charger-Derni%C3%A8re_Version-000000?style=for-the-badge" alt="Télécharger la dernière version"></a>

Chaque plateforme lit le même flux de publication, et une version installée vérifie les mises à jour trois secondes après son lancement.

## Documentation

| Document | Sujet |
|---|---|
| [Architecture](docs/ARCHITECTURE.md) | Backend Rust, IPC Tauri, stratégie de cache, frontières de sécurité |
| [Structure de l'application et plateformes](docs/APP_STRUCTURE_AND_PLATFORMS.md) | Règles de mise en page adaptative, spécificités par plateforme, matrice de release |
| [API Backend](docs/BACKEND_API.md) | Commandes Tauri, contrats sérialisés, téléchargement de documents, codes d'erreur |
| [Système de design](docs/DESIGN_SYSTEM.md) | Tokens, points de rupture (breakpoints), mises en page responsives |
| [Directives de design](DESIGN.md) | Application des tokens, primitives partagées, états honnêtes |
| [Intégrations](docs/INTEGRATIONS.md) | Flux iCal, webhooks, widgets, alertes in-app |
| [Performance](docs/PERFORMANCE.md) | Commandes de référence, outils de profilage, budgets de bundle |
| [Produit](PRODUCT.md) | Utilisateurs, périmètre, contraintes, principes produit |
| [Ressources de marque](assets/README.md) | Fichiers de logo, géométrie, palette, composant Svelte |

## Développement

### Prérequis

- [Bun](https://bun.sh/) >= 1.2
- [Rust et Cargo](https://rustup.rs/) >= 1.80
- [Prérequis de plateforme Tauri](https://v2.tauri.app/start/prerequisites/) pour les cibles à compiler

### Commandes

```bash
# Cloner et installer
git clone https://github.com/fr-timothe/BetterAimaira.git
cd BetterAimaira
bun install

# Desktop (1280x800, barre de titre personnalisée sans bordure)
bun run desktop:dev

# Aperçu mobile sur desktop (résolution 412x892, mode mobile forcé)
bun run mobile:dev

# Mobile natif, sur émulateur ou appareil physique
bun run android:dev
bun run ios:dev
```

```bash
# Compilation release
bun run desktop:build

# Compilation release exportée vers dist-desktop/, purge de target/
bun run desktop:build:export

# Nettoyage des artefacts de build (src-tauri/target, .svelte-kit, build)
bun run clean

# Cache intermédiaire du compilateur uniquement, bundles release conservés
bun run clean:cache
```

Pousser un tag `v*` déclenche le workflow [`.github/workflows/release.yml`](.github/workflows/release.yml), qui compile l'installeur Windows ainsi que l'APK Android, génère les manifestes de mise à jour et publie la release interrogée par l'application.

### Vérification

```bash
bun run check
bun run build
cd src-tauri
cargo test
cargo clippy --all-targets -- -D warnings
```

`bun run dev` affiche l'interface dans un navigateur mais sans le backend Rust : l'authentification au portail et le trousseau de clés n'y sont donc pas disponibles. Utilisez `bun run desktop:dev` ou `bun run mobile:dev` pour exécuter le backend complet.

### Pont d'automatisation pour le développement

`bun run desktop:dev` démarre un pont local d'automatisation (`tauri-plugin-mcp-bridge`) sur `127.0.0.1:9223` afin de piloter et vérifier l'application face au portail durant le développement. Il est exclu des builds de release via la fonctionnalité Cargo `dev-automation` et un garde `#[cfg(debug_assertions)]` dans `src-tauri/src/lib.rs`. Ses permissions sont déclarées dans `src-tauri/capabilities/dev/dev-mcp-bridge.json`.

N'activez pas ce pont dans les versions distribuées.

## Confidentialité

- **Aucun relais cloud.** Les requêtes transitent directement de l'appareil vers le portail configuré par l'étudiant, nulle part ailleurs.
- **Ni télémétrie, ni analytique, ni service push.** La vérification des notes ne tourne que lorsque l'application est active.
- **Les cookies restent dans Rust.** Le gestionnaire de cookies de session réside en mémoire et ne franchit jamais la frontière IPC.
- **Mots de passe stockés dans le coffre-fort de l'OS.** Aucun mot de passe en clair dans SQLite ou les préférences frontend. La déconnexion explicite supprime l'entrée enregistrée.
- **HTTPS obligatoire.** Tout portail en HTTP est immédiatement rejeté avant la transmission des identifiants.
- **Cache local uniquement.** SQLite conserve les empreintes de notes, les données d'affichage et l'état des alertes non lues.
- **Strictement en lecture seule.** Le client n'écrit jamais sur le portail distant.
- **Données du portail non fiables.** Le contenu distant est rendu sous forme de texte brut sécurisé.

## Stack technique

| Couche | Technologies |
|---|---|
| **Cœur et backend** | [Rust](https://www.rust-lang.org/), [Tauri 2.0](https://v2.tauri.app/), `reqwest` (cookie jar), `scraper` (parsing HTML), `rusqlite` (cache), `keyring` (coffre-fort OS) |
| **Frontend** | [Svelte 5](https://svelte.dev/) (runes), [TypeScript](https://www.typescriptlang.org/), [Tailwind CSS v4](https://tailwindcss.com/), adaptateur statique [SvelteKit](https://svelte.dev/docs/kit) |
| **Thème et tokens** | [FL-Theme via tweakcn](https://tweakcn.com/r/themes/cmq57ht7w000204l2axo6ho9v), republié en utilitaires Tailwind via le bloc `@theme inline` dans `src/app.css` — voir [DESIGN.md](DESIGN.md) |
| **Typographie** | [Inter](https://rsms.me/inter/) variable, auto-hébergée avec `@fontsource-variable/inter` (un client Tauri ne dépend pas d'un CDN externe) |
| **Icônes et graphiques** | [Lucide Svelte](https://lucide.dev/), SVG Svelte 5 sur mesure |
| **Internationalisation** | [Paraglide JS](https://inlang.com/m/gerre34r/library-inlang-paraglideJs) (Français, Anglais) |
| **Gestionnaire de paquets** | [Bun](https://bun.sh/) |

## Contribution

Les contributions sont les bienvenues. Avant d'ouvrir une pull request :

- Consultez [DESIGN.md](DESIGN.md) : utilitaires au niveau de l'élément, tokens issus de `src/app.css`, aucune déclaration locale de couleur ou de rayon d'arrondi.
- Placez chaque texte visible par l'utilisateur dans le catalogue Paraglide (`messages/`).
- Respectez la contrainte de lecture seule : aucune nouvelle route d'écriture vers le portail.
- Exécutez le bloc de vérification ci-dessus : `bun run check` et `cargo clippy -- -D warnings` doivent s'exécuter sans erreur ni avertissement.
- Messages de commit rédigés en anglais, à l'impératif, avec une ligne de résumé courte.

## Licence

[GPL-3.0](LICENSE). Aimaira est un produit tiers ; ce projet est un client indépendant et n'est aucunement affilié à son éditeur.
