# REPORT A — AUREX engineering upgrade

Date: 2026-11-01. This is a **partial engineering upgrade**, not the commercial/cross-
platform acceptance milestone. FAZORA is a separate execution (FAZORA_HANDOFF.md).

## Baseline and work

BASELINE_2026-11-01.md records the inspected Git root, existing uncommitted foundation,
31 passing tests, toolchain/workspace, missing packages/platforms and permission limits.
Existing changes and MIT license were preserved; no reset/history rewrite/force push.

Security review: SECURITY_AUDIT.md. Added typed partial-commit errors and six production-
path copy fault regressions; fixed checkout credential persistence; hardened new package
verification against traversal, symlink/duplicate/checksum/PAX/decompression hazards.
No unmeasured runtime optimization or universal safety/security certification is claimed.

## Verification evidence (local)

Raw logs: ignored `artifacts/evidence/` (not secrets or generated data in Git).

| Check | Result | Actual command/evidence |
|---|---|---|
| Formatting | PASS | `cargo fmt --all --check` and separate GUI Rust formatting |
| Clippy | PASS | `cargo clippy --workspace --all-targets --locked -- -D warnings`, exit 0 |
| Rust tests | PASS | `cargo test --workspace --locked`: 37 (baseline 31 + 6 fault regressions) |
| Engineering tests | PASS | `python3 -m unittest discover -s tests -v`: 10, `Ran 10 tests ... OK` |
| Workflow structure/type lint | PASS | actionlint v1.7.12 source build, exit 0; shellcheck/pyflakes not installed locally |
| Workflow security | PASS, scoped | zizmor 1.31.1 `--offline --persona auditor`, exit 0, `No findings to report` |
| YAML/project policy | PASS | `validate_workflows.py`: six workflows, pins/permissions/credentials/timeouts/contexts checked |
| Dependency advisory/license audit | BLOCKED locally | cargo-audit/deny unavailable under registry egress; remote jobs defined without advisory exceptions |
| Linux CLI build/runtime | PASS | `cargo build --workspace --release --locked`; packaged binary fixture lifecycle before/after archiving |
| Linux GUI build/runtime | BLOCKED locally | GUI offline cargo check exits 101: missing `tokio`; native WebView unavailable |
| Windows native build/runtime | PENDING GitHub | Actual MSVC CLI and GUI build jobs; provider explicitly unimplemented; no installer falsely shipped |
| Android APK/AAB | BLOCKED | readiness exits 2 with JSON; no mobile entrypoint/config/provider/SDK/NDK/JDK; no APK/AAB |
| Reproducibility | PASS within stated scope | Two archives from the same binary/source metadata compare byte-for-byte equal |
| GitHub execution | PENDING | PR/run URLs and actual remote status to be recorded after push; local YAML is not remote execution |

## Artifacts

`artifacts/packages/aurex-cli-0.1.0-x86_64-unknown-linux-gnu.tar.gz`, adjacent `.sha256`,
`build-metadata.json`, `sbom.cdx.json`. Unsigned **CLI engineering candidate** only.
Per-member SHA-256 manifest, source/compiler/dirty metadata, exact installed Rust
notices and OS native dependency list included. Local candidates honestly record a
dirty working tree until committed. No GUI runtime, installer, code signing, APK/AAB
or official release is claimed. CI candidates are separate from GitHub Releases.
See RELEASES.md for validation/provenance/limitations. Generated files are ignored.

## Automation and docs

Independent CI, Linux, Windows, Android readiness, weekly security and manual
candidate workflows; weekly Cargo/Actions Dependabot; cargo-deny policy without
advisory ignores. Read-only permissions, immutable action pins, no checkout token
persistence, no untrusted shell context interpolation, trusted-branch-only cache
writes, bounded durations, no signing/publishing secrets or permissions.

README, status, test matrix, operation/security guides updated; new testing, CI/CD,
artifact, baseline/audit and independent execution reports. Default branch discovered
as main. No FAZORA workspaces/runtime/identity/engines are merged into this project.

## Remaining acceptance blockers

- Compile and actually run/test the Dioxus GUI, including keyboard/focus/shutdown.
- Implement and test Windows native provider and real Android SAF/mobile packaging.
- Run dependency/security/license jobs on a normally connected native runner and fix
  actual findings; no broad ignore to force green checks.
- Actual filesystem disk-full/short-write/kernel fsync/hostile races beyond injected
  tests; permissions/grant revocation and OS-specific runtime acceptance.
- Complete dependency/native redistribution review; immutable/reproducible OS images,
  signed packages, installation tests and approved protected publication pipeline.
- Aarch64 Linux is not enabled; no flag-constant/native validation pretends otherwise.
- FAZORA native playback/FFI/license/build work belongs in its separate checkout.
