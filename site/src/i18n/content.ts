// Every user-visible string on the site lives here, once per locale. The shared
// `Content` type is the reason: a key added to one language and forgotten in the
// other fails `astro check` instead of shipping a hole in the page.
//
// Nothing in this file may claim a capability the app does not have. The
// download platforms in particular follow what `.github/workflows/release.yml`
// actually publishes: a Windows installer, an Android APK and an unsigned iOS
// IPA. macOS and Linux are built from source, and the page says so.

export const languages = {
	fr: 'Français',
	en: 'English',
} as const;

export type Lang = keyof typeof languages;

export const defaultLang: Lang = 'fr';

export const repo = 'https://github.com/fr-timothe/BetterAimaira';

/** The update feed the installed app already polls, on the branch this site ships from. */
export const altStoreSource = 'https://betteraimaira.montfrond.work/updates/beta/altstore.json';

export type PlatformId = 'windows' | 'android' | 'ios' | 'macos' | 'linux';

/** The pages that exist in both languages, and pair up in the language switch. */
export type PageId = '' | 'download' | 'schools';

/** How a school's portal signs a student in, from `assets/schools/schools.json`. */
export type PortalLogin = 'password' | 'email-first' | 'sso';

/** What the app can do with that portal, which is what the page is asked. */
export type SchoolStatus = 'ready' | 'emailFirst' | 'sso' | 'unknown';

/** `release` ships a downloadable asset; `source` is built locally. */
export type PlatformAvailability = 'release' | 'source';

interface PlatformCard {
	name: string;
	/** One line naming what a visitor gets, shown under the platform name. */
	summary: string;
	requirement: string;
	steps: string[];
	note: string;
}

interface Content {
	/** Shown under every app screenshot: the portal content in them is authored. */
	demoNote: string;
	meta: {
		homeTitle: string;
		homeDescription: string;
		downloadTitle: string;
		downloadDescription: string;
		schoolsTitle: string;
		schoolsDescription: string;
	};
	nav: {
		skip: string;
		features: string;
		privacy: string;
		faq: string;
		docs: string;
		download: string;
		schools: string;
		languageLabel: string;
		menuLabel: string;
		home: string;
	};
	hero: {
		title: string;
		lead: string;
		primary: string;
		secondary: string;
		licence: string;
		screenshotAlt: string;
		unofficial: string;
	};
	strengths: {
		title: string;
		items: { title: string; body: string }[];
	};
	privacy: {
		title: string;
		lead: string;
		items: { title: string; body: string }[];
		quote: string;
		quoteBody: string;
	};
	/** The measurement banner. Refusing is not a dead end: it falls back to
	 *  cookieless measurement, so the copy must not promise "nothing at all". */
	consent: {
		label: string;
		body: string;
		accept: string;
		reject: string;
		more: string;
	};
	features: {
		title: string;
		body: string;
		points: string[];
		image: string;
		alt: string;
	}[];
	video: {
		title: string;
		body: string;
		play: string;
		fallback: string;
		download: string;
	};
	platforms: {
		title: string;
		lead: string;
		released: string;
		fromSource: string;
		items: { id: PlatformId; name: string; detail: string; availability: PlatformAvailability }[];
		cta: string;
	};
	faq: {
		title: string;
		items: { question: string; answer: string }[];
	};
	closing: {
		title: string;
		body: string;
		primary: string;
		secondary: string;
	};
	footer: {
		tagline: string;
		resources: string;
		project: string;
		documentation: string;
		releases: string;
		licence: string;
		source: string;
		issues: string;
		unofficial: string;
	};
	download: {
		title: string;
		lead: string;
		detecting: string;
		resolvingAsset: string;
		detectedFor: string;
		detectedUnknown: string;
		notMyDevice: string;
		versionLabel: string;
		publishedLabel: string;
		sizeLabel: string;
		fetchFailed: string;
		fetchFailedAction: string;
		buildFromSource: string;
		allPlatforms: string;
		stepsLabel: string;
		requirementLabel: string;
		copySource: string;
		copied: string;
		betaTitle: string;
		signatureTitle: string;
		betaBody: string;
		signatureNote: string;
		platforms: Record<PlatformId, PlatformCard>;
	};
	schools: {
		title: string;
		lead: string;
		searchLabel: string;
		searchPlaceholder: string;
		clear: string;
		/** The home page check: its heading, its lead, and what it does not do. */
		checkTitle: string;
		checkLead: string;
		checkHint: string;
		/** `{count}` is the number of schools in the directory. */
		checkSeeAll: string;
		checkSheet: string;
		countOne: string;
		countOther: string;
		filterLabel: string;
		filterAll: string;
		categories: Record<string, string>;
		statusLabel: string;
		/** One badge and one sentence per outcome, so no visitor has to guess. */
		status: Record<SchoolStatus, { badge: string; body: string }>;
		portalLabel: string;
		groupNote: string;
		websiteLabel: string;
		emptyTitle: string;
		emptyBody: string;
		missingTitle: string;
		missingBody: string;
		sourceNote: string;
		cta: string;
	};
}

const buildCommands = `git clone https://github.com/fr-timothe/BetterAimaira.git
cd BetterAimaira
bun install
bun run desktop:build`;

