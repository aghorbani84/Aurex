# CI/CD and dependency policy

Default branch `main` was discovered through GitHub, not guessed. Build/CI/security
workflows run on PRs, pushes to that branch and manual dispatch. Security is also
scheduled weekly. No publishing permissions, secrets, pull_request_target, broad
error suppression or repository-settings changes are used.

| Workflow | Responsibility |
|---|---|
| ci.yml | Native Linux/Windows core check/fmt/Clippy/tests/release compilation; Python fixture tests |
| build-linux.yml | Validated unsigned CLI candidate archive; independent actual Dioxus release build |
| build-windows.yml | Native MSVC CLI/help and Dioxus release build; no unusable file-manager package |
| build-android.yml | Manual **failing readiness gate**, diagnostic JSON only; not APK packaging |
| security.yml | actionlint 1.7.12, zizmor 1.31.1 auditor, Gitleaks 8.30.1, cargo-audit 0.22.2, cargo-deny 0.20.2 |
| release.yml | Manual CLI engineering artifact build only; cannot publish a Release/tag/deployment |

Android is not implemented, so an honest gate is preferable to speculative SDK/NDK
YAML that pretends to produce a usable APK. The gate is deliberately not a required
PR check yet. Replace it with an actual SDK/NDK/JNI/SAF/package/signature/runtime
workflow only when the mobile slice exists. No Android application identifier,
keystore, signing password, AAB or VLC/mpv dependency is invented in this repository.

Third-party actions use full verified upstream SHAs; checkout sets
`persist-credentials: false`. Rust uses the hosted runner's rustup to install pinned
1.88.0; Go tooling uses 1.25.5. Python tooling is 3.11 with pinned scanner packages;
its exact patch is logged by setup-python. Native packages come from runner Ubuntu
repositories; this is not a byte-reproducible OS image guarantee. No unverified DLLs
or third-party runtime downloads are bundled. Build metadata records actual compilers.

Cache keys separate OS, architecture, Rust version, lockfile and core/desktop graphs.
PRs only restore caches. Cache saving is limited to trusted default-branch pushes;
no PR-generated target output can be promoted as a release. Dependencies can still
rebuild without caches. Go setup cache is disabled because no Go workspace is part
of AUREX. Timeouts and concurrency cancellation bound workflow resource usage.

Dependabot treats `/`, `/apps/aurex-app` and GitHub Actions independently.
`deny.toml` has **no advisory exceptions**. License allow-listing is policy, not a
conclusion that native libraries, codec patents or distribution obligations have
been fully reviewed. Duplicate versions warn; unknown sources/wildcards/yanked
packages fail. New advisories must fail visibly and be fixed or receive narrowly
reviewed time-limited justification, not blanket ignores.

Tools actually run locally: actionlint v1.7.12 built from official tagged source,
zizmor 1.31.1, YAML and project policy checks. Restricted egress prevented the official
actionlint binary download; its source build uses official pinned GitHub module
checkouts and local go.mod replacements (`v1.7.12+dirty`, not a modified product).
Go 1.25.5 was installed user-locally from PyPI go-bin. This does not substitute for
release toolchain provenance. cargo-audit/deny require registry access unavailable
here; their remote jobs and advisory findings must be reported independently.

CodeQL Rust support and `act` were not enabled/claimed: neither substitutes for the
native runtime matrix or tested scanners above. The local policy checker rejects
unpinned actions, shell context interpolation, credential persistence, broad job
permissions and hidden failures. It supplements, not replaces, actionlint/zizmor.
Artifacts are bounded, checksummed, unsigned CLI-only candidates (RELEASES.md).
See reports/AUREX_ENGINEERING_REPORT.md for actual run links/status, not YAML promises.

### First real GitHub run corrections

GitHub's first execution passed native Linux/Windows core checks and CLI artifact
runtime tests, but reported Node 20 deprecation for the old action major versions
and a dependency-audit failure. Action pins were re-verified against current official
Node 24 releases (checkout 7.0.1, setup-python 7.1.0, setup-go 7.0.0, cache 6.1.0,
upload-artifact 7.0.2). cargo-audit 0.22.2's tagged MSRV is verified as 1.88.
The audit runner now executes **all four graphs**, records real exit codes, fails if
any check fails and emits escaped, bounded error annotations plus a diagnostic
artifact. No advisory exception or vulnerability suppression was added. This also
exposes actionable errors when egress blocks the GitHub log storage hostname.
