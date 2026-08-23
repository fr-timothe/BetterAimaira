# Performance

BetterAimaira measures performance separately for the shared Svelte frontend and each Tauri target. A change is considered an optimization only when the relevant metric improves without regressions in memory, energy use, package size, or correctness.

## Local baseline

Run the verification checks and frontend size report:

```sh
bun run check
bun run build
bun run perf:report
cd src-tauri
cargo test
cargo clippy --all-targets -- -D warnings
```

Record cold and warm build durations separately. `desktop:build` preserves Cargo cache; use `clean:cache` when a clean baseline is required.

## Shared metrics

- Startup to first paint, interaction readiness, and schedule readiness.
- Request count, transferred bytes, and response duration per Aimaira endpoint.
- HTML/JSON parsing duration and SQLite transaction duration.
- JavaScript and CSS raw and gzip size from `perf:report`.
- Idle CPU, peak memory, resize frame pacing, and network requests after ten calendar navigations.

Logs must never include portal credentials, cookies, private URLs, or student records.

## Platform matrix

| Target | Runtime tools | Release artifact |
|---|---|---|
| Windows | WebView2 DevTools, Windows Performance Recorder, Process Explorer | NSIS |
| macOS | Web Inspector and Instruments on Apple Silicon | app and DMG |
| Linux | WebKitGTK inspector, `perf`, and `/usr/bin/time -v` on Wayland | DEB and AppImage |
| Android | Android Studio Profiler and `adb shell dumpsys meminfo` on a test device | APK for diagnostics, AAB for release |
| iOS | Instruments on a physical iPhone; Simulator for functional coverage only | signed archive |

Desktop measurements must be captured on their target OS; Windows results are not evidence for macOS or Linux.

## Current safeguards

- Authenticated views are loaded on demand instead of bundling in the initial login chunk.
- Portal resources and schedule ranges use a 5-minute in-memory cache with in-flight request deduplication.
- Refreshes keep current data visible during background revalidation.
- Calendar events are indexed by local day.
- Shared clock timer pauses while the webview is hidden.
- PDF streaming stops immediately when the 25 MiB limit is exceeded.
- Native credential stores are selected per target.
- Release bundles and build cleanup are configured per platform.

## Windows synthetic reference

Captured on local production preview on 2026-08-21 without CPU or network throttling:

- LCP: 269 ms.
- CLS: 0.00.
- Production output: 674.82 KiB total; JavaScript and CSS total 169.82 KiB gzip.
- Login route entry chunk: 0.07 KiB after code splitting, down from the monolithic 328.83 KiB route chunk.
- Largest deferred JavaScript chunk: 67.74 KiB raw, 17.46 KiB gzip.

These localhost figures serve as regression baselines.
