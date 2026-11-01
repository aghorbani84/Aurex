# Test matrix

Local run: 2026-11-01, Linux x86-64, Rust 1.88.0, kernel 6.1.158 sandbox.

| Suite / gate | Result | Evidence |
|---|---|---|
| CLI integration | PASS (4) | Real subprocess lifecycle, grants/confirm, conflict/traversal, literal crafted names |
| Core navigation | PASS (4) | Branching history, independent tabs, reset selection, bounds |
| Copy fault injection | PASS (6) | Midstream/precommit/postcommit cancellation, source mutation, injected ENOSPC/file-sync and directory-sync partial commit |
| Python engineering | PASS (10) | Actual package launch/reproducibility, malicious package fixtures and Android fail-closed report |
| Task executor | PASS (4) | Real Future wake, panic isolation, deterministic queue backpressure, shared progress/cancel |
| Filesystem provider | PASS (19) | See test names in `crates/aurex-fs/tests/provider.rs` |
| Root formatting | PASS | `cargo fmt --all --check` |
| UI Rust formatting only | PASS | `cargo fmt --manifest-path apps/aurex-app/Cargo.toml --all --check`; NOT RSX type checking |
| Root Clippy | PASS | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| Root release build | PASS | `cargo build --workspace --release --locked` |
| CLI smoke benchmark | Executed | `docs/PERFORMANCE.md`; not a release performance threshold |
| Dioxus dependency resolution | PASS | Cargo-generated Rust-compatible GUI Cargo.lock |
| Dioxus compilation/runtime/UI tests | BLOCKED | Unavailable crate downloads and native WebView toolchain/display |
| Native Windows tests | NOT RUN | CI definition exists, not run in this session |
| Android SAF/runtime/build | NOT RUN | No provider, SDK/NDK/emulator or package workflow |
| Workflow scans | PASS locally | actionlint 1.7.12, zizmor 1.31.1 auditor, YAML/project policy |
| Dependency advisory/license audit | PENDING remote scan | No blanket advisory exceptions; transitive license review pending |

## Filesystem coverage

Real create/copy/move/rename/delete cycle; byte progress; fail/skip/keep-both conflicts;
concurrent no-replace copy winners; Unicode and non-UTF8 identities; hidden/filter/
metadata; paging and pre-pagination descending sort; empty and multi-buffer files;
read-only grants; OS permission refusal (not asserted on privileged UID 0 runners);
parent/absolute/NUL traversal; symlink/broken-link escapes; bounded binary/oversize
previews; FIFO nonblocking rejection; nonexistent source cleanup; nonrecursive-delete/
directory-copy refusal; private output permissions; root FD identity after rename.
All mutable fixtures are created via exclusive private temporary directories and
removed by fixture owners. No tests touch existing user directories or mounts.

## Gaps / required fault injection

- Real short writes/read failures/disk-full and kernel fsync failure integration
  beyond the now-implemented deterministic injected failure checkpoints.
- O_TMPFILE/procfs unavailable filesystems; EXDEV on controlled independent mounts.
- Concurrent adversarial parent replacement, mount/hardlink boundary policies.
- 50k-entry refusal/large-directory memory ceiling and cache benchmarks.
- Worker resource exhaustion, graceful scheduler shutdown, active-job GUI close.
- UI accessibility/focus/keyboard/touch and actual responsive WebView runtime.
- Windows UNC/ACL/locking/long/case paths; Android grant/revocation/provider cases.
- Archive/plugin/credential tests belong to future implemented modules, not fake tests.

The 37-Rust + 10-Python test total excludes doc-test suites with zero cases. CI files are test
**definitions**, not a statement that GitHub checks already passed.
