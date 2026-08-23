# Product

<!-- impeccable:product-schema 1 -->

## Platform

adaptive

## Users

La première version cible les étudiants d'un établissement utilisant le portail Aimaira. Elle permet à chaque utilisateur de saisir l'adresse de son portail sur l'écran de connexion sans supposer qu'il colle la page d'accueil exacte.

## Product purpose

BetterAimaira est un client Tauri pour consulter les informations étudiantes Aimaira. Le premier jalon couvre la connexion au portail, le chargement du planning réel, le cache local et un écran Aujourd'hui utilisable.

Le produit répond au besoin lorsque l'étudiant identifie immédiatement son cours actuel ou suivant, sa salle et l'état de synchronisation des données, y compris hors ligne.

## Positioning

BetterAimaira remplace la navigation dans le portail web par une interface étudiante locale, réactive et centrée sur les informations quotidiennes. Le client communique directement avec le portail sélectionné par l'utilisateur, sans relais cloud, et conserve les données utiles hors ligne.

## Operating context

- Utilisation sur mobile entre deux cours et sur ordinateur pour consulter un planning plus dense.
- Le portail validé en référence est une instance Aimaira unique, dite instance de référence.
- Le champ d'URL du portail est vide à la première connexion. L'utilisateur peut coller une URL Aimaira profonde ou avec paramètres. L'application nettoie la saisie pour extraire l'origine HTTPS avant connexion.
- Aimaira repose sur des pages HTML, des formulaires ASP.NET, des cookies de session et un point de terminaison de planning renvoyant du JSON dans une réponse déclarée HTML.

## Capabilities and constraints

- Première version strictement en lecture seule.
- Parcours actuel: configuration du portail, authentification, planning réel et écrans Aujourd'hui/Planning.
- Système de traduction complet dès le premier jalon, avec français et anglais comme langues initiales via Paraglide JS.
- Interface Svelte 5 et coeur Rust Tauri 2 partagés entre les plateformes.
- Application Tauri uniquement. Aucune version web ou PWA n'est prévue.
- Les identifiants, cookies et données étudiantes ne quittent jamais l'appareil, hors requêtes directes avec le portail choisi.
- Les cookies restent dans le backend Rust et ne sont pas exposés au frontend.
- Les identifiants persistants utilisent le trousseau sécurisé de chaque système. Aucun mot de passe en clair dans SQLite ou les préférences frontend.
- HTTPS est obligatoire pour tout portail. Les connexions HTTP sont refusées avant l'envoi des identifiants.
- Les identifiants opaques restent des chaînes.
- Les données en cache affichent leur date de synchronisation et leur état de fraîcheur.
- La facturation, les paiements, les démarches administratives et les écritures distantes sont hors périmètre.
- La compatibilité avec les variations d'autres établissements Aimaira sera évaluée après validation de l'instance de référence.
- Aucune donnée de démonstration ne figure dans les surfaces authentifiées de cette verticale.

## Brand commitments

- Nom: BetterAimaira.
- Inspiration produit: principes Papillon d'accès direct à l'information, d'adaptation aux plateformes et de transparence sur les données estimées ou obsolètes.
- Identité visuelle propre à BetterAimaira. Papillon sert de référence méthodologique, pas de modèle graphique à reproduire.
- Le thème FL-Theme publié sur `https://tweakcn.com/themes/cmq57ht7w000204l2axo6ho9v` est la base visuelle approuvée.
- Le premier jalon livre le thème clair. Le mode sombre est prévu pour une étape ultérieure.
- Les fonctions principales restent accessibles à toutes les tailles de fenêtre.

## Evidence on hand

- Reconnaissance locale anonymisée du portail, conservée hors dépôt.
- Notes d'authentification et de routes, conservées hors dépôt.
- Architecture cible: `docs/ARCHITECTURE.md` et `docs/APP_STRUCTURE_AND_PLATFORMS.md`.
- Système visuel initial: `docs/DESIGN_SYSTEM.md` et `src/app.css`.
- Aucune capture brute du portail, donnée personnelle, valeur de cookie ou fixture HTML réelle ne doit être ajoutée au dépôt.

## Product principles

1. Présenter l'information utile avec un minimum d'actions.
2. Afficher le cache immédiatement, puis actualiser en arrière-plan sans bloquer l'interface.
3. Rendre visibles la fraîcheur, le mode hors ligne et les erreurs du portail.
4. Conserver les secrets et les données étudiantes localement.
5. Adapter la densité et les interactions aux capacités de l'appareil sans retirer de fonction essentielle.

## Accessibility and inclusion

- Navigation complète au clavier et focus visible sur ordinateur.
- Cibles tactiles d'au moins 44px sur mobile.
- Aucun état communiqué uniquement par la couleur.
- Prise en charge de `prefers-reduced-motion`, des zones sûres et du zoom de texte sans troncature.
