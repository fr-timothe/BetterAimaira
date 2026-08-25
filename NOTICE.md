# Notice

`BetterAimaira` is released under the [GPL-3.0](LICENSE). That licence covers the
source code written for this repository. It does not, and cannot, cover the
third-party material listed below, which is reproduced here under the terms of
its own licence or under a right of reference. Anyone redistributing this
project keeps this file with it.

## Aimaira

Aimaira is a product of its vendor, and `Aimaira` is their trademark. This
project is an independent client. It is not published, endorsed, supported or
reviewed by that vendor, and it is not affiliated with any school running an
Aimaira portal.

The name appears in this project only to say what the client connects to, which
is the one way to say it: a reader looking for a client for their Aimaira portal
has no other word for it. No vendor logo, artwork, stylesheet or source code is
reproduced here. Every request the app makes goes to the portal address the
student enters, with the student's own credentials, and it only ever reads.

## School names and logos

`assets/schools/` carries the name, logo, website and category of each school
known to run an Aimaira portal, and the site and the app both render that list.

Those names and logos belong to the schools themselves. They are not licensed
under the GPL-3.0, they are not part of this project's own assets, and nothing
here grants any right over them. They are reproduced for one purpose: telling a
student whether this client works with their school's portal, and which address
that portal has. A logo is shown next to its own school's entry, never as an
endorsement of this project by that school.

A school that would rather not appear can ask for its entry to be removed by
opening an issue, and it will be, logo and all.

## Bundled third-party software

| Component | Licence |
| --- | --- |
| [Inter](https://rsms.me/inter/) (`@fontsource-variable/inter`) | SIL Open Font License 1.1 |
| [Lucide](https://lucide.dev/) (`lucide-svelte`) | ISC |
| [Svelte](https://svelte.dev/), [SvelteKit](https://svelte.dev/docs/kit), [Vite](https://vite.dev/), [Tailwind CSS](https://tailwindcss.com/) | MIT |
| [Tauri](https://v2.tauri.app/) and its plugins | MIT or Apache-2.0 |
| Rust crates listed in `src-tauri/Cargo.toml` | MIT or Apache-2.0, per crate |

Each carries its own licence text in its own distribution; this table names them
rather than restating them.
