// The site serves the repository's own brand and showcase assets. Copying them
// into `public/media/` at build time keeps `assets/` the single source of truth:
// a screenshot replaced there reaches the site on the next build, and the
// repository never carries two copies of the same file.
//
// `assets/showcase/betteraimaira-demo.webp` is deliberately not in the list. It
// weighs 19 MB, which no landing page can spend on a hero.
//
// The list holds only what a page actually references. `logo-lockup.svg` is not
// here because the lockup is rendered as real text (see `Lockup.astro`), and no
// screenshot is copied unless `content.ts` names it.
import { cp, mkdir, rm, stat } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const siteRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = resolve(siteRoot, '..');
const target = join(siteRoot, 'public', 'media');

/** @type {Array<[string, string]>} source relative to the repository root, then destination name */
const files = [
	['static/favicon.svg', 'favicon.svg'],
	['static/favicon.png', 'favicon.png'],
	['assets/showcase/screenshot-1.png', 'screenshot-today.png'],
	['assets/showcase/screenshot-3.png', 'screenshot-week.png'],
	['assets/showcase/screenshot-4.png', 'screenshot-grades.png'],
	['assets/showcase/betteraimaira-presentation.mp4', 'presentation.mp4'],
	['assets/showcase/presentation-poster.webp', 'presentation-poster.webp'],
];

await rm(target, { recursive: true, force: true });
await mkdir(target, { recursive: true });

const missing = [];
for (const [source, name] of files) {
	const from = join(repoRoot, source);
	try {
		await stat(from);
	} catch {
		missing.push(source);
		continue;
	}
	await cp(from, join(target, name));
}

if (missing.length > 0) {
	// A missing asset is a broken page, not a warning to scroll past.
	console.error(`copy-media: missing source assets:\n  ${missing.join('\n  ')}`);
	process.exit(1);
}

console.log(`copy-media: ${files.length} assets copied into public/media/`);
