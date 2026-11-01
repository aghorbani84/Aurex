# Development

## Layout

- `crates/aurex-core`: identities, metadata, capabilities, errors, navigation/tab state.
- `crates/aurex-tasks`: bounded executor, real Future, cancellation and progress.
- `crates/aurex-fs`: async provider API and Linux x86-64 descriptor-relative backend.
- `apps/aurex-cli`: tested command-line vertical slice with native arguments.
- `apps/aurex-app`: separate Dioxus native desktop source/lockfile/assets; native compilation verified, runtime not accepted.
- `scripts/check.sh`: reproducible core checks; `benchmark.py`: private fixtures.

Run `scripts/check.sh` from the repository root. It does not compile the GUI; run its
separate commands in BUILDING.md. The root Cargo.lock has no registry dependencies;
UI updates must update its separate lockfile and run its own tests/audits.

## Design changes

Prefer a complete provider/engine/test/UI slice over empty feature crates. Keep all
blocking work on the bounded scheduler, never in a Dioxus event handler's synchronous
body. Await tasks in components; `.wait()` is CLI/test-only. Do not cancel by simply
dropping a Future. Avoid blanket shell execution and path-based URI coercion.

Never turn an OS refusal into an apparently successful result. New destructive
operations require scope previews, accurate partial results and deterministic failure
tests. Add real capability discovery before enabling platform-specific controls.

## State

Current GUI bookmarks/history/theme/tabs/log are session-only. There is no config
file/database/session restore. Do not add ad-hoc plaintext credential settings.
Persistence needs atomic/versioned writes or transactional migrations, corruption
handling and regression tests. SQLite is not justified until that slice exists.

## Testing practices

Use exclusive temporary fixture roots. Reuse no existing data. Keep regression tests
platform-gated and record skipped/blocked native cases honestly. Benchmarks include
process overhead and OS caches; never extrapolate them to UI or production speed.
See TEST_MATRIX.md and PERFORMANCE.md. Actual GitHub core/native Linux/Windows
builds and CLI artifact/runtime checks passed. Strict GUI deny still fails; no GUI
runtime, release or installer acceptance is claimed. Executed run/artifact links and
source revisions are in reports/AUREX_ENGINEERING_REPORT.md.
