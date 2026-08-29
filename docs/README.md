<div align="center">
  <img src="../assets/mark.svg" height="120">
<h1>BetterAimaira Documentation</h1>
<p><strong>Everything the code does not say by itself: boundaries, contracts, and the rules a change has to respect.</strong></p>
<br>
<p>
<a href="../README.md"><img src="https://img.shields.io/badge/%E2%86%90_Project_README-000000?style=for-the-badge" alt="Project README"></a>
<a href="ARCHITECTURE.md"><img src="https://img.shields.io/badge/Architecture-000000?style=for-the-badge" alt="Architecture"></a>
<a href="BACKEND_API.md"><img src="https://img.shields.io/badge/Backend_API-000000?style=for-the-badge" alt="Backend API"></a>
</p>
</div>

## Read this first

| Document | Covers | Status |
|---|---|---|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Rust backend layout, Tauri IPC, Svelte runes state, cache strategy, security and translation boundaries | implemented |
| [BACKEND_API.md](BACKEND_API.md) | Every Tauri command, its payload contract, document downloads, error codes | implemented |
| [APP_STRUCTURE_AND_PLATFORMS.md](APP_STRUCTURE_AND_PLATFORMS.md) | Adaptive layout decisions, per-platform expression, native boundaries, release matrix | mixed |
| [DESIGN_SYSTEM.md](DESIGN_SYSTEM.md) | Tokens, palette, breakpoints, adaptive layout rules | implemented |
| [PERFORMANCE.md](PERFORMANCE.md) | Baseline commands, platform profiling tools, budgets, current safeguards | implemented |
| [INTEGRATIONS.md](INTEGRATIONS.md) | iCal export, desktop and mobile widgets | planned |

Two more documents live at the repository root because they are not scoped to the backend:

- [DESIGN.md](../DESIGN.md) — how the tokens are applied at the element, which shared primitive owns
  what, and the six honest states every data surface has to distinguish.
- [PRODUCT.md](../PRODUCT.md) — users, scope, constraints and product principles.

## Conventions

- **Status labels.** A section that describes something not yet built says so in its own heading or
  first line. `mixed` above means the document holds both; `planned` means none of it ships yet.
- **English.** Documentation, code, identifiers and commit messages are in English.
- **No portal data.** No raw portal capture, personal record, cookie value or real HTML fixture ever
  enters this repository. Reconnaissance notes stay outside it.
- **Relative paths.** Documents link with repository-relative paths. The one exception is
  `keystore.properties`, which Gradle requires as an absolute machine-local path.
- **No `Updated:` stamps.** Git history is the source of truth for when a document changed.
