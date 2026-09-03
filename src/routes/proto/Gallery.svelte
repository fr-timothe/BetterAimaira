<script lang="ts">
  /**
   * DEV-ONLY PROTOTYPE GALLERY — not part of the shipped app.
   *
   * Four live phones side by side: the view that ships today, then the three
   * candidate structures for it. Each is the real thing in an iframe at
   * 390x844, running the same authored demo week, so what is compared is the
   * structure and not a drawing of it.
   */
  import { ArrowUpRight } from 'lucide-svelte';

  type Candidate = {
    slug: string;
    label: string;
    role: string;
    thesis: string;
    gains: string[];
    risk: string;
    incumbent?: boolean;
  };

  const candidates: Candidate[] = [
    {
      slug: 'current',
      label: 'Vue actuelle',
      role: 'Ce qui est livré aujourd’hui',
      thesis:
        'Barre de contrôle, résumé du jour et bandeau des jours empilés au-dessus d’une grille horaire qui prend ce qui reste.',
      gains: [
        'Hauteur = durée : un cours long est un bloc haut, un trou est un vide.',
        'Les trois portées existent et la ligne d’heure courante est là.',
      ],
      risk:
        'Mesuré ici : 388 px de chrome avant la grille, qui reçoit 360 px et montre 4,5 h sur une amplitude de 10 h. En semaine, six colonnes de 136 px minimum dans 366 px : le scroll part sur deux axes et le balayage de période doit être désactivé.',
      incumbent: true,
    },
    {
      slug: 'pile',
      label: 'La pile',
      role: 'Structure A',
      thesis:
        'Le téléphone abandonne la grille : le jour est une pile de cartes pleine largeur, et le temps libre est une règle étiquetée au lieu d’un vide payé 4,5 rem par heure.',
      gains: [
        'Mesuré ici : 45 px de chrome en haut au lieu de 388, et le double de hauteur donnée au planning.',
        'Les contrôles sont un étage du dock, pas une seconde barre posée dessus : un seul objet flottant, une bordure, un rayon, une ombre.',
        'Aucun scroll horizontal nulle part : le balayage change de période dans les trois portées, y compris la semaine.',
        'Un cours passe de 36 px illisibles à une carte qui porte heure, durée, intitulé, salle, prof et catégorie.',
        'Le mois est une carte de densité qui déplace le même scroll, pas une quatrième vue.',
        'La semaine revient en pied de page, collée en bas quand la journée est légère : la navigation prend la place vide au lieu d’un bandeau permanent.',
      ],
      risk:
        'En renonçant à hauteur = durée, la semaine perd sa forme d’un coup d’œil : un mardi chargé et un mardi léger se lisent pareil tant qu’on ne les a pas fait défiler.',
    },
    {
      slug: 'continuum',
      label: 'Le continuum',
      role: 'Structure B',
      thesis:
        'La portée n’est plus une vue mais un niveau de zoom de la même grille horaire. À chaque zoom la période est calée sur la hauteur disponible.',
      gains: [
        'Mesuré ici : la semaine entière, 08:00 à 18:00, tient dans l’écran. Zéro scroll, ni vertical ni horizontal — et la grille garde exactement la même hauteur quand on change de période.',
        'Les contrôles sont un étage du dock, pas une seconde barre posée dessus : un seul objet flottant, une bordure, un rayon, une ombre.',
        'Un seul vocabulaire de gestes : balayage = période, touche une colonne = zoom sur ce jour, touche une case de mois = zoom sur cette semaine.',
        'Le mois est le même dessin, cinq fois plus petit, et il reste décodable : chaque semaine porte l’échelle horaire en gouttière (8 h / 13 h / 18 h), un filet médian traverse les cellules, et chaque jour affiche son nombre de cours.',
        'Le mois dit aussi *quoi* : chaque marque porte son code — CM, TD, TP, EX, PR — plus son heure de début quand elle est assez haute, la place étant mesurée et non devinée.',
        'Un examen à venir est la seule marque saturée du mois : c’est ce qu’on cherche en ouvrant un mois, donc c’est le point focal, et le reste garde son champ pâle.',
        'Ce qui est passé lâche sa couleur de catégorie au lieu d’être atténué — donc ce qui reste à venir est ce qui ressort, et aucun libellé ne passe sous le plancher de contraste.',
        'Le mois est un champ découpé par des filets, pas trente cartes bordées — et une marque garde le fond pâle de sa catégorie avec un liseré saturé de 1 px, donc la vue reste dans la palette claire.',
        'Un jour vide garde sa grille : la bande dit encore « dimanche, 08:00 à 18:00, rien dedans », ce qu’une carte à la place de la grille ne dit pas.',
      ],
      risk:
        'Au zoom semaine un bloc fait environ 53 px de large. Deux cours qui se chevauchent tombent à 25 px : ils gardent leur couleur de catégorie et une heure compacte, et disent le reste au toucher. La semaine devient une forme qu’on lit, plus un texte — et au zoom mois on lit une charge et une plage, jamais un horaire précis.',
    },
    {
      slug: 'now',
      label: 'Maintenant d’abord',
      role: 'Structure C',
      thesis:
        'Plus d’en-tête du tout : un panneau encre répond « quel cours, quelle salle, combien de temps », et le reste du jour défile dessous.',
      gains: [
        'Zéro chrome de contrôle : le panneau est l’en-tête, et il porte de l’information au lieu de boutons.',
        'La loupe date s’ouvre en tirant le panneau vers le bas : semaine et mois sont ce geste, pas deux vues.',
        'La réponse — quel cours, quelle salle, combien de temps — est lisible à bout de bras sans rien toucher.',
      ],
      risk:
        'Un jour libre vide le panneau. Construit, le prix s’avère plus faible qu’annoncé : la liste continue sur les trois prochains jours qui ont cours, donc l’écran reste utile — va voir samedi 5. Ce qui reste vrai, c’est qu’il n’y a plus de vue d’ensemble de la semaine : la loupe donne des densités, pas des horaires.',
    },
  ];
