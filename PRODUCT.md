# Product

<!-- impeccable:product-schema 1 -->

## Platform

adaptive

## Users

The first version targets students of a school that runs the Aimaira portal. Each user enters their
own portal address on the login screen; nothing assumes they paste the exact home page.

## Product purpose

BetterAimaira is a Tauri client for reading Aimaira student information. The first milestone covers
portal sign-in, loading the real schedule, the local cache, and a Today screen that is actually
usable.

The product answers the need when the student immediately sees their current or next course, its
room, and how fresh the data is — including offline.

## Positioning

BetterAimaira replaces navigating the web portal with a local, responsive student interface focused
on daily information. The client talks directly to the portal the user selected, with no cloud
relay, and keeps useful data available offline.

## Operating context

- Used on a phone between two classes, and on a computer for a denser schedule view.
- The reference portal is a single Aimaira instance, called the reference instance.
- The portal URL field is empty on first sign-in. The user may paste a deep or parameterized Aimaira
  URL; the application reduces it to its HTTPS origin before connecting.
- Aimaira is built on HTML pages, ASP.NET forms, session cookies, and a schedule endpoint that
  returns JSON inside a response declared as HTML.

## Capabilities and constraints

- The first version is strictly read-only.
- Current path: portal configuration, authentication, the real schedule, and the Today/Schedule
  screens.
- A complete translation system from the first milestone, with French and English as the initial
  languages, through Paraglide JS.
- One Svelte 5 interface and one Rust Tauri 2 core shared across platforms.
- Tauri application only. No web or PWA version is planned.
- Credentials, cookies and student data never leave the device, apart from direct requests to the
  chosen portal.
- Cookies stay in the Rust backend and are not exposed to the frontend.
- Persistent credentials use each system's secure store. No plaintext password in SQLite or in
  frontend preferences.
- HTTPS is mandatory for any portal. HTTP connections are refused before credentials are sent.
- Opaque identifiers stay strings.
- Cached data displays its sync time and freshness state.
- Billing, payments, administrative procedures and remote writes are out of scope.
- Compatibility with other Aimaira instances will be assessed after the reference instance is
  validated.
- No demo data appears in the authenticated surfaces of this vertical slice.

## Brand commitments

- Name: BetterAimaira.
- Product inspiration: the Papillon principles of direct information access, platform adaptation,
  and transparency about estimated or stale data.
- The visual identity is BetterAimaira's own. Papillon is a methodological reference, not a graphic
  model to reproduce.
- The FL-Theme published at `https://tweakcn.com/themes/cmq57ht7w000204l2axo6ho9v` is the approved
  visual base.
- The first milestone ships the light theme. Dark mode comes in a later step.
- Core features stay reachable at every window size.
- The public showcase site (`site/`) follows the category-standard product-landing convention,
  calibrated on `papillon.bzh`, Raycast/Linear, Tauri/Zed and Proton/Signal. This is a standing
  preference, chosen deliberately over a composition of its own: that surface spends its budget on
  finish, not on invention.

## Evidence on hand

- Anonymized local reconnaissance of the portal, kept outside the repository.
- Authentication and route notes, kept outside the repository.
- Target architecture: `docs/ARCHITECTURE.md` and `docs/APP_STRUCTURE_AND_PLATFORMS.md`.
- Initial visual system: `docs/DESIGN_SYSTEM.md` and `src/app.css`.
- No raw portal capture, personal data, cookie value or real HTML fixture may be added to the
  repository.

## Product principles

1. Present the useful information with a minimum of actions.
2. Show the cache immediately, then refresh in the background without blocking the interface.
3. Make freshness, offline mode and portal errors visible.
4. Keep secrets and student data local.
5. Adapt density and interaction to the device without removing an essential feature.

## Accessibility and inclusion

- Full keyboard navigation and visible focus on desktop.
- Touch targets of at least 44px on mobile.
- No state communicated by colour alone.
- Support for `prefers-reduced-motion`, safe areas, and text zoom without truncation.
