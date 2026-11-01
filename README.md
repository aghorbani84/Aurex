# AUREX

A Rust file manager under active development, targeting Linux, Windows and Android.
**This is an early engineering foundation, not a production or cross-platform release.**

## What can I run today?

The **Linux x86-64 CLI and filesystem core are built and tested**. They provide real
root-scoped browsing, filename filtering, metadata, verified regular-file copying,
same-filesystem moves, rename, folder creation, permanent deletion of files/empty
folders, and bounded UTF-8 previews. Read-only is the default.

A **Dioxus 0.7.10 desktop implementation** connects these same providers to dual panes,
tabs, selection, operation confirmations, cancellation, activity, and text previews.
Native **Linux and Windows compilation passed on GitHub**, after hosted compiler
errors were fixed. GUI runtime/display/shutdown acceptance is still **NOT TESTED**;
strict GUI dependency/license policy still fails. Local crate/native downloads are
restricted. Compilation is not evidence of a supported graphical application.

Windows filesystem support, production GUI installers, archives, remote connections,
plugins, APK tools, persistence and recursive operations are **not implemented**.

The Linux/Android follow-up adds a Linux startup folder chooser and a real Android
SAF read-only browser/preview implementation. It is **under native validation**, not
production-ready. See the [current implementation and release gates](docs/reports/PRODUCTION_READINESS_2026-11-02.md).

## Quick start: tested Linux core

Install stable Rust **1.88.0** using your normal user-local Rust installation, then:

```sh
cargo test --workspace --locked
cargo build --release --locked

# Grant only a disposable directory initially. All entry paths are root-relative.
lab="$(mktemp -d -t aurex-lab.XXXXXXXX)" || exit 1
printf 'Hello, AUREX\n' > "$lab/example.txt"
cargo run --locked -p aurex-cli -- "$lab" list
cargo run --locked -p aurex-cli -- "$lab" cat example.txt
cargo run --locked -p aurex-cli -- "$lab" --write mkdir copies
cargo run --locked -p aurex-cli -- "$lab" --write copy example.txt copies/example.txt
```

`cargo run -p aurex-cli -- --help` lists commands. Deletion additionally requires
`--confirm` and is permanent. Existing destinations are never silently overwritten.
Linux browsing requires procfs; copying additionally requires `O_TMPFILE` and
hardlink publication support on the destination filesystem. Unsupported filesystems
fail without destroying the source. Never start testing against system directories.

## Development

```sh
scripts/check.sh
python3 scripts/benchmark.py   # after the release build; private temporary fixtures
# GUI runtime experiment only; known security gates remain open.
# Use a private empty root on a host with the documented native dependencies:
lab="$(mktemp -d -t aurex-gui-lab.XXXXXXXX)" || exit 1
cargo run --manifest-path apps/aurex-app/Cargo.toml --locked -- "$lab"
```

The small root workspace is dependency-free and builds offline once Rust is installed.
The renderer is an explicitly separate workspace with its own committed lockfile,
so GUI dependencies cannot block filesystem regression tests. There is no web
filesystem server, externally exposed API, telemetry, or automatic file execution.

## Documentation

- [Current status](AUREX_STATUS.md) · [Implementation plan](docs/IMPLEMENTATION_PLAN.md)
- [Architecture](ARCHITECTURE.md) · [Feature availability](FEATURES.md)
- [Security model](SECURITY.md) · [Operation guarantees](docs/FILE_OPERATIONS.md)
- [Build instructions](docs/BUILDING.md) · [Development](docs/DEVELOPMENT.md)
- [Platform matrix](docs/PLATFORM_SUPPORT.md) · [Tests](docs/TEST_MATRIX.md)
- [Research and toolchain verification](docs/RESEARCH.md)
- [CI/security policy](docs/CI_CD.md) · [Testing](docs/TESTING.md)
- [Engineering artifacts](docs/RELEASES.md) · [Upgrade report](docs/reports/AUREX_ENGINEERING_REPORT.md)

The engineering upgrade adds typed partial-commit errors, deterministic copy fault
regressions, workflow scanners/weekly dependency policy, and a **validated unsigned
Linux CLI archive** with SHA-256, metadata, scoped SBOM and Rust notices. It is not a
GUI/Windows/Android production release. Build it with `python3 scripts/package.py`
after the release build from a clean checkout (see the artifact guide).

## License

MIT; see the existing [LICENSE](LICENSE). Reference applications are studied for
workflows only; their source, assets and branding are not incorporated.
