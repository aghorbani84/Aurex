# Installation

There are no published AUREX packages or supported installers from this implementation.
Do not download purported release artifacts on the strength of this documentation.

For development, follow BUILDING.md, build the CLI locally, and invoke
`target/release/aurex-cli --help`. No system-wide installation, shell integration,
privileges, services or network ports are needed. Use a private disposable directory
as the initial root; write access is opt-in. Deletion is permanent.

The Dioxus app is unverified source, not an installable verified GUI release. Windows
and Android do not yet have functional native filesystem providers. Packaging and
clean install/uninstall verification belong to the production-quality milestone.

Troubleshooting:

- Missing Rust/Cargo: install the pinned stable user-local official toolchain.
- Missing GUI crate/native packages: use a normally connected, provisioned build host.
- `O_TMPFILE`/hardlink error: destination filesystem does not support this copy path;
  source remains untouched. Do not work around it by disabling no-follow/no-replace.
- `procfs required`: this native provider requires procfs and does not use shell `ls`.
- Permission error: choose an authorized root/account; do not silently elevate.
- Unsupported provider/platform/cross-volume move: capability is not implemented;
  see PLATFORM_SUPPORT.md and FILE_OPERATIONS.md rather than assuming a UI bug.
