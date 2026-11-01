# Testing and validation

```sh
scripts/check.sh
cargo check --workspace --all-targets --locked
python3 -m unittest discover -s tests -v
# Workflow checks require the pinned development tools described in CI_CD.md:
python3 scripts/validate_workflows.py
actionlint
zizmor --offline --persona auditor .github/workflows
cargo audit --file Cargo.lock
cargo audit --file apps/aurex-app/Cargo.lock
cargo deny --manifest-path Cargo.toml check advisories licenses bans sources
cargo deny --manifest-path apps/aurex-app/Cargo.toml check advisories licenses bans sources
```

Core tests do NOT compile the separate Dioxus workspace. Use the commands in
BUILDING.md on a native WebView-capable host. No `--all-features` shortcut or feature
suppression pretends Android/Windows filesystem runtime has been validated.

The Rust suite now includes six deterministic production-copy checkpoints: midstream
cancellation, last-precommit cancellation, stable-source violation, injected ENOSPC/
file-sync failures, postcommit directory-sync failure and postcommit cancellation.
These are explicit fault injections, not claims of exhausting a real disk. A copy
that was published but cannot be confirmed durable returns `ErrorKind::Durability`
and `Error.committed`, retains its original OS error and warns against blind retry.
No rollback or retry deletes the published destination or the source.

Python tests exercise actual CLI package launch, transfers/conflict refusal, byte-
identical archive generation, checksums, traversal/symlinks/duplicates, PAX/decompression
limits, and Android fail-closed reporting. All fixtures are private temporary roots.
The verifier never extracts arbitrary names or executes untrusted archive input.

Windows tests currently cover portable core/executor/navigation and CLI help, not a
working Windows provider. Linux provider tests are architecture-gated; Android is
blocked and produces a readiness report, not a passing Rust-only "APK build".
See TEST_MATRIX.md and reports/ for exact local/remote evidence and gaps.
