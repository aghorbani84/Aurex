# Linux + Android implementation and production acceptance

Date: 2026-11-02. **Not a production certification or release.**

## Baseline before changes

Repository `/home/user/Aurex`, active branch `arena/2e0e28b8-aurex`, clean tree at
`23284822b374f8229b2d3dce073cd6960250de1f` (merged PR #1). Root/core + separate Dioxus
workspace and lockfiles inspected. Prior executed evidence: 37 Rust/10 Python tests,
native Linux/Windows compilation; no GUI runtime test. Android had no entrypoint,
SAF implementation, manifest/config or actual build workflow, only a failing gate.
Five transitive desktop advisories and libfuzzer-sys NCSA policy remained unresolved.
Original MIT LICENSE retained. No FAZORA source/identity is part of this work.

This fresh sandbox initially lacked Rust/Java/Gradle/ADB/dx. Raw baseline is ignored
`artifacts/evidence/production-baseline.json`. User-local Rust 1.88 was bootstrapped
from checksum-verified npm mirror components for development only, not trusted release
provenance. Native toolchains/device runtime cannot be installed system-wide here.

## Implemented

- Linux installed-app startup opens a real native folder chooser instead of exiting
  without command-line ROOT. It does not default to home or grant the whole disk.
  Read-only by default, explicit opt-in writes, cancellation grants nothing. CLI ROOT
  invocation remains supported; native filenames remain native paths.
- Explicit feature-gated native DOM test: actual WebView browsing/history, tabs,
  filtering, UTF-8 preview, disabled writes, fixture hashes and window-close exit.
  A virtual display is still a real renderer, not HTML mocked as native acceptance.
- Separate `desktop`/`mobile` Cargo features; actual Android main, Dioxus configuration,
  Kotlin Gradle plugin and Android UI. Android is deliberately **read-only**: folder
  picker, persisted read grant, provider listing, navigation, local filtering and
  bounded UTF-8 previews. No pretend copy/move/delete controls or broad storage grant.
- SAF ContentResolver calls off Rust's renderer thread, bounded worker queue and
  native query/open timeouts; explicit provider errors, revocation/forget handling,
  private preferences/no backups. Documents retain opaque content identities, never
  coerced into local paths. Foreign or undiscovered document IDs fail before opens.
- Core document-session model validates authority, discovered kinds, bounded history/
  identities and response generations. Same-URI regrant and asynchronous stale
  results cannot authorize old work. Native boundary independently checks OS grants.
- JNI uses the live Activity's class loader (not FindClass on an attached worker),
  scoped local frames and owned global references, clears pending Java exceptions,
  and does not rely on the upstream process-global cached Activity helper. Picker
  lifecycle unregisters on Activity destruction. No Rust callback pointer is retained.
- Actual Android build jobs for ARM64 and x86-64. SDK/build-tools 35, NDK r28b
  `28.1.13356709` verified against official android/ndk release; JDK17/Rust1.88;
  official dx0.7.10 asset SHA256 verified before execution. APK checks verify app
  identifier, explicitly **debug** signing, broad-permission absence, ELF ABI/16KiB
  alignment, backup/cleartext policy and checksums. No production signing material.

Nine required packages added for the official Manganis JNI/native-source metadata
bridge; no previously locked package version changed. Manganis0.7.10 is MIT OR
Apache2, verified against official tagged source. Its upstream const-serialize
pre-release internals remain in the exact lockfile and require real audit validation;
we do not misrepresent them as stable or security-certified. rfd/JNI/serde_json lines
are exact versions already present in the baseline lockfile.

## Verification

| Command/gate | Actual result |
|---|---|
| `scripts/check.sh` | PASS locally: 44 Rust + 18 Python tests, formatting, strict core Clippy, release CLI |
| New document model regressions | PASS: opaque identities, forged/file navigation, atomic foreign-page refusal, stale pages, same-URI grant replacement, malformed URIs and resource limits |
| Android artifact unit fixtures | PASS: real ELF header parsing, wrong ABI/4KiB/truncated refusal, debug identity/backup/permissions policy; mocked SDK transport is NOT a device test |
| `validate_workflows.py` | PASS: six independent workflows, full SHAs/read-only credentials/timeouts/safe shell env |
| zizmor1.31.1 auditor | PASS locally: no findings; does not audit application/native dependencies |
| Linux native build + graphical test | Native compile PASS again in [run 41349931498](https://github.com/aghorbani84/Aurex/actions/runs/41349931498); smoke-feature compile FAIL before launch. Found missing direct `dioxus-document` import/optional feature; correction and compiler-output capture added, rerun pending |
| Android native build/APKs | PASS for ARM64 and x86-64 native builds + complete APK verifier in [run 41349931562](https://github.com/aghorbani84/Aurex/actions/runs/41349931562). ARM64 job PASS; x86 runtime step FAIL. Debug APKs only, not production approval |
| Android picker/grant/revocation runtime | Emulator step FAIL before harness emitted a result in run 41349931562; precise setup/boot cause unknown. Added bounded SDK/AVD/KVM/emulator diagnostics; rerun pending. Physical-device refusal has unit coverage only |
| Dependency advisory/license gate | [Run 41349270167](https://github.com/aghorbani84/Aurex/actions/runs/41349270167) FAIL: same five advisories/NCSA. Explicit ARM64 mobile-feature graph also executed in [41349931425](https://github.com/aghorbani84/Aurex/actions/runs/41349931425); security still FAIL, no ignores/allow-list relaxation |
| Production signing/publication | NOT DONE, no secrets/keystores/release publication authorized |

Raw ignored evidence: `artifacts/evidence/production-local-checks.log`,
`production-workflow-security.log`; hosted runs/artifacts must be recorded after
actual execution, not inferred from YAML or filenames.

Additional actual hosted evidence at `be8d8ed`: core [CI 41349270285](https://github.com/aghorbani84/Aurex/actions/runs/41349270285)
PASS; Linux CLI packaging PASS; Windows native compile [41349270192](https://github.com/aghorbani84/Aurex/actions/runs/41349270192)
PASS (not Windows filesystem/runtime support). Workflow/secret-scanning job PASS.
Actions log/artifact downloads are blocked by this sandbox's outbound-host restrictions;
checks API annotations remain accessible. The first Linux runtime failure had no
application annotation; its precise cause is unknown, not assumed to be GPU-related.

The pinned Dioxus Android template contains an explicit loopback-only HTTP exception
in its generated network-security resource. `usesCleartextTraffic=false` does not
override that API24+ resource. Engineering APKs must not be represented as completely
network-denied; review/remove the template exception before production distribution.
The GUI probe now directly uses the already-locked `dioxus-document`0.7.10 API; no
additional package versions or Web/wasm packages are introduced for that fix.
Native SAF now also independently rejects virtual entries and changed document kinds,
not merely disabled UI buttons. Physical/cloud provider and lifecycle validation remain open.

## Remaining production gates

- Confirm native source and Gradle integration compile, install/launch APKs and run
  Android picker/cancel/persist/revoke/virtual/cloud-provider/lifecycle tests on real
  framework implementations, including supported API levels/16KiB physical devices.
- Verify transactional persisted-grant cleanup when switching/releasing grants fails;
  the current implementation can retain an unused OS grant after a partial failure.
- Android mutations are not implemented. No Linux provider/CLI is substituted for SAF.
- Resolve the maintained GTK/WebKit dependency/advisory path and native redistribution
  notices; do not add fake direct upgrades that leave old vulnerable transitives.
- Full GUI accessibility, lifecycle/active-write shutdown, input/focus, install/uninstall
  and migration tests. A smoke test alone is not daily-driver acceptance.
- Android target SDK35 is an engineering baseline, not a Play eligibility guarantee.
  Review current store requirements/toolchain compatibility, Gradle dependency locks,
  permissions/backup/network policy, original icons, approved signing and distribution.
- Real storage faults/races, disk-full/power-loss, O_TMPFILE filesystem support and
  broad performance/battery measurements remain beyond existing deterministic tests.
