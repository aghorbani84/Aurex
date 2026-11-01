# Engineering artifacts, not published releases

The implemented package is an **unsigned Linux x86-64 CLI engineering candidate**,
not the Dioxus GUI, a production release, an Android APK or a Windows file manager.

```sh
cargo build --workspace --release --locked
python3 scripts/package.py                    # clean committed checkout
# Local-only verification of the current dirty working tree, explicitly recorded:
python3 scripts/package.py --allow-dirty
python3 scripts/package.py --verify artifacts/packages/aurex-cli-0.1.0-x86_64-unknown-linux-gnu.tar.gz
```

Packaging runs the actual binary before and after archiving: help, listing, mkdir,
copy, move, conflict refusal and permanent deletion using exclusively owned temporary
fixtures. It validates runtime linking via ldd. Archives have stable source-commit
time, ownership and order; include SHA-256 manifests, version/commit/dirty/compiler
metadata, source LICENSE/security/operation docs, actual installed Rust toolchain
notices and a CycloneDX root-lockfile inventory. Native runtime dependencies are
listed separately. The inventory is **not** a complete binary composition/license
or vulnerability audit. Verify checksum provenance before running any executable;
checksums do not authenticate an untrusted download. The verifier does not execute
by default and never uses extractall. Its execute option is for trusted local builds.

The GNU archive needs a compatible glibc/libgcc host; no universal Linux compatibility
or installation/file-association integration is promised. Extract to a user-owned
folder; run `bin/aurex-cli --help`; removing that folder uninstalls it. No administrator
privileges or system-wide writes. No AppImage/DEB/RPM/MSI or signed packages are claimed.

CI artifacts are downloadable test outputs, **distinct from GitHub Releases**. The
candidate workflow is manually requested and build-only, with read-only repository
permissions. It does not publish, deploy, create tags, access signing secrets or
change protected environments/settings. Official publication requires separate
explicit authorization and native GUI/platform runtime, dependency/license/security
and signing acceptance. Windows GUI builds are compile/link evidence only; Android
readiness is a deliberate failing gate until a real mobile app + SAF exists.
