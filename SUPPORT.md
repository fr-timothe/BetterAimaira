# Obtenir de l'aide

> 🇬🇧 [English version](SUPPORT.en.md)

## D'abord : est-ce l'application ou le portail ?

C'est la distinction qui règle la plupart des demandes. BetterAimaira lit ton
portail, il ne le pilote pas et n'a aucun accès à ton dossier.

**Le projet ne peut rien pour :** une note fausse ou manquante, une absence mal
justifiée, un emploi du temps que ton école n'a pas encore publié, un mot de
passe oublié, un compte bloqué, un document administratif absent. Tout cela vit
dans le portail. Ça se règle avec le secrétariat ou l'administration de ton
établissement, et par personne d'autre.

**Le projet peut quelque chose pour :** l'application qui refuse de se
connecter alors que le portail fonctionne dans un navigateur, un affichage faux
alors que le portail affiche juste, un plantage, une mise à jour qui n'arrive
pas, une école absente de la liste.

## Avant d'ouvrir une issue

1. La [FAQ du site](https://betteraimaira.montfrond.work/#faq) répond aux
   questions les plus fréquentes.
2. La [page de compatibilité](https://betteraimaira.montfrond.work/ecoles) dit
   si ton école est connue et quelle adresse de portail elle utilise.
3. Les [issues existantes](https://github.com/fr-timothe/BetterAimaira/issues?q=is%3Aissue)
   — ouvertes comme fermées, la tienne y est peut-être déjà.

## Ouvrir une issue

[github.com/fr-timothe/BetterAimaira/issues](https://github.com/fr-timothe/BetterAimaira/issues)

À mettre dedans :

- système d'exploitation et version ;
- version de l'application et canal suivi (`stable` ou `beta`) ;
- le code d'erreur affiché par l'interface, tel quel ;
- ce que tu attendais, ce qui s'est produit ;
- ton école, si ça peut aider — facultatif.

**À ne jamais mettre dedans.** Une issue est publique, lisible par n'importe qui
et indexée par les moteurs de recherche. Donc : pas de mot de passe, pas de
cookie, pas de jeton, aucune capture d'écran laissant voir ton nom, tes notes,
tes absences ou d'autres étudiants, aucun PDF venant du portail. Si une capture
est nécessaire, masque tout ce qui identifie quelqu'un.

## École absente ou adresse de portail fausse

Ouvre une issue. Les règles du répertoire sont dans
[assets/schools/README.md](assets/schools/README.md) : une adresse ne s'y écrit
jamais au jugé, elle se confirme d'abord. Si tu connais l'adresse exacte de ton
portail, dis-la — c'est ce qui manque le plus souvent.

Une école qui préfère ne pas figurer dans la liste peut demander son retrait de
la même façon : voir [NOTICE.md](NOTICE.md).

## Faille de sécurité

**N'ouvre pas d'issue publique.** Passe par l'onglet `Security` du dépôt, bouton
`Report a vulnerability`, qui ouvre un fil privé. Si ce bouton n'est pas
disponible, ouvre une issue disant seulement que tu as un rapport de sécurité et
qu'il te faut un canal privé — aucun détail, aucune preuve de concept.

Le périmètre est celui de BetterAimaira : le code de ce dépôt et ses artefacts
publiés. Les failles des portails Aimaira eux-mêmes ne relèvent pas de ce
projet, et tester un portail sur lequel tu n'as pas de compte n'est pas une
recherche de sécurité.

## Ce à quoi t'attendre

Projet mené par une personne, sur son temps. Les réponses arrivent quand elles
arrivent, il n'y a aucun engagement de délai. Une issue précise, reproductible
et sans donnée personnelle est traitée bien plus vite qu'un signalement vague.

Pour contribuer plutôt que signaler : [CONTRIBUTING.md](CONTRIBUTING.md).
