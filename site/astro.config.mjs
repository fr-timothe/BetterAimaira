// The site ships to the root of the custom domain already carried by the
// `gh-pages` branch (`CNAME`), so there is no repository base path. That branch
// also holds the update feed the installed app polls (`updates/<channel>/`),
// which is why the deploy workflow commits into it instead of switching Pages
// to the Actions source.
import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
	site: 'https://betteraimaira.montfrond.work',
	i18n: {
		defaultLocale: 'fr',
		locales: ['fr', 'en'],
		routing: { prefixDefaultLocale: false },
	},
	integrations: [
		sitemap({
			i18n: {
				defaultLocale: 'fr',
				locales: { fr: 'fr-FR', en: 'en-GB' },
			},
		}),
	],
	vite: {
		plugins: [tailwindcss()],
	},
});
