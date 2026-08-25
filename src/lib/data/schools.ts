/**
 * The schools known to run an Aimaira portal, and the search behind the picker
 * shown before the login form.
 *
 * The directory is bundled rather than fetched: it is 26 kB, it never changes
 * between two releases, and a picker that needs the network to list names would
 * be useless in the one place a student opens it — a phone with a flaky campus
 * connection. Only the logos are remote, from the site this app ships from, and
 * every one of them is allowed to fail.
 *
 * See `assets/schools/README.md` for where the list comes from and how to
 * extend it.
 */
import directory from '../../../assets/schools/schools.json';

/**
 * The sign-in form a portal actually serves.
 *
 * `password` is the one `src-tauri/src/aimaira.rs` posts to: username and
 * password in a single step. `email-first` asks for the address alone and
 * decides what to do with it server-side, and `sso` hands the reader to an
 * outside identity provider with no password field at all.
 */
export type PortalLogin = 'password' | 'email-first' | 'sso';

export interface School {
  /** Slug, and the logo filename. */
  id: string;
  name: string;
  category: string;
  /**
   * The portal to sign in to, or `null` when it is not known. A null is not a
   * missing feature: the school is an Aimaira client either way, and the picker
   * hands those readers to the address field instead of guessing.
   */
  portalUrl: string | null;
  /** The form that portal serves; `null` alongside a null `portalUrl`. */
  portalLogin: PortalLogin | null;
  /** Set when `portalUrl` was inherited from the school's group, not its own name. */
  group?: string;
  website?: string;
}

export const schools: School[] = directory as School[];

/** Where the site publishes the logos. Kept in sync with `site/scripts/copy-media.mjs`. */
const LOGO_BASE = 'https://betteraimaira.montfrond.work/media/schools';

export function schoolLogoUrl(school: School): string {
  return `${LOGO_BASE}/${school.id}.webp`;
}

/**
 * The initials drawn while a logo is loading, and in place of one that never
 * arrives. Digits count as words so `89 - L'École…` reads as `89`.
 */
export function schoolInitials(school: School): string {
  const words = school.name
    .replace(/[^\p{L}\p{N}\s-]/gu, ' ')
    .split(/[\s-]+/)
    .filter(Boolean);
  const numeric = words[0]?.match(/^\d+$/);
  if (numeric) return numeric[0].slice(0, 2);
  const significant = words.filter(
    (word) => word.length > 2 || word === word.toUpperCase()
  );
  // A one-word name has no second initial to take, and a lone letter is not a
  // mark: `Aflokkat` reads as `AF`, not as `A`.
  if (significant.length < 2) {
    return (significant[0] ?? words[0] ?? school.name).slice(0, 2).toUpperCase();
  }
  return significant
    .slice(0, 2)
    .map((word) => word[0].toUpperCase())
    .join('');
}

function normalize(text: string): string {
  return text
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .toLowerCase();
}

/**
 * Every school flattened to one searchable string, built once.
 *
 * The group name is in there on purpose: a student who knows their school
 * belongs to Eduservices but not that the portal is named after it should still
 * find their entry, and vice versa.
 */
const haystacks = new Map<string, string>(
  schools.map((school) => [
    school.id,
    normalize(
      [school.name, school.group ?? '', school.id.replace(/-/g, ' '), school.portalUrl ?? '']
        .join(' ')
    ),
  ])
);

/**
 * Schools matching every word of the query, in directory order.
 *
 * Word-by-word rather than substring: `ecole design nantes` has to find
 * `L'École du Design Nantes Atlantique`, whose words the reader will not type
 * in the order the school wrote them.
 */
export function searchSchools(query: string): School[] {
  const words = normalize(query).split(/\s+/).filter(Boolean);
  if (words.length === 0) return schools;
  return schools.filter((school) => {
    const haystack = haystacks.get(school.id) ?? '';
    return words.every((word) => haystack.includes(word));
  });
}
