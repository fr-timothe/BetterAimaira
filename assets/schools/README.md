# School directory

The list of schools that run an Aimaira portal, plus one logo each. It feeds two
things: the school picker the app shows before the login form, and the
compatibility page on the site (`/ecoles`, `/en/schools`).

Nothing here is fetched at runtime. The app bundles `schools.json`, the site
serves `logos/` from `/media/schools/`, and both only change when this directory
does. That is deliberate: a student's portal address is not something to
discover by probing their network.

## Files

| File | What it is |
| --- | --- |
| `schools.json` | The directory itself, sorted by name, accent-insensitive. |
| `logos/<id>.webp` | One logo per entry, 240×120, transparent, trimmed and centred. |

## `schools.json` entries

```jsonc
{
  "id": "acfa-multimedia",        // slug; also the logo filename
  "name": "ACFA Multimédia",      // as the school writes it
  "category": "creation",         // one of the keys below
  "portalUrl": "https://eduservices.myintranet.online/", // or null
  "group": "Eduservices",         // only when portalUrl comes from the group
  "website": "https://www.acfamultimedia.com/" // optional
}
```

`portalUrl` is `null` for a school whose portal address is not known. The entry
still belongs here: the school is an Aimaira client, so the app works — the
student pastes the address themselves. The picker says exactly that instead of
guessing an address that would fail to sign in.

`group` marks an inherited address: the school's group runs one portal for all
its schools, and this entry was mapped through the group rather than confirmed
against the school's own name. It is a weaker claim than a bare `portalUrl`, and
the site shows it as such.

Categories: `arts`, `business`, `communication`, `creation`, `droits`,
`gestion`, `hotel`, `immobilier`, `informatique`, `inge`, `mode`, `sante`,
`sport`, `tourisme`. Labels for them live in `site/src/i18n/content.ts` and in
`messages/*.json`, so a new key needs a label in both.

## Where it came from

The names, logos, websites and categories were taken once from Aimaira's own
client page, <https://aimaira.fr/clients>. The portal addresses were not: each
one was confirmed by loading `https://<host>/login?ReturnUrl=%2F` and checking
the page carries Aimaira's `__RequestVerificationToken` field, the same marker
`src-tauri/src/aimaira.rs` uses to reject a portal that is not Aimaira. 39
distinct portals were confirmed that way, covering 104 of the 129 schools.

## Updating it

By hand, and only from something you have checked:

1. Add or edit the entry in `schools.json`, keeping the file sorted by name.
2. Confirm any new `portalUrl` by opening `<portalUrl>login?ReturnUrl=%2F` and
   seeing an Aimaira login form. If you cannot, leave `portalUrl` as `null`
   rather than writing a guess.
3. Drop the logo in `logos/<id>.webp` at 240×120 with a transparent background,
   trimmed of its margins and centred. Logos are drawn on a light chip in both
   the app and the site, so a dark-ink logo on transparency is fine; a logo that
   is white on transparency is not.
4. Run `bun run check` at the repository root and `bun run check` in `site/`.