const fr: Content = {
	demoNote: 'Données de démonstration, écrites pour ces captures.',
	meta: {
		homeTitle: 'BetterAimaira — ton portail Aimaira, enfin lisible',
		homeDescription:
			'Client local pour les intranets étudiants Aimaira : cours en direct, emploi du temps, notes et absences. Aucun relais cloud, identifiants dans le trousseau du système, lecture seule.',
		downloadTitle: 'Télécharger BetterAimaira',
		downloadDescription:
			'Installeur Windows, APK Android, IPA iOS via AltStore, et compilation depuis les sources sur macOS et Linux.',
		schoolsTitle: 'Ton école est-elle compatible ? — BetterAimaira',
		schoolsDescription:
			'La liste des établissements qui utilisent Aimaira, avec l’adresse de leur portail et ce que BetterAimaira sait en faire. Si ton école n’y est pas, son portail n’est probablement pas compatible.',
	},
	nav: {
		skip: 'Aller au contenu',
		features: 'Fonctionnalités',
		privacy: 'Confidentialité',
		faq: 'Questions',
		docs: 'Documentation',
		download: 'Télécharger',
		schools: 'Écoles',
		languageLabel: 'Langue',
		menuLabel: 'Menu',
		home: 'Accueil',
	},
	hero: {
		title: 'Ton portail Aimaira, enfin lisible.',
		lead: 'Un client local qui lit ton intranet étudiant : le cours en cours, sa salle, la semaine, les notes et les absences. Il parle au portail que tu saisis et à rien d’autre — il n’existe aucun serveur BetterAimaira entre les deux.',
		primary: 'Télécharger',
		secondary: 'Voir le code',
		licence: 'Gratuit, code ouvert, GPL-3.0',
		screenshotAlt:
			'Écran Aujourd’hui de BetterAimaira : le cours en cours avec sa salle, son décompte et l’heure de dernière synchronisation.',
		unofficial: 'Client non officiel, sans lien avec l’éditeur d’Aimaira ni avec ton établissement.',
	},
	strengths: {
		title: 'Conçu pour la journée de cours',
		items: [
			{
				title: 'Ton cours actuel, dès l’ouverture',
				body: 'L’app s’ouvre directement sur ton cours en cours ou le prochain : sa salle, son enseignant, le temps qu’il reste. Rien à chercher.',
			},
			{
				title: 'Ta semaine complète, sur n’importe quel écran',
				body: 'Le même emploi du temps sur sept jours, en grille sur un grand écran ou en liste sur ton téléphone. Aucune information n’est cachée pour gagner de la place.',
			},
			{
				title: 'Jamais un écran vide, même sans réseau',
				body: 'L’app affiche d’abord ce qu’elle connaît déjà, puis vérifie s’il y a du nouveau derrière. Tu vois toujours à quelle heure ça a été mis à jour pour la dernière fois.',
			},
		],
	},
	privacy: {
		title: 'Tes identifiants ne sortent pas de ton appareil.',
		lead: 'Ce n’est pas une intention, c’est la manière dont l’application est construite. Le cœur en Rust est le seul à parler au portail, et il n’a personne d’autre à appeler.',
		items: [
			{
				title: 'Aucun relais cloud',
				body: 'Les requêtes vont directement au portail dont tu as saisi l’adresse. Il n’y a pas de serveur intermédiaire, donc rien à faire fuiter.',
			},
			{
				title: 'Trousseau du système',
				body: 'Si tu choisis d’enregistrer ton mot de passe, il va dans le Credential Manager de Windows, le Trousseau macOS, le Keystore Android ou le Secret Service Linux. Jamais en clair dans un fichier.',
			},
			{
				title: 'HTTPS obligatoire',
				body: 'Un portail en HTTP est refusé avant qu’un seul identifiant soit envoyé. Le lien que tu colles est réduit à son origine HTTPS, quelle que soit la page.',
			},
			{
				title: 'Lecture seule',
				body: 'Cette version ne peut rien écrire sur le portail. Pas de démarche administrative, pas de facturation, pas d’envoi de réponse.',
			},
			{
				title: 'Mesure d’usage, si tu l’acceptes',
				body: 'L’application demande au premier lancement si elle peut compter ses usages. Sans identifiant : chaque lancement envoie un numéro tiré au hasard et jamais enregistré, donc deux ouvertures ne peuvent pas être reliées à la même personne. Refuser n’envoie rien, pas même le refus. Ce site, lui, te demande son cookie : si tu le refuses, ta visite est comptée sans cookie, et la seule chose gardée sur ton appareil est ton refus.',
			},
		],
		quote: 'On ne connaît pas ton mot de passe.',
		quoteBody:
			'Les cookies de session restent dans le cœur Rust et ne sont jamais exposés à l’interface. Aucune donnée d’élève ne quitte l’appareil, en dehors des requêtes vers le portail que tu as choisi.',
	},
	consent: {
		label: 'Mesure d’audience',
		body: 'Pas de pub, pas de revente, pas de partage : juste savoir combien de personnes passent ici. Le cookie sert à ne pas te recompter demain. Si tu le refuses, ta visite est comptée sans cookie, et la seule chose gardée sur ton appareil est ton refus.',
		accept: 'Accepter',
		reject: 'Refuser le cookie',
		more: 'Ce qu’on mesure',
	},
	features: [
		{
			title: 'Aujourd’hui',
			body: 'La vue qui répond à la seule question qu’on se pose entre deux cours.',
			points: [
				'Cours actuel ou suivant, avec sa salle et son enseignant',
				'Décompte et barre de progression sur le cours en cours',
				'Lien de séance Tempo, uniquement quand le portail le signale comme visible',
			],
			image: 'screenshot-today.png',
			alt: 'Écran Aujourd’hui : cours en direct, métriques de la journée et état de fraîcheur des données.',
		},
		{
			title: 'Emploi du temps',
			body: 'La même semaine que le portail, dans la densité que ta fenêtre permet.',
			points: [
				'Grille hebdomadaire dès que la largeur le permet',
				'Sélecteur de jour et liste détaillée sur écran étroit',
				'Paramètres de planning du portail respectés, dimanches inclus',
			],
			image: 'screenshot-week.png',
			alt: 'Emploi du temps en grille hebdomadaire, du lundi au dimanche, avec les cours par créneau.',
		},
		{
			title: 'Notes et absences',
			body: 'Les valeurs telles que le portail les renvoie, jamais déduites d’une phrase traduite.',
			points: [
				'Notes et évaluations avec leur barème et leur coefficient',
				'Absences et retards, avec leur motif quand il existe',
				'Documents et questionnaires, en lecture seule',
			],
			image: 'screenshot-grades.png',
			alt: 'Écran des notes : évaluations listées avec leur barème et leur moyenne.',
		},
	],
	video: {
		title: 'Regarde-la tourner',
		play: 'Lire la vidéo de présentation',
		body: 'Vingt-quatre secondes de l’application en usage réel : connexion au portail, cours en direct, navigation dans la semaine.',
		fallback: 'Ton navigateur ne peut pas lire cette vidéo.',
		download: 'Télécharger la vidéo (MP4, 1080p)',
	},
	platforms: {
		title: 'Une base de code, cinq plateformes',
		lead: 'Une interface Svelte et un cœur Rust partagés. Les releases publient aujourd’hui trois installateurs ; macOS et Linux se compilent depuis les sources, avec le même système de mise à jour.',
		released: 'Installateur publié',
		fromSource: 'Compilation locale',
		items: [
			{ id: 'windows', name: 'Windows', detail: 'Installeur .exe, mise à jour en place', availability: 'release' },
			{ id: 'android', name: 'Android', detail: 'APK universel, trois architectures', availability: 'release' },
			{ id: 'ios', name: 'iOS', detail: 'IPA via AltStore ou SideStore', availability: 'release' },
			{ id: 'macos', name: 'macOS', detail: 'À compiler depuis les sources', availability: 'source' },
			{ id: 'linux', name: 'Linux', detail: 'À compiler depuis les sources', availability: 'source' },
		],
		cta: 'Voir les instructions d’installation',
	},
	faq: {
		title: 'Questions',
		items: [
			{
				question: 'Est-ce que ça marche avec mon établissement ?',
				answer:
					'Le développement cible pour l’instant une instance Aimaira de référence. Tu colles l’adresse de ton propre portail au premier lancement, et n’importe quelle page du portail suffit : elle est réduite à son origine HTTPS avant la connexion. La compatibilité avec les autres instances sera évaluée une fois l’instance de référence validée.',
			},
			{
				question: 'Où va mon mot de passe ?',
				answer:
					'Nulle part, si tu ne demandes pas à l’enregistrer. Si tu le demandes, il va dans le trousseau sécurisé de ton système d’exploitation, jamais dans un fichier de configuration ni dans la base locale. Les cookies de session restent côté Rust et ne sont pas exposés à l’interface.',
			},
			{
				question: 'Est-ce que je peux modifier quelque chose depuis l’application ?',
				answer:
					'Non. Cette version est strictement en lecture seule. Les démarches administratives, la facturation et toute écriture vers le portail sont hors périmètre.',
			},
			{
				question: 'Pourquoi Windows a un installeur et pas macOS ?',
				answer:
					'Parce que la chaîne de publication compile aujourd’hui l’installeur Windows, l’APK Android et l’IPA iOS. macOS et Linux se compilent en local avec `bun run desktop:build`, et lisent ensuite exactement le même flux de mise à jour.',
			},
			{
				question: 'Est-ce que ça marche hors ligne ?',
				answer:
					'Oui, sur les données déjà synchronisées. Chaque vue affiche son cache immédiatement et indique explicitement son état : en chargement, vide, périmé, hors ligne, session expirée ou en erreur. Une synchronisation qui échoue le dit, elle ne laisse pas passer des données périmées pour des données fraîches.',
			},
			{
				question: 'C’est officiel ?',
				answer:
					'Non. BetterAimaira est un client non officiel, sans aucun lien avec l’éditeur d’Aimaira ni avec ton établissement. Le code est ouvert et vérifiable.',
			},
			{
				question: 'Combien ça coûte ?',
				answer:
					'Rien. Le projet est publié sous licence GPL-3.0, sans compte à créer et sans publicité. Aucun traceur publicitaire non plus : ce site te demande son cookie de mesure et compte ta visite sans cookie si tu le refuses, et l’application ne compte ses usages que si tu l’acceptes au premier lancement — sans identifiant, et jamais tes données scolaires.',
			},
			{
				question: 'J’ai trouvé un bug.',
				answer:
					'Ouvre une issue sur le dépôt avec la plateforme, la version affichée dans l’application et ce que tu faisais. Les codes d’erreur affichés par l’interface sont stables et aident beaucoup.',
			},
		],
	},
	closing: {
		title: 'Ce n’est pas encore une version stable.',
		body: 'BetterAimaira est en préversion. Les vues principales — connexion, emploi du temps, notes, absences — fonctionnent sur l’instance de référence, et le reste avance. Si tu l’installes maintenant, tu verras des rugosités, et une issue vaut mieux qu’un abandon.',
		primary: 'Télécharger la préversion',
		secondary: 'Lire les notes de version',
	},
	footer: {
		tagline: 'Un client local pour les intranets étudiants Aimaira.',
		resources: 'Ressources',
		project: 'Projet',
		documentation: 'Documentation',
		releases: 'Notes de version',
		licence: 'Licence GPL-3.0',
		source: 'Code source',
		issues: 'Signaler un problème',
		unofficial:
			'Client non officiel. Aimaira est une marque de son éditeur ; ce projet n’a aucun lien avec lui ni avec un établissement.',
	},
	schools: {
		title: 'Ton école est-elle compatible ?',
		lead: 'Ces établissements utilisent Aimaira. Trouve le tien pour voir l’adresse de son portail et ce que BetterAimaira sait en faire. La liste est tenue à la main : elle ne t’observe pas et n’interroge aucun portail.',
		searchLabel: 'Chercher ton établissement',
		searchPlaceholder: 'Nom, sigle ou groupe — ESGI, Eduservices, Nantes…',
		clear: 'Effacer',
		checkTitle: 'Vérifie ton école avant de télécharger',
		checkLead:
			'Tape le nom de ton établissement : tu sauras tout de suite si BetterAimaira sait lire son portail. La recherche se fait dans cette page, aucun portail n’est interrogé.',
		checkHint: 'Liste tenue à la main, complétée à chaque école confirmée.',
		checkSeeAll: 'Voir les {count} établissements',
		checkSheet: 'Ouvrir la fiche complète',
		countOne: 'établissement',
		countOther: 'établissements',
		filterLabel: 'Filtrer par domaine',
		filterAll: 'Tous',
		categories: {
			arts: 'Arts du spectacle',
			business: 'Commerce & management',
			communication: 'Communication & marketing',
			creation: 'Création & multimédia',
			droits: 'Droit & sciences politiques',
			gestion: 'Gestion & comptabilité',
			hotel: 'Hôtellerie & gastronomie',
			immobilier: 'Immobilier',
			informatique: 'Informatique & technologie',
			inge: 'Ingénierie & sciences',
			mode: 'Mode & artisanat',
			sante: 'Santé & médico-social',
			sport: 'Sport',
			tourisme: 'Tourisme',
		},
		statusLabel: 'Connexion',
		status: {
			ready: {
				badge: 'Prise en charge',
				body: 'Le portail présente le formulaire identifiant + mot de passe que BetterAimaira sait remplir.',
			},
			emailFirst: {
				badge: 'À vérifier',
				body: 'Le portail demande d’abord ton adresse e-mail, puis décide de la suite. Ce déroulé n’a pas été testé depuis l’application : essaie, et dis-nous ce que ça donne.',
			},
			sso: {
				badge: 'Non prise en charge',
				body: 'La connexion passe par un compte extérieur au portail. BetterAimaira ne sait pas s’y authentifier.',
			},
			unknown: {
				badge: 'Adresse inconnue',
				body: 'L’établissement utilise Aimaira, mais l’adresse de son portail ne nous a pas été confirmée. Saisis-la toi-même dans l’application.',
			},
		},
		portalLabel: 'Portail',
		groupNote: 'Portail du groupe {group}.',
		websiteLabel: 'Site de l’école',
		emptyTitle: 'Aucun établissement ne correspond',
		emptyBody: 'Essaie le sigle plutôt que le nom complet, ou le nom du groupe auquel ton école appartient.',
		missingTitle: 'Ton école n’est pas dans la liste ?',
		missingBody: 'Alors elle n’utilise probablement pas Aimaira, et BetterAimaira ne pourra rien en lire. Si tu sais qu’elle l’utilise, ouvre une issue avec l’adresse de son portail : elle sera ajoutée ici.',
		sourceNote: 'Liste mise à jour à la main. Adresses de portail vérifiées une par une, sans jamais interroger le compte de qui que ce soit.',
		cta: 'Télécharger l’application',
	},
	download: {
		title: 'Télécharger BetterAimaira',
		lead: 'Trois plateformes ont un installateur publié. Sur macOS et Linux, la compilation locale prend une commande.',
		detecting: 'Détection de ton appareil…',
		resolvingAsset: 'Lecture de la dernière version publiée…',
		detectedFor: 'Pour ton appareil :',
		detectedUnknown: 'Choisis ta plateforme',
		notMyDevice: 'Ce n’est pas mon appareil',
		versionLabel: 'Version',
		publishedLabel: 'Publiée le',
		sizeLabel: 'Taille',
		fetchFailed: 'Impossible de lire la dernière version depuis GitHub.',
		fetchFailedAction: 'Ouvrir la page des releases',
		buildFromSource: 'Compiler depuis les sources',
		allPlatforms: 'Toutes les plateformes',
		stepsLabel: 'Installation',
		requirementLabel: 'Prérequis',
		copySource: 'Copier l’adresse de la source',
		copied: 'Adresse copiée',
		betaTitle: 'Préversion',
		signatureTitle: 'Signature vérifiée',
		betaBody:
			'Les versions publiées sont des préversions beta. L’application vérifie le flux de mise à jour trois secondes après son lancement et te propose la suivante quand elle sort.',
		signatureNote:
			'L’installeur Windows est publié avec sa signature minisign `.sig`, celle que le système de mise à jour intégré vérifie avant de remplacer une version installée.',
		platforms: {
			windows: {
				name: 'Windows',
				summary: 'Installeur NSIS 64 bits, se met à jour en place.',
				requirement: 'Windows 10 ou 11, 64 bits.',
				steps: [
					'Télécharge l’installeur `.exe`.',
					'Windows SmartScreen affiche un avertissement : l’installeur n’est pas signé par un certificat commercial. Clique sur « Informations complémentaires », puis sur « Exécuter quand même ».',
					'Suis l’installeur jusqu’au bout.',
					'Au premier lancement, colle l’adresse de ton portail Aimaira. N’importe quelle page du portail convient.',
				],
				note: 'Les mises à jour suivantes s’installent depuis l’application : elle vérifie le flux trois secondes après son lancement.',
			},
			android: {
				name: 'Android',
				summary: 'APK universel : `arm64-v8a`, `armeabi-v7a` et `x86_64` dans le même fichier.',
				requirement: 'Android 7.0 ou plus récent.',
				steps: [
					'Télécharge l’APK.',
					'Ouvre le fichier téléchargé. Android demande d’autoriser l’installation depuis cette source : accorde-la à ton navigateur ou à ton gestionnaire de fichiers.',
					'Valide l’invite d’installation du système.',
					'Au premier lancement, colle l’adresse de ton portail.',
				],
				note: 'Android compare les `versionCode`, pas les noms de version : un APK dont le code n’est pas supérieur à celui installé n’est pas vu comme une mise à jour par le système.',
			},
			ios: {
				name: 'iOS',
				summary: 'IPA non signé, installé par AltStore ou SideStore avec ton propre identifiant Apple.',
				requirement: 'iOS 15 ou plus récent, et AltStore ou SideStore déjà installé.',
				steps: [
					'Installe AltStore ou SideStore sur ton iPhone.',
					'Dans l’application, ajoute la source ci-dessous.',
					'Installe BetterAimaira depuis cette source.',
					'Au premier lancement, colle l’adresse de ton portail.',
				],
				note: 'L’IPA est compilé sans signature de code : AltStore et SideStore le resignent avec ton compte. Avec un compte Apple gratuit, le certificat est à renouveler tous les sept jours.',
			},
			macos: {
				name: 'macOS',
				summary: 'Aucun bundle publié pour l’instant : la compilation locale produit le `.app` et le `.dmg`.',
				requirement: 'Bun 1.2 ou plus, Rust 1.80 ou plus, et les outils en ligne de commande Xcode.',
				steps: [buildCommands],
				note: 'Le bundle est écrit dans `src-tauri/target/release/bundle/`. Une fois installé, il lit le même flux de mise à jour que les autres plateformes.',
			},
			linux: {
				name: 'Linux',
				summary: 'Aucun paquet publié pour l’instant : la compilation locale produit l’AppImage et le `.deb`.',
				requirement:
					'Bun 1.2 ou plus, Rust 1.80 ou plus, et les prérequis de plateforme Tauri (`libwebkit2gtk-4.1-dev` et ses compagnons).',
				steps: [buildCommands],
				note: 'Le mot de passe enregistré passe par le Secret Service de la session, donc par GNOME Keyring ou KWallet selon ton bureau.',
			},
		},
	},
};

