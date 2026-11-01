# Architecture

## Current dependency graph

```text
CLI / Dioxus app ───→ aurex-fs ───→ aurex-core
        │                │         (identities, errors, navigation)
        └────────┬───────┘
                 ↓
            aurex-tasks
       (executor, no core dependency)
```

Only crates with real responsibilities exist. There are no placeholder remote,
archive, APK or plugin crates. Rust 2024 edition, stable 1.88.0. Dioxus 0.7.10 is a
renderer, never the filesystem abstraction.

## Providers and identities

`Location` distinguishes native root-relative `PathBuf`, document URI, archive
entry, and remote identities. Only `Local` has an implemented provider. Other
identities explicitly return `Unsupported`; URIs are never coerced into paths.
Native `OsString`/`PathBuf` survive round trips, including invalid UTF-8 filenames.
Lossy conversion is display/filter-only, not operational identity.

`FilesystemProvider: Send + Sync` provides capability discovery and awaitable
listing, bounded text reads, and operations. `Task<Result<T>>` separates executor
failure from filesystem failure. A scheduler-full error is immediate backpressure,
not an unbounded queued promise. Capability discovery is operation-level; the OS
still enforces access and filesystem-specific constraints on every invocation.

## Executor

`aurex-tasks` owns a bounded channel and 1..32 blocking workers. The CLI waits on a
condition variable; Dioxus awaits a real `Future` using a registered `Waker`.
Polling never runs filesystem I/O. Worker jobs are panic-isolated. Cancellation is
cooperative and explicit; dropping a future does NOT cancel its job. Submitted
jobs drain when the last scheduler sender is dropped; workers are not joined.
Process exit may interrupt jobs, so copy staging must be cleanup-safe independently.

The dependency-free executor and manually implemented `std::error::Error` are a
bootstrapping decision for the restricted environment, not a claim that the full
requested Tokio/Serde/thiserror/tracing stack is complete. Tokio 1.51.6 supplies
GUI timers; a future journal/config slice should introduce audited serialization,
structured tracing and storage only alongside actual persistence requirements.

## Linux boundary

`aurex-fs/src/linux.rs` contains the narrowly documented unsafe syscall boundary,
restricted to Linux x86-64. Parent paths are walked from the granted descriptor
using `openat(O_DIRECTORY|O_NOFOLLOW)`. The root is also opened component-by-component
following initial canonicalization. Local identities reject absolute paths, `..`
and NUL; paths are capped at 16 KiB and filters at 1024 bytes. Regular files use `O_NONBLOCK|O_NOFOLLOW`; FIFOs are rejected without waiting.
Listing uses procfs descriptor paths and `symlink_metadata`.

Copies use anonymous `O_TMPFILE` descriptors, two 128 KiB buffers, complete readback,
source identity/size/mtime/ctime checks, file fsync and no-replace `linkat` publication
through a controlled procfs FD link. Rename/move uses `renameat2(RENAME_NOREPLACE)`;
EXDEV is explicitly unsupported. See operation semantics and security limitations.

## UI state and bounded resources

The standalone desktop workspace imports exactly the same core path dependencies.
No browser is given arbitrary filesystem RPC or shell execution. The Dioxus source
uses signals, async resources, native operations and CSS tokens; no fake fixtures.
Provider calls run on the shared worker pool. Resource inputs are memoized to avoid
rescanning just because selection changes.

Current bounded listing collects/sorts at most 50,000 entries before returning
500-entry pages. This is **not incremental loading or virtualization**. Filters
are Unicode-lowercased substring matches, not regex, recursive search or an index.
History is limited to 64 locations per tab and 32 tabs per pane. UI history is
100 session records, with eight displayed; no durable audit log exists yet.

## Platform boundaries and planned extensions

Windows must use native handles/reparse-point controls and correct no-replace/
sharing semantics; no UNIX API substitution. Android must use document identifiers,
provider flags, persisted grants and revocation-aware SAF operations. Archives need
separate quotas/traversal protection. Remote authentication belongs in native
credential storage, not settings. Plugins/AI require explicit capability brokers.
None of these planned providers can currently be instantiated.

## Persistence and packaging

No SQLite or saved sessions yet. Validated unsigned Linux CLI engineering archive packaging exists; GUI/platform
installers and signing infrastructure do not. The two
Cargo.lock files pin actual applications; the root offline core is intentionally
isolated from the native renderer's dependency graph. CI gates cover compilation,
tests and auditing, but their execution and platform runtime are separate evidence.
