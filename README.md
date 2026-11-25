# AUREX

> A safety-first file manager in Rust — a tested Linux CLI and filesystem core today, with a Dioxus desktop UI in progress.

[![CI](https://github.com/aghorbani84/Aurex/actions/workflows/ci.yml/badge.svg)](https://github.com/aghorbani84/Aurex/actions/workflows/ci.yml)
![Rust](https://img.shields.io/badge/Rust-1.88-DEA584?logo=rust&logoColor=black)
![Edition](https://img.shields.io/badge/edition-2024-informational)
![Status](https://img.shields.io/badge/status-early%20development-orange)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

## Overview

AUREX is a file manager under active development, with Linux, Windows and Android as long-term targets. Its design priority is that file operations are **scoped, verified and never destructive by surprise**: everything happens inside a root directory you grant, links are never followed, writes are opt-in, and existing files are never silently overwritten.

> [!NOTE]
> **Status:** this is an early engineering foundation, not a release. The **Linux x86-64 CLI and filesystem core are built and tested**. The Dioxus desktop UI compiles on Linux and Windows but has **not** been accepted as a working GUI. Windows/Android filesystem providers, installers, archives, remote connections, plugins and recursive operations are not implemented. See [AUREX_STATUS.md](AUREX_STATUS.md).

## Features

**Filesystem core + CLI (Linux, tested)**

- **Root-scoped access** — all paths are relative to a granted root; no traversal through symlinks
- **Read-only by default** — any change requires `--write`; deletion also requires `--confirm`
- **Browse** — list directories (including hidden entries) with name filtering and paging
- **Preview** — bounded UTF-8 text preview (capped at 256 KiB)
- **Verified copy** — regular files are streamed, verified byte-for-byte, then published without replacing existing files
- **Move / rename** — same-filesystem only, never overwriting
- **Create folders** and **permanently delete** files or empty folders
- **Conflict policies** (fail / skip / keep both), cooperative cancellation and shared progress
- **Bounded executor** — blocking I/O runs on a small worker pool with a queue limit

**Desktop UI (source implemented, not yet accepted)** — dual panes, per-pane tabs and history, selection, bookmarks, operation dialogs with confirmation, activity and cancel, text preview, and light/dark themes, all calling the same Rust providers.

## Tech Stack

| Area | Technology |
| --- | --- |
| Language | Rust 1.88 (edition 2024), pinned via `rust-toolchain.toml` |
| Core crates | `aurex-core`, `aurex-fs`, `aurex-tasks` — no third-party dependencies |
| CLI | `aurex-cli` |
| Desktop UI | Dioxus 0.7.10 (separate workspace and lockfile) |
| Tooling | GitHub Actions CI, security workflow, `cargo-deny`, Python helper scripts |

## Getting Started

### Prerequisites

- Rust **1.88.0** (installed automatically by `rustup` from `rust-toolchain.toml`)
- Linux x86-64 for the tested CLI (browsing needs procfs; copying needs `O_TMPFILE` and hard-link support on the destination filesystem)

### Build and test the core

```sh
git clone https://github.com/aghorbani84/Aurex.git
cd Aurex
cargo test --workspace --locked
cargo build --release --locked
```

### Try the CLI on a disposable directory

```sh
lab="$(mktemp -d -t aurex-lab.XXXXXXXX)" || exit 1
printf 'Hello, AUREX\n' > "$lab/example.txt"

cargo run --locked -p aurex-cli -- "$lab" list
cargo run --locked -p aurex-cli -- "$lab" cat example.txt
cargo run --locked -p aurex-cli -- "$lab" --write mkdir copies
cargo run --locked -p aurex-cli -- "$lab" --write copy example.txt copies/example.txt
```

```text
Usage: aurex-cli ROOT [--write] COMMAND [ARGS]

  list [PATH] [FILTER]       List directory, including hidden entries
  cat PATH                   Bounded UTF-8 text preview
  mkdir PATH                 Create private directory
  copy SOURCE DEST           Verified regular-file copy; never overwrite
  move SOURCE DEST           Same-filesystem move; never overwrite
  rename SOURCE DEST         Rename; never overwrite
  delete PATH --confirm      PERMANENTLY delete a file or empty directory
```

> [!WARNING]
> Deletion is permanent. Always test against a throwaway directory, never against system folders.

### Desktop UI (experimental)

Requires the native WebView dependencies listed in [docs/BUILDING.md](docs/BUILDING.md):

```sh
lab="$(mktemp -d -t aurex-gui-lab.XXXXXXXX)" || exit 1
cargo run --manifest-path apps/aurex-app/Cargo.toml --locked -- "$lab"
```

### Full local check

```sh
scripts/check.sh   # fmt, clippy (-D warnings), tests, release build, Python tests
```

## Project Structure

```text
Aurex/
├── crates/
│   ├── aurex-core/    # Shared types and navigation/tab model
│   ├── aurex-fs/      # Filesystem provider (Linux implementation)
│   └── aurex-tasks/   # Bounded blocking-I/O executor
├── apps/
│   ├── aurex-cli/     # Command-line interface + integration tests
│   └── aurex-app/     # Dioxus desktop UI (separate workspace)
├── scripts/           # check, benchmark, packaging and audit helpers
├── docs/              # Architecture, building, testing, platform matrix, reports
└── .github/workflows/ # CI, platform builds, security, release
```

## Documentation

[Status](AUREX_STATUS.md) · [Features](FEATURES.md) · [Architecture](ARCHITECTURE.md) · [Security](SECURITY.md) · [File operation guarantees](docs/FILE_OPERATIONS.md) · [Building](docs/BUILDING.md) · [Platform support](docs/PLATFORM_SUPPORT.md) · [Implementation plan](docs/IMPLEMENTATION_PLAN.md) · [Changelog](CHANGELOG.md)

## License

Released under the [MIT License](LICENSE). Reference applications were studied for workflows only; their source, assets and branding are not incorporated.

## Author

Built by [@aghorbani84](https://github.com/aghorbani84).
