// Resolving the current release, in the browser.
//
// `/releases/latest` is deliberately NOT used: GitHub resolves that endpoint to
// the newest release *not* flagged as a prerelease, and this project ships betas
// only. The release workflow's own comment says the same thing about the update
// feed. So the list endpoint is read and the newest non-draft release wins.

export const owner = 'fr-timothe';
export const repoName = 'BetterAimaira';

export type PlatformId = 'windows' | 'android' | 'ios' | 'macos' | 'linux';

export interface ReleaseAsset {
	name: string;
	url: string;
	size: number;
}

export interface Release {
	tag: string;
	version: string;
	publishedAt: string | null;
	prerelease: boolean;
	assets: ReleaseAsset[];
}

interface ApiAsset {
	name?: unknown;
	browser_download_url?: unknown;
	size?: unknown;
}

interface ApiRelease {
	tag_name?: unknown;
	draft?: unknown;
	prerelease?: unknown;
	published_at?: unknown;
	assets?: unknown;
}

/** Only these three are published as installable assets; the rest are manifests and signatures. */
const assetMatchers: Partial<Record<PlatformId, RegExp>> = {
	windows: /-x86_64\.exe$/,
	android: /-universal\.apk$/,
	ios: /-arm64\.ipa$/,
};

export async function fetchLatestRelease(): Promise<Release> {
	const response = await fetch(`https://api.github.com/repos/${owner}/${repoName}/releases?per_page=10`, {
		headers: { Accept: 'application/vnd.github+json' },
	});
	if (!response.ok) {
		throw new Error(`GitHub API responded ${response.status}`);
	}

	const payload: unknown = await response.json();
	if (!Array.isArray(payload)) {
		throw new Error('GitHub API returned an unexpected payload');
	}

	const newest = (payload as ApiRelease[]).find((entry) => entry.draft !== true && typeof entry.tag_name === 'string');
	if (!newest) {
		throw new Error('No published release found');
	}

	const tag = String(newest.tag_name);
	const rawAssets = Array.isArray(newest.assets) ? (newest.assets as ApiAsset[]) : [];

	return {
		tag,
		version: tag.replace(/^v/, ''),
		publishedAt: typeof newest.published_at === 'string' ? newest.published_at : null,
		prerelease: newest.prerelease === true,
		assets: rawAssets
			.filter((asset) => typeof asset.name === 'string' && typeof asset.browser_download_url === 'string')
			.map((asset) => ({
				name: String(asset.name),
				url: String(asset.browser_download_url),
				size: typeof asset.size === 'number' ? asset.size : 0,
			})),
	};
}

export function assetFor(release: Release, platform: PlatformId): ReleaseAsset | null {
	const matcher = assetMatchers[platform];
	if (!matcher) return null;
	return release.assets.find((asset) => matcher.test(asset.name)) ?? null;
}

/** The AltStore/SideStore source manifest, taken from the release rather than the feed when present. */
export function altStoreAsset(release: Release): ReleaseAsset | null {
	return release.assets.find((asset) => asset.name === 'altstore.json') ?? null;
}

export function detectPlatform(): PlatformId | null {
	if (typeof navigator === 'undefined') return null;

	const ua = navigator.userAgent;
	// iPadOS reports itself as a Mac, and the touch-point count is the only
	// reliable separator left.
	const iPadOnDesktopUa = /Macintosh/.test(ua) && navigator.maxTouchPoints > 1;

	if (/Android/i.test(ua)) return 'android';
	if (/iPhone|iPod|iPad/i.test(ua) || iPadOnDesktopUa) return 'ios';
	if (/Windows/i.test(ua)) return 'windows';
	if (/Macintosh|Mac OS X/i.test(ua)) return 'macos';
	if (/Linux|X11|CrOS/i.test(ua)) return 'linux';
	return null;
}

export function formatSize(bytes: number, locale: string): string {
	if (!bytes) return '—';
	const mega = bytes / 1024 / 1024;
	const value = new Intl.NumberFormat(locale, {
		minimumFractionDigits: mega < 10 ? 1 : 0,
		maximumFractionDigits: mega < 10 ? 1 : 0,
	}).format(mega);
	return `${value} ${locale.startsWith('fr') ? 'Mo' : 'MB'}`;
}

export function formatDate(iso: string | null, locale: string): string {
	if (!iso) return '—';
	const parsed = new Date(iso);
	if (Number.isNaN(parsed.getTime())) return '—';
	return new Intl.DateTimeFormat(locale, { day: 'numeric', month: 'long', year: 'numeric' }).format(parsed);
}
