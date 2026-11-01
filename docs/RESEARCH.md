# Research and verification log

Date: 2026-11-01. Official source/API references were read, not executed as a primary
filesystem engine. No proprietary UI/assets or third-party application code copied.

## Dioxus

- Official latest stable release API:
  https://api.github.com/repos/DioxusLabs/dioxus/releases/latest → **v0.7.10**.
- Tagged source: https://github.com/DioxusLabs/dioxus/tree/v0.7.10
  (`57d6794ad60b949e5bd8aa282f6f8c3dc97a365e`).
- Read workspace and `packages/dioxus`, `packages/desktop`, `packages/liveview`
  manifests; launch/context builder and signal/resource examples; upstream native CI.
- Dioxus declared MSRV 1.83.0, MIT OR Apache-2.0. Pin app to =0.7.10 with the actual
  macro/html/signal/hooks/desktop/launch features. Tokio =1.48.0 (MIT), matching the
  tagged workspace's runtime line. Chosen stable compiler: 1.88.0.
- GUI lock generated from official crates.io index records via GitHub API; resolver 3
  avoids known Rust-1.89-only uuid selection. All registry checksums are retained.
- Crate tarballs/native libraries are unavailable here; the GUI has not been compiled,
  and transitive dependencies still need an actual advisory/license audit.
- A native renderer was chosen instead of introducing a browser-exposed filesystem
  server just to produce a screenshot. Official docs URL remains the tagged package's
  https://dioxuslabs.com/learn/0.7/getting_started (not reachable in this environment).

## Yazi

https://github.com/sxyazi/yazi at
`edb4965e976fe6b095180bfaabbf38caa1f9aba3`.
Read scheduler, worker and progress sources. Notable separation: typed task input,
shared task handles/progress, per-category worker scheduling, ongoing-task registry,
cancellation scopes and result dispatch. Studied, not copied. AUREX's initial
executor uses a bounded queue and OS worker threads with awaitable results, rather
than depending on the Yazi executable or replicating its architecture wholesale.
Priority pools/recovery/task persistence are not implemented. Official user docs:
https://yazi-rs.github.io/ (not fetched under this egress policy).

## Workflow references

Official GitHub readmes inspected:

- https://github.com/doublecmd/doublecmd — dual panels, independent workflows.
- https://github.com/KDE/dolphin — discoverable, non-intrusive power features;
  avoid burying ordinary navigation beneath a toolbox.
- https://github.com/zhanghai/MaterialFiles — mobile breadcrumb and provider-aware
  architecture; native filenames and OS metadata instead of parsing `ls` output.

These are workflow references only; no GPL implementation/assets were incorporated
into this MIT project. Windows Explorer, GNOME Files, Total Commander and MiXplorer
remain workflow targets from the brief, not newly validated integrations.

## MT Manager: blocked, not claimed complete

https://mt2.cn/ and https://mt.cc/guide/ both failed TLS access under sandbox egress
policy. The brief informs dual-pane/selection/path/transfer requirements, but is not
substituted for studying their official documentation. Complete this research when
network access is available, especially archive/APK/editor constraints before those
phases. No reverse engineering, license bypass or unauthorized APK tooling added.

## Toolchain and dependencies

No Rust/Cargo/dx were initially installed. Official static.rust-lang.org,
index.crates.io and static.crates.io were unreachable. User-local npm @rustbin
rustc/cargo/std/rustfmt/Clippy 1.88.0 components enabled core tests. These are an
unofficial development mirror, not release provenance verification. No system-wide
packages installed. Root core intentionally has no registry dependencies; manual
structured errors and a small executor are real, tested bootstrap code. Serde,
thiserror, tracing, SQLite, archive decoders and platform APIs need dedicated later
slices and verified compatibility/licenses, not unused dependency marketing.
