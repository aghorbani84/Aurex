# Building

## Core and CLI — verified on Linux x86-64

Use official user-local stable Rust 1.88.0 and its rustfmt/Clippy components. The
repository toolchain file pins this version; no nightly features or wildcard
application dependencies. No external crates are needed for the root workspace.

```sh
cargo build --workspace --release --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Runtime: Linux x86-64, procfs mounted and accessible, filesystem permissions as the
launching user. Copy destinations need O_TMPFILE and hardlink publication. Kernel
6.1.158 was exercised here; compatibility with older kernels/distributions is not
asserted. renameat2 requires libc/kernel support; no unsafe fallback is provided.

## Dioxus desktop — compilation verified, runtime gate open

Dioxus stable **0.7.10**, Tokio **1.51.6**. The app is a separate workspace importing
the root's real crates. Resolver 3 selected Rust 1.88-compatible transitive versions.
Native Linux and Windows release compilation passed on GitHub at source 9bf7405.
Local builds remain blocked by egress/native tooling. Hosted strict desktop deny
still fails on documented transitive advisories/license policy; **no GUI runtime
or release acceptance** is claimed. See reports/AUREX_ENGINEERING_REPORT.md.
On a normally provisioned Ubuntu 24.04 development host, install the packages used by
Dioxus's tagged upstream CI using your approved system package process:

```text
pkg-config libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev
libxdo-dev libglib2.0-dev libssl-dev libasound2-dev libudev-dev nasm
```

No package installation was performed system-wide in the sandbox. The disposable
hosted Ubuntu native-build job installed its documented build prerequisites.

```sh
cargo check --manifest-path apps/aurex-app/Cargo.toml --locked
cargo clippy --manifest-path apps/aurex-app/Cargo.toml --all-targets --locked
lab="$(mktemp -d -t aurex-gui-lab.XXXXXXXX)" || exit 1
cargo run --manifest-path apps/aurex-app/Cargo.toml --locked -- "$lab"
# Explicit mutation grant, only after testing read-only operation:
cargo run --manifest-path apps/aurex-app/Cargo.toml --locked -- "$lab" --write
```

A graphical display/WebView runtime is needed; native desktop rendering is not a
sandbox browser preview. `dx` is not required for this plain Cargo desktop launch.
Dioxus CLI is not installed/verified here. `cargo fmt` only formats Rust syntax;
it does not compile RSX or prove event handlers type-check.

## Windows and Android

CI executed native Linux/Windows **compilation** jobs successfully. Windows filesystem provider
construction currently returns Unsupported; a compiled shell would not be a usable
Windows file manager. Android has neither an SAF provider nor a verified mobile
entrypoint/build configuration. No Android build/packaging job is fabricated.

An unsigned Linux CLI engineering archive is implemented and runtime-validated
(see RELEASES.md). No RPM/DEB/AppImage/MSI/APK/AAB installer/signing workflow is implemented. Release
builds must use official toolchain provenance, real native tests, dependency/license
audits and secure signing before release packaging. Never commit signing material.

## Restricted sandbox note

Official static.rust-lang.org, index.crates.io and static.crates.io were unreachable.
Development-only Rust components came from user-local npm @rustbin mirrors. Official
index records were read via GitHub to generate the GUI lockfile; no registry override
or proxy is required/committed. This cannot substitute for native hosted compilation, actual GUI runtime acceptance
or release supply-chain verification; hosted build evidence is recorded separately.