</script>

<svelte:head><title>Prototypes — Planning mobile</title></svelte:head>

<div class="h-full overflow-y-auto bg-background">
  <!-- The four phones stay at their real pixel size and the row scrolls if the
       window is narrower than they are. Scaling a prototype down is the one
       thing that would put us back where the wireframes were. -->
  <div class="flex w-full flex-col gap-8 px-6 py-8">
    <header class="flex max-w-[46rem] flex-col gap-3">
      <h1 class="text-3xl leading-tight font-extrabold tracking-[-0.02em] text-foreground">
        Trois structures pour le planning sur téléphone
      </h1>
      <p class="text-base leading-relaxed text-muted-foreground">
        Quatre téléphones réels, à 390 × 844, avec la même semaine de démonstration et le même dock.
        Le premier est la vue livrée aujourd’hui. Les trois suivants sont les candidats. Tout est
        interactif : change de portée, de période, ouvre un cours, tire le panneau.
      </p>
      <p class="text-sm text-muted-foreground">
        Les données sont écrites pour la démonstration — aucun relevé du portail n’est dans ce dépôt.
      </p>
    </header>

    <div class="flex gap-6 overflow-x-auto pb-2">
      {#each candidates as candidate (candidate.slug)}
        <section class="flex w-[24rem] shrink-0 flex-col gap-4">
          <div class="flex items-baseline justify-between gap-3">
            <div class="flex min-w-0 flex-col">
              <span
                class="text-2xs font-bold tracking-[0.06em] uppercase"
                class:text-danger-strong={candidate.incumbent}
                class:text-primary-deep={!candidate.incumbent}>{candidate.role}</span
              >
              <h2 class="text-xl leading-tight font-extrabold text-foreground">
                {candidate.label}
              </h2>
            </div>
            <a
              href={`/proto/${candidate.slug}?mobile`}
              target="_blank"
              rel="noreferrer"
              class="inline-flex min-h-9 shrink-0 items-center gap-1 rounded-sm px-2 text-xs
                     font-bold text-primary-deep transition-control fine-hover:bg-muted"
            >
              Plein écran
              <ArrowUpRight size={14} aria-hidden="true" />
            </a>
          </div>

          <!-- The device frame is a frame, not a picture of a phone: one border,
               one radius, and the iframe at its real pixel size inside. -->
          <div
            class="overflow-hidden rounded-xl border bg-card"
            class:border-danger={candidate.incumbent}
            class:border-border={!candidate.incumbent}
          >
            <iframe
              src={`/proto/${candidate.slug}?mobile`}
              title={`Prototype ${candidate.label}`}
              width="390"
              height="844"
              class="block h-[844px] w-[390px] border-0"
              loading="lazy"
            ></iframe>
          </div>

          <div class="flex flex-col gap-3">
            <p class="text-sm leading-relaxed text-foreground">{candidate.thesis}</p>

            <ul class="flex flex-col gap-1.5">
              {#each candidate.gains as gain (gain)}
                <li class="flex gap-2 text-xs leading-relaxed text-muted-foreground">
                  <span class="mt-[0.45rem] size-1 shrink-0 rounded-full bg-primary-deep"></span>
                  {gain}
                </li>
              {/each}
            </ul>

            <p
              class="rounded-md border border-warning bg-warning-surface px-3 py-2 text-xs
                     leading-relaxed text-warning-strong"
            >
              <strong class="font-extrabold">Le prix.</strong>
              {candidate.risk}
            </p>
          </div>
        </section>
      {/each}
    </div>
  </div>
</div>
