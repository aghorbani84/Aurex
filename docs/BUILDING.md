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
The Android workflow separately installs SHA256-verified official dx0.7.10. `cargo fmt` only formats Rust syntax;
it does not compile RSX or prove event handlers type-check.

Launching without ROOT now opens an explicit native folder chooser, read-only by
default. The checkbox grants writes only to the selected root. Cancelling grants
nothing. Native graphical CI uses Xvfb/xauth/dbus-x11 and software rendering:

```sh
LIBGL_ALWAYS_SOFTWARE=1 WEBKIT_DISABLE_DMABUF_RENDERER=1 \
  dbus-run-session -- xvfb-run -a python3 scripts/gui_smoke.py
```

This is an acceptance attempt, not a production-support guarantee.

## Android engineering APKs — native builds verified, runtime gate open

The implemented Android entrypoint uses Kotlin SAF, never the Linux path provider.
It supports read-only tree selection, persisted read permission, listing/navigation,
filtering, and bounded UTF-8 preview. No broad-storage permission or Android mutation
controls. Both ARM64 and x86-64 builds/APK verification passed in
[run 41349931562](https://github.com/aghorbani84/Aurex/actions/runs/41349931562).
The x86-64 emulator acceptance step failed; runtime support is not established.

Use approved user-local tooling: official Rust1.88, JDK17, Android SDK platform and
build-tools35.0.0, command-line tools, and NDK r28b `28.1.13356709`. The build scripts
expect `ANDROID_HOME` and `ANDROID_NDK_HOME` to name those real installations.

```sh
rustup target add --toolchain 1.88.0 aarch64-linux-android x86_64-linux-android
python3 scripts/install_dx.py --output "$HOME/.local/share/aurex/dx-0.7.10"
export PATH="$HOME/.local/share/aurex/dx-0.7.10:$PATH"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/28.1.13356709"
python3 scripts/build_android.py --target aarch64-linux-android
python3 scripts/build_android.py --target x86_64-linux-android
```

Outputs are under `artifacts/android/<target>/`: real debug APK, SHA256 and verification
metadata. Release-mode Rust compilation does **not** mean production Android signing:
these APKs use Android's debug certificate. Only install trusted engineering builds
on a disposable emulator/test device, not a daily-driver containing important data.
The automated harness refuses physical devices and uses an owned fixture directory;
its CI setup option is restricted to disposable GitHub runners. Do not run that
privileged setup option on personal hosts. No signing keys/passwords are requested.
See [current evidence and remaining gates](reports/PRODUCTION_READINESS_2026-11-02.md).

## Windows

CI executed native Linux/Windows **compilation** jobs successfully. Windows filesystem provider
construction currently returns Unsupported; a compiled shell would not be a usable
Windows file manager. An unsigned Linux CLI engineering archive is implemented and
runtime-validated (see RELEASES.md). No production RPM/DEB/AppImage/MSI/APK/AAB
installer/signing pipeline is provided. Android debug CI artifacts are engineering
outputs, not a published release. Release builds require official toolchain
provenance, native runtime acceptance, dependency/license audits and approved secure
signing. Never commit signing material.

## Restricted sandbox note

Official static.rust-lang.org, index.crates.io and static.crates.io were unreachable.
Development-only Rust components came from user-local npm @rustbin mirrors. Official
index records were read via GitHub to generate the GUI lockfile; no registry override
or proxy is required/committed. This cannot substitute for native hosted compilation, actual GUI runtime acceptance
or release supply-chain verification; hosted build evidence is recorded separately.
