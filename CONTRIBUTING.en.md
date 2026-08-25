# Contributing to BetterAimaira

Thanks for the interest. This document says what a contribution has to respect
to be mergeable, and what the project turns down however good the code is.

> 🇫🇷 [Version française](CONTRIBUTING.md)

## Read before writing code

| Document | What it settles |
| --- | --- |
| [PRODUCT.md](PRODUCT.md) | Users, scope, constraints, product principles |
| [DESIGN.md](DESIGN.md) | Tokens applied at the element, shared primitives, the six honest states |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Rust layout, Tauri IPC, Svelte state, cache, security boundaries |
| [docs/BACKEND_API.md](docs/BACKEND_API.md) | Every Tauri command, its payload contract, the error codes |
| [NOTICE.md](NOTICE.md) | What in this repository does not belong to the project |

## The boundaries that do not move

A pull request crossing one of these is turned down, however clean, however
often users ask for it. They are the promises the README and the site make, and
they are not renegotiated case by case.

- **Read-only.** No write route to the portal. No profile, no password, no
  questionnaire, no payment, no administrative request.
- **No third party on the network.** The app talks to the portal the student
  configured and to the project's update feed. Nothing else: no telemetry, no
  analytics, no relay, no push service, no CDN.
- **HTTPS only.** An HTTP portal is rejected before any credential is sent.
- **Secrets stay in Rust.** Session cookies live in backend memory and never
  cross the IPC boundary. Passwords live in the OS keychain, never in SQLite or
  in frontend preferences.
- **Portal HTML does not leave Rust.** Remote content is turned into plain text
  before it reaches the frontend, and treated as untrusted throughout.
- **The automation bridge stays out of distributed builds.** It lives behind the
  `dev-automation` Cargo feature and a `#[cfg(debug_assertions)]` guard.

## Setup

Full prerequisites and commands in the [README](README.en.md#development). The
minimum:

```bash
bun install
bun run desktop:dev
```

## What a PR has to satisfy

- **Every visible string goes through Paraglide.** Strings live in
  `messages/fr.json` and `messages/en.json`, both of them, never inline in a
  component. A key added to one locale and not the other fails the build.
- **No locally declared colour or radius.** Tokens come from `src/app.css`; see
  [DESIGN.md](DESIGN.md).
- **Every data surface distinguishes its states** — loading, empty, stale,
  offline, expired, error. A view that guesses is a bug.
- **No real data in fixtures.** No credential, no cookie, no student name, no
  PDF content, no portal screenshot. Tests use example hosts, the way
  `src-tauri/src/` already does.
- **The site follows, if the change touches it.** `site/` has its own
  `bun run check`.

The verification block has to run clean, no error and no warning:

```bash
bun run check
bun run build
cd src-tauri
cargo test
cargo clippy --all-targets -- -D warnings
```

## Commits and pull requests

- Messages in English, imperative, one short summary line.
- Conventional commit types (`feat`, `fix`, `docs`, `ci`, `refactor`…): release
  notes are built from them, and a mistyped commit lands in the wrong group.
  Read the rendering back with `bun run release:notes` before tagging.
- One PR, one subject. A fix and a refactor in the same diff review badly and
  revert worse.
- A screenshot for anything visible, in both languages when the text moves.

## Adding a school

The school directory has its own rules, chief among them never writing a guessed
portal address: see [assets/schools/README.md](assets/schools/README.md). The
names and logos it carries belong to the schools, not to the project —
[NOTICE.md](NOTICE.md).

## Reporting a bug, asking a question, disclosing a vulnerability

See [SUPPORT.en.md](SUPPORT.en.md). A security vulnerability does not go in a
public issue.
