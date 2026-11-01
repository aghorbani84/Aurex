# Implementation plan

Updated 2026-11-01. Status labels are acceptance-based, not feature-count marketing.

## Baseline (verified)

Repository began at `e437dd825563647d06cca80b6db3240e20eca44e` with README and MIT
LICENSE only. No application, tests, CI, Rust, Cargo, Dioxus CLI or native platform
tooling was present. Working branch remains `arena/c8d70e1e-aurex`.

## Phase 0 — partial

- [x] Inspect repository; preserve original license and unrelated files.
- [x] Verify stable Dioxus 0.7.10 release/tagged manifests and relevant APIs.
- [x] Read Yazi scheduler/worker architecture; inspect traditional workflow references.
- [x] Define provider identities/capabilities/error contracts and test strategy.
- [x] Build minimal dependency-free Rust core and CLI on Linux x86-64.
- [x] Generate both Cargo.lock files (GUI uses Rust-compatible resolver 3).
- [ ] Finish official MT Manager documentation research (network blocked).
- [ ] Compile and runtime-validate a minimal Dioxus desktop app.
- [ ] Validate Windows and Android toolchains and a real application on each.

## First vertical slice — Linux local file tools (accepted within stated scope)

- [x] Real browsing, name filter/sort, metadata, hidden files, bounded pages.
- [x] Root-relative identity and descriptor-anchored no-symlink traversal.
- [x] Create directory, regular-file copy, same-filesystem move/rename, delete
  file/link/empty folder; no overwrite; fail/skip/keep-both transfer policies.
- [x] Bounded worker pool with awaitable completion and queue backpressure.
- [x] Streaming copy/readback, anonymous staging, cancellation/progress contracts.
- [x] Bounded UTF-8 preview with special-file/binary/size safeguards.
- [x] CLI end-to-end regression tests; format/lint/test/release build.
- [x] Bounded navigation/tab model; security and operational limitations documented.

## Phase 1 desktop + Android — not accepted

The graphical source is connected to real providers, but cannot be accepted based
on code inspection alone. Next work, in order:

1. Provision a host with official Rust, crates.io access, GTK/WebKit and a display.
   Run the GUI CI check and fix **all** compiler/lint issues, then execute the app.
2. Add real WebView UI smoke tests: independent panes/history/tabs, stale-result
   races, non-UTF names, selection, conflict dialogs, progress/cancel, permission
   errors, previews, focus return/traps and keyboard navigation. Test active-job
   shutdown and add an unsaved/active-operation guard before daily use.
3. Add serializable configuration, atomic session persistence, schema/version
   handling, recovery and durable operation audit records. Introduce tracing and
   typed serialization/error dependencies with verified licenses and tests.
4. Implement OS trash with restore IDs. Keep permanent delete opt-in.
5. Design recursive operation planning with bounded traversal and per-file outcomes;
   add source-mutation, interrupted write/fsync/disk-full injection tests. Implement
   cross-filesystem move only with committed verified copy + identity-safe source
   removal and accurate partial-failure states. No atomicity claim across volumes.
6. Implement Windows handle/reparse-point-aware provider on a native runner;
   test locking, sharing, case/UNC/long paths and Recycle Bin.
7. Implement Android SAF bridge on a real SDK/emulator/device; test persisted
   grants, revocation, scoped storage, back navigation and mobile UI.

## Later phases — not started

- Phase 2: persistent bookmarks/sessions, tree, drag/drop, properties, trash UX,
  real virtualization and incremental browsing beyond the current safety cap.
- Phase 3: operation journal/recovery, plan-based bulk rename/compare/sync,
  checksums/duplicates; independent measurements and destructive-action previews.
- Phase 4: sandbox-conscious previews and archives with quotas/traversal tests.
- Phase 5: editor/external-change safeguards, authorized APK inspection workflows,
  permission and native storage integrations.
- Phase 6: remote providers + secure credential storage; restricted plugins/AI.
- Phase 7: native platform QA, profiling/security audit, verified packaging/signing,
  clean install/uninstall, release acceptance. No release publishing authorized.

For every slice: inspect baseline → plan/contracts → code → regression/security
cases → format/lint/test → actual platform runtime → inspect diff → update docs.
A successful compile is not release acceptance. Unavailable tooling remains a
blocker, never a simulated platform implementation.
