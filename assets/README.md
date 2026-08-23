# BetterAimaira: Assets et charte graphique

Ce dossier regroupe les fichiers sources vectoriels de l'identité visuelle de **BetterAimaira**, basés sur le concept **"Le Signal de Créneau" (The Slot Matrix)**.

## Fichiers disponibles

| Fichier | Format | Description | Utilisation |
| :--- | :--- | :--- | :--- |
| [`logo.svg`](./logo.svg) | SVG (100x100) | Icône complète sur fond squircle bleu encre (`#13253F`) | Icône d'application, splash screen, avatar |
| [`mark.svg`](./mark.svg) | SVG (100x100) | Marque vectorielle transparente pour fond clair | En-têtes in-app, barres de navigation, badges |
| [`mark-dark.svg`](./mark-dark.svg) | SVG (100x100) | Marque vectorielle transparente pour fond sombre | Mode sombre, pieds de page sombres |
| [`logo-lockup.svg`](./logo-lockup.svg) | SVG (280x80) | Lockup horizontal complet (Symbole + Typographie "BetterAimaira") | En-tête de connexion desktop, documentation |

## Symbolisme et géométrie

- **Colonnes de planning:** Piliers verticaux arrondis évoquant les créneaux d'un emploi du temps étudiant.
- **Colonne active cyan (`#00B9E8`):** Représente le cours immédiat ou prochain ("Today"), la salle et l'instantanéité de l'information.
- **Point focal (Beacon):** Représente la synchronisation en temps réel et la ponctualité.
- **Monogramme:** L'agencement des trois colonnes et du point bas dessine la silhouette d'un A majuscule.

## Palette de couleurs (FL-Theme)

- **Bleu encre:** `#13253F` / `oklch(0.3098 0.0748 248.9089)`
- **Cyan primaire:** `#00B9E8` / `oklch(0.7347 0.1447 228.9136)`
- **Bleu secondaire:** `#223D63` / `#3A5378`
- **Blanc:** `#FFFFFF`

## Composant Svelte 5

Dans le code source, utiliser le composant dédié [`src/lib/assets/Logo.svelte`](../src/lib/assets/Logo.svelte) :

```svelte
<script>
  import Logo from '$lib/assets/Logo.svelte';
</script>

<!-- Marque transparente pour en-tête -->
<Logo size={24} variant="mark" />

<!-- Icône avec fond squircle -->
<Logo size={40} variant="icon" />

<!-- Lockup avec texte -->
<Logo size={32} variant="lockup" />
```
