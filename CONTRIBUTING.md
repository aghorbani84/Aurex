# Contributing

Read SECURITY.md, ARCHITECTURE.md and AUREX_STATUS.md before changes.

1. Work on a focused vertical slice with an explicit acceptance test.
2. Keep filesystem identity native; do not turn document URIs into local paths.
3. Run `scripts/check.sh`. On a GUI-capable host also compile/lint/run the separate
   Dioxus workspace; a root-workspace test pass does not cover UI compilation.
4. Use private temporary test directories. Never run destructive tests against
   home, system directories, devices, real mounts or existing user data.
5. Add regression tests for failures, conflicts, cancellation and scope boundaries.
6. Document every unsafe block's platform and ownership assumptions. Unsupported
   capabilities must remain errors, not guessed cross-platform substitutions.
7. Update status, feature/support/test matrices and relevant guarantees together.

Do not commit secrets, keystores, generated datasets, target directories or vendored
third-party code without a clear license/provenance review. Do not copy reference
application branding/assets/code. No release or force push is implied by contributing.
