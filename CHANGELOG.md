# Changelog

## Unreleased — 2026-11-01

- Establish real Rust core, task executor, Linux descriptor-relative provider and CLI.
- Add verified no-overwrite file copies, same-filesystem moves/renames, directory
  creation, nonrecursive permanent deletion and bounded text preview.
- Add paging/filtering/metadata and bounded navigation/tab state.
- Add Dioxus 0.7.10 desktop source wired to providers; **not compiled/runtime-verified**.
- Add 31 passing local tests, repeatable smoke measurements, documentation and CI gates.
- Retain existing MIT license; no installers or release artifacts published.

### Engineering hardening

- Add typed committed-destination/durability errors and six deterministic copy fault tests.
- Validate/package unsigned Linux CLI candidates with checksums, metadata, scoped SBOM,
  Rust notices, runtime smoke and reproducibility/security regression tests.
- Add validated CI/native build/security/update workflows; fail-closed Android readiness.
- Preserve the independent FAZORA boundary and document actual blockers rather than
  inventing platform packages, multimedia support or signed releases.