const en: Content = {
	demoNote: 'Demo data, written for these screenshots.',
	meta: {
		homeTitle: 'BetterAimaira — your Aimaira portal, finally readable',
		homeDescription:
			'A local client for Aimaira student intranets: live class, schedule, grades and absences. No cloud relay, credentials in the system keychain, read-only.',
		downloadTitle: 'Download BetterAimaira',
		downloadDescription:
			'Windows installer, Android APK, iOS IPA through AltStore, and a source build on macOS and Linux.',
		schoolsTitle: 'Is your school supported? — BetterAimaira',
		schoolsDescription:
			'The list of schools running Aimaira, with their portal address and what BetterAimaira can do with it. If your school is not there, its portal is probably not supported.',
	},
	nav: {
		skip: 'Skip to content',
		features: 'Features',
		privacy: 'Privacy',
		faq: 'Questions',
		docs: 'Documentation',
		download: 'Download',
		schools: 'Schools',
		languageLabel: 'Language',
		menuLabel: 'Menu',
		home: 'Home',
	},
	hero: {
		title: 'Your Aimaira portal, finally readable.',
		lead: 'A local client that reads your student intranet: the class you are in, its room, the week, grades and absences. It talks to the portal you type in and to nothing else — there is no BetterAimaira server in between.',
		primary: 'Download',
		secondary: 'View the code',
		licence: 'Free, open source, GPL-3.0',
		screenshotAlt:
			'BetterAimaira Today screen: the current class with its room, its countdown, and the last sync time.',
		unofficial: 'Unofficial client, unaffiliated with the Aimaira vendor or with your school.',
	},
	strengths: {
		title: 'Built for a day of classes',
		items: [
			{
				title: 'Your current class, the moment you open it',
				body: 'The app opens directly on your current or next class: its room, its teacher, how long is left. Nothing to look for.',
			},
			{
				title: 'Your whole week, on any screen',
				body: 'The same seven-day schedule, as a grid on a wide screen or a list on your phone. No information is hidden to save space.',
			},
			{
				title: 'Never a blank screen, even offline',
				body: 'The app shows what it already knows first, then checks for anything new behind it. You always see when it was last updated.',
			},
		],
	},
	privacy: {
		title: 'Your credentials never leave your device.',
		lead: 'Not an intention — the way the app is built. The Rust core is the only part that talks to the portal, and it has nobody else to call.',
		items: [
			{
				title: 'No cloud relay',
				body: 'Requests go straight to the portal whose address you entered. There is no server in the middle, so there is nothing in the middle to leak.',
			},
			{
				title: 'System keychain',
				body: 'If you choose to save your password it goes to Windows Credential Manager, the macOS Keychain, the Android Keystore or the Linux Secret Service. Never in plaintext in a file.',
			},
			{
				title: 'HTTPS only',
				body: 'An HTTP portal is refused before a single credential is sent. Whatever page you paste is reduced to its HTTPS origin first.',
			},
			{
				title: 'Read-only',
				body: 'This version cannot write anything to the portal. No administrative procedures, no billing, no submitted answers.',
			},
			{
				title: 'Usage counting, if you agree to it',
				body: 'On first launch the app asks whether it may count its own usage. With no identifier: each launch sends a number drawn at random and never saved, so two openings cannot be tied to the same person. Declining sends nothing, not even the refusal. This site asks for its cookie instead: decline it and your visit is still counted without one, and the only thing kept on your device is your refusal.',
			},
		],
		quote: 'We do not know your password.',
		quoteBody:
			'Session cookies stay inside the Rust core and are never exposed to the interface. No student data leaves the device, apart from the requests to the portal you chose.',
	},
	consent: {
		label: 'Audience measurement',
		body: 'No ads, no resale, no sharing: just knowing how many people come through. The cookie is what keeps tomorrow from counting you twice. Decline it and your visit is counted without one, and the only thing kept on your device is your refusal.',
		accept: 'Accept',
		reject: 'Decline the cookie',
		more: 'What we measure',
	},
	features: [
		{
			title: 'Today',
			body: 'The view that answers the only question worth asking between two classes.',
			points: [
				'Current or next class, with its room and its teacher',
				'Countdown and progress bar on the class in progress',
				'Tempo session link, only when the portal marks it visible',
			],
			image: 'screenshot-today.png',
			alt: 'Today screen: live class, day metrics, and the freshness state of the data.',
		},
		{
			title: 'Schedule',
			body: 'The same week as the portal, at the density your window allows.',
			points: [
				'Week grid as soon as the width permits',
				'Day picker and detailed list on a narrow screen',
				'The portal’s own schedule settings honoured, Sundays included',
			],
			image: 'screenshot-week.png',
			alt: 'Schedule as a weekly grid, Monday to Sunday, with each class in its slot.',
		},
		{
			title: 'Grades and absences',
			body: 'The values exactly as the portal returns them, never inferred from a translated sentence.',
			points: [
				'Grades and assessments with their scale and weight',
				'Absences and lateness, with the reason when there is one',
				'Documents and questionnaires, read-only',
			],
			image: 'screenshot-grades.png',
			alt: 'Grades screen: assessments listed with their scale and average.',
		},
	],
	video: {
		title: 'Watch it run',
		play: 'Play the presentation video',
		body: 'Twenty-four seconds of the app in real use: signing in to the portal, the live class, moving through the week.',
		fallback: 'Your browser cannot play this video.',
		download: 'Download the video (MP4, 1080p)',
	},
	platforms: {
		title: 'One codebase, five platforms',
		lead: 'One Svelte interface and one Rust core, shared. Releases currently publish three installers; macOS and Linux are built from source and read the same update feed.',
		released: 'Installer published',
		fromSource: 'Local build',
		items: [
			{ id: 'windows', name: 'Windows', detail: '.exe installer, updates in place', availability: 'release' },
			{ id: 'android', name: 'Android', detail: 'Universal APK, three architectures', availability: 'release' },
			{ id: 'ios', name: 'iOS', detail: 'IPA through AltStore or SideStore', availability: 'release' },
			{ id: 'macos', name: 'macOS', detail: 'Built from source', availability: 'source' },
			{ id: 'linux', name: 'Linux', detail: 'Built from source', availability: 'source' },
		],
		cta: 'See the install instructions',
	},
	faq: {
		title: 'Questions',
		items: [
			{
				question: 'Will it work with my school?',
				answer:
					'Development currently targets one reference Aimaira instance. You paste your own portal address on first launch, and any page of that portal will do: it is reduced to its HTTPS origin before connecting. Compatibility with other instances will be assessed once the reference instance is validated.',
			},
			{
				question: 'Where does my password go?',
				answer:
					'Nowhere, unless you ask to save it. If you do, it goes to your operating system’s secure store, never to a config file or the local database. Session cookies stay on the Rust side and are not exposed to the interface.',
			},
			{
				question: 'Can I change anything from the app?',
				answer:
					'No. This version is strictly read-only. Administrative procedures, billing and any remote write are out of scope.',
			},
			{
				question: 'Why does Windows get an installer and macOS does not?',
				answer:
					'Because the release pipeline currently builds the Windows installer, the Android APK and the iOS IPA. macOS and Linux build locally with `bun run desktop:build`, and then read exactly the same update feed.',
			},
			{
				question: 'Does it work offline?',
				answer:
					'Yes, on data already synced. Every view paints its cache immediately and states its condition explicitly: loading, empty, stale, offline, session expired or failed. A sync that fails says so rather than passing stale data off as fresh.',
			},
			{
				question: 'Is this official?',
				answer:
					'No. BetterAimaira is an unofficial client with no connection to the Aimaira vendor or to your school. The code is open and auditable.',
			},
			{
				question: 'What does it cost?',
				answer:
					'Nothing. The project is released under GPL-3.0, with no account to create and no ads. No advertising trackers either: this site asks for its measurement cookie and counts your visit without one if you decline it, and the app only counts its own usage if you agree to it on first launch — with no identifier, and never your school data.',
			},
			{
				question: 'I found a bug.',
				answer:
					'Open an issue on the repository with your platform, the version shown in the app, and what you were doing. The error codes the interface displays are stable and help a lot.',
			},
		],
	},
	closing: {
		title: 'This is not a stable release yet.',
		body: 'BetterAimaira is in prerelease. The main views — sign-in, schedule, grades, absences — work against the reference instance, and the rest is moving. Install it now and you will hit rough edges; an issue beats giving up.',
		primary: 'Download the prerelease',
		secondary: 'Read the release notes',
	},
	footer: {
		tagline: 'A local client for Aimaira student intranets.',
		resources: 'Resources',
		project: 'Project',
		documentation: 'Documentation',
		releases: 'Release notes',
		licence: 'GPL-3.0 licence',
		source: 'Source code',
		issues: 'Report an issue',
		unofficial:
			'Unofficial client. Aimaira is a trademark of its vendor; this project is unaffiliated with it and with any school.',
	},
	schools: {
		title: 'Is your school supported?',
		lead: 'These schools run Aimaira. Find yours to see its portal address and what BetterAimaira can do with it. The list is maintained by hand: it does not watch you, and it queries no portal.',
		searchLabel: 'Find your school',
		searchPlaceholder: 'Name, initials or group — ESGI, Eduservices, Nantes…',
		clear: 'Clear',
		checkTitle: 'Check your school before you download',
		checkLead:
			'Type your school’s name and you will know straight away whether BetterAimaira can read its portal. The search runs inside this page; no portal is queried.',
		checkHint: 'Maintained by hand, extended as each school is confirmed.',
		checkSeeAll: 'See all {count} schools',
		checkSheet: 'Open the full entry',
		countOne: 'school',
		countOther: 'schools',
		filterLabel: 'Filter by field',
		filterAll: 'All',
		categories: {
			arts: 'Performing arts',
			business: 'Business & management',
			communication: 'Communication & marketing',
			creation: 'Design & multimedia',
			droits: 'Law & political science',
			gestion: 'Accounting & finance',
			hotel: 'Hospitality & gastronomy',
			immobilier: 'Real estate',
			informatique: 'Computing & technology',
			inge: 'Engineering & science',
			mode: 'Fashion & craft',
			sante: 'Health & social care',
			sport: 'Sport',
			tourisme: 'Tourism',
		},
		statusLabel: 'Sign-in',
		status: {
			ready: {
				badge: 'Supported',
				body: 'The portal serves the username and password form BetterAimaira knows how to fill in.',
			},
			emailFirst: {
				badge: 'Unverified',
				body: 'The portal asks for your email address first and decides what to do with it server-side. That flow has not been tested from the app: try it, and tell us how it goes.',
			},
			sso: {
				badge: 'Not supported',
				body: 'Sign-in goes through an account outside the portal. BetterAimaira cannot authenticate against it.',
			},
			unknown: {
				badge: 'Address unknown',
				body: 'The school runs Aimaira, but its portal address has not been confirmed. Enter it yourself in the app.',
			},
		},
		portalLabel: 'Portal',
		groupNote: 'Shared {group} group portal.',
		websiteLabel: 'School website',
		emptyTitle: 'No school matches',
		emptyBody: 'Try the initials rather than the full name, or the name of the group your school belongs to.',
		missingTitle: 'Your school is not in the list?',
		missingBody: 'Then it probably does not run Aimaira, and BetterAimaira will have nothing to read. If you know it does, open an issue with its portal address and it will be added here.',
		sourceNote: 'Maintained by hand. Portal addresses were confirmed one by one, without ever querying anybody’s account.',
		cta: 'Download the app',
	},
	download: {
		title: 'Download BetterAimaira',
		lead: 'Three platforms have a published installer. On macOS and Linux, the local build is one command.',
		detecting: 'Detecting your device…',
		resolvingAsset: 'Reading the newest published release…',
		detectedFor: 'For your device:',
		detectedUnknown: 'Pick your platform',
		notMyDevice: 'This is not my device',
		versionLabel: 'Version',
		publishedLabel: 'Published',
		sizeLabel: 'Size',
		fetchFailed: 'Could not read the latest version from GitHub.',
		fetchFailedAction: 'Open the releases page',
		buildFromSource: 'Build from source',
		allPlatforms: 'All platforms',
		stepsLabel: 'Install',
		requirementLabel: 'Requirements',
		copySource: 'Copy the source URL',
		copied: 'URL copied',
		betaTitle: 'Prerelease',
		signatureTitle: 'Verified signature',
		betaBody:
			'Published versions are beta prereleases. The app checks the update feed three seconds after launch and offers the next one when it lands.',
		signatureNote:
			'The Windows installer ships with its minisign `.sig`, the signature the built-in updater verifies before replacing an installed version.',
		platforms: {
			windows: {
				name: 'Windows',
				summary: '64-bit NSIS installer, updates in place.',
				requirement: 'Windows 10 or 11, 64-bit.',
				steps: [
					'Download the `.exe` installer.',
					'Windows SmartScreen shows a warning: the installer is not signed with a commercial certificate. Click “More info”, then “Run anyway”.',
					'Follow the installer through.',
					'On first launch, paste your Aimaira portal address. Any page of the portal will do.',
				],
				note: 'Later updates install from inside the app: it checks the feed three seconds after launch.',
			},
			android: {
				name: 'Android',
				summary: 'Universal APK: `arm64-v8a`, `armeabi-v7a` and `x86_64` in one file.',
				requirement: 'Android 7.0 or newer.',
				steps: [
					'Download the APK.',
					'Open the downloaded file. Android asks you to allow installs from this source: grant it to your browser or file manager.',
					'Accept the system install prompt.',
					'On first launch, paste your portal address.',
				],
				note: 'Android compares `versionCode`, not version names: an APK whose code is not higher than the installed one is not seen as an update by the system.',
			},
			ios: {
				name: 'iOS',
				summary: 'Unsigned IPA, installed by AltStore or SideStore with your own Apple ID.',
				requirement: 'iOS 15 or newer, with AltStore or SideStore already installed.',
				steps: [
					'Install AltStore or SideStore on your iPhone.',
					'In the app, add the source below.',
					'Install BetterAimaira from that source.',
					'On first launch, paste your portal address.',
				],
				note: 'The IPA is built without code signing: AltStore and SideStore re-sign it with your account. On a free Apple account the certificate needs renewing every seven days.',
			},
			macos: {
				name: 'macOS',
				summary: 'No bundle published yet: the local build produces the `.app` and the `.dmg`.',
				requirement: 'Bun 1.2 or newer, Rust 1.80 or newer, and the Xcode command line tools.',
				steps: [buildCommands],
				note: 'The bundle is written to `src-tauri/target/release/bundle/`. Once installed it reads the same update feed as every other platform.',
			},
			linux: {
				name: 'Linux',
				summary: 'No package published yet: the local build produces the AppImage and the `.deb`.',
				requirement:
					'Bun 1.2 or newer, Rust 1.80 or newer, and the Tauri platform prerequisites (`libwebkit2gtk-4.1-dev` and its companions).',
				steps: [buildCommands],
				note: 'A saved password goes through the session’s Secret Service, so GNOME Keyring or KWallet depending on your desktop.',
			},
		},
	},
};

export const content: Record<Lang, Content> = { fr, en };

export function useContent(lang: Lang) {
	return content[lang];
}

/**
 * The slug each page answers on, per language. The pair is what makes the
 * language switch land on the twin of the page being read rather than the home
 * page — and the schools page is the reason this is a table and no longer one
 * shared string: it is `/ecoles` in French and `/en/schools` in English.
 */
const pageSlugs: Record<Lang, Record<PageId, string>> = {
	fr: { '': '', download: 'download', schools: 'ecoles' },
	en: { '': '', download: 'download', schools: 'schools' },
};

/** `/`, `/download` and `/ecoles` in French; `/en/…` in English. */
export function localePath(lang: Lang, page: PageId = ''): string {
	const prefix = lang === defaultLang ? '/' : `/${lang}/`;
	return `${prefix}${pageSlugs[lang][page]}`;
}
