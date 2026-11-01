# REPORT A — AUREX engineering upgrade

Date: 2026-11-01. **Partial engineering upgrade, not production/cross-platform
acceptance.** Draft [PR #1](https://github.com/aghorbani84/Aurex/pull/1) targets the
discovered default branch `main` from `arena/c8d70e1e-aurex`.
FAZORA remains a [separate execution](FAZORA_HANDOFF.md).

## Baseline, scope and changes

[Baseline](BASELINE_2026-11-01.md): root `/home/user/Aurex`, initial commit
`e437dd825563647d06cca80b6db3240e20eca44e`, existing uncommitted core/GUI foundation,
31 passing Rust tests, separate Cargo workspaces, missing native packages/providers.
Existing changes and original MIT LICENSE preserved; no reset/history rewrite,
force push, release publication, signing/secrets or repository-setting changes.

- Added typed partial-commit/durability errors retaining the committed destination
  and OS cause: failed fsync after publication must not encourage blind retry.
- Six production-copy fault regressions and exclusive mode-0700 test roots.
- Real Linux CLI archive lifecycle tests, deterministic packaging, bounded malicious
  archive validation, SHA-256, provenance, scoped SBOM and actual Rust notices.
  Packaging rebuilds locked sources for the advertised target, not a stale executable.
- Six independent least-privilege workflows, pinned official action SHAs, weekly
  scanners/Dependabot, safe checkout/cache boundaries, fail-closed Android readiness.
- Real hosted failures drove Node 24 action compatibility updates, all-graph audit
  failure retention, bounded API-readable diagnostics and GUI type-error fixes.
- Confirmed Tokio I/O-safety advisory fixed narrowly at 1.51.6; its required macros
  package updated to 2.7.2. Remaining native GUI advisories/license rejection stay
  visible and block release. See [security review](SECURITY_AUDIT.md).

## Command verification (local)

Raw ignored logs: `artifacts/evidence/`. A passing local command is not a hosted run.

| Check | Result | Actual command/evidence |
|---|---|---|
| Formatting | PASS | `cargo fmt --all --check`; separate GUI workspace formatting |
| Core Clippy | PASS | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| Rust tests | PASS | `cargo test --workspace --locked`: 37 (31 + six fault regressions) |
| Engineering tests | PASS | `python3 -m unittest discover -s tests -v`: 10; real CLI artifacts and crafted archive/audit/compiler failures |
| Workflow structure | PASS, scoped | actionlint 1.7.12 official-source build; local shellcheck/pyflakes unavailable |
| Workflow security | PASS, scoped | zizmor 1.31.1 `--offline --persona auditor`: no findings |
| YAML/project policy | PASS | `validate_workflows.py`: six workflows |
| History secrets | PASS, scoped | actual Gitleaks 8.30.1 history scan: exit 0/no leaks, not proof of absence |
| Dependency tools | BLOCKED locally | Registry egress prevents local audit/deny installation; actual hosted results below |
| Linux CLI build/runtime | PASS | Explicit `--target x86_64-unknown-linux-gnu --release --locked`; fixture operations before/after packaging |
| Reproducibility | PASS, scoped | Two archives from identical source/binary metadata compare byte-for-byte; not cross-host reproducible compilation |
| Local GUI | BLOCKED | Missing crate tarballs/GTK/WebKit/display; local offline check exit 101 |
| Android readiness | BLOCKED, expected exit 2 | JSON reports absent mobile/provider/packaging prerequisites; not an APK build |

## Actual GitHub executions

These URLs are real runs, not definitions. Later documentation-only revisions do
not retroactively change the tested source revision. Superseded cancellations are
not compilation success or failure evidence.

| Run | Observed result and scope |
|---|---|
| [CI 41343845994](https://github.com/aghorbani84/Aurex/actions/runs/41343845994) | SUCCESS: native Ubuntu/Windows portable core checks |
| [CI 41344252459](https://github.com/aghorbani84/Aurex/actions/runs/41344252459) | SUCCESS with verified Node 24 action pins |
| [Linux 41344877670](https://github.com/aghorbani84/Aurex/actions/runs/41344877670) | CLI candidate SUCCESS/uploaded; GUI FAIL: hosted Rust diagnostics exposed E0659/E0599 |
| [Windows 41344877648](https://github.com/aghorbani84/Aurex/actions/runs/41344877648) | MSVC CLI compile/help PASS; GUI FAIL on the same source type errors; provider still unimplemented |
| [Security 41343845904](https://github.com/aghorbani84/Aurex/actions/runs/41343845904) | Scanner job PASS; old audit tool failed early, hiding subsequent graphs; original log inaccessible |
| [Security 41344252493](https://github.com/aghorbani84/Aurex/actions/runs/41344252493) | All four graphs executed; core audit/deny and desktop audit PASS, desktop deny FAIL including Tokio I/O safety |
| [Security 41344877621](https://github.com/aghorbani84/Aurex/actions/runs/41344877621) | Tokio finding gone after patch; desktop deny still FAIL on five other advisories and a license requirement; no ignores |
| [CI 41345497993](https://github.com/aghorbani84/Aurex/actions/runs/41345497993) | SUCCESS: native Linux/Windows core checks; Linux runs 37 Rust + 10 Python cases (Windows portable subset, Linux packaging case skipped) |
| [Linux 41345497988](https://github.com/aghorbani84/Aurex/actions/runs/41345497988) | CLI fresh-target candidate PASS; GUI FAIL on one RSX list ownership error, now fixed with a separate selection-path snapshot |
| [Windows 41345498006](https://github.com/aghorbani84/Aurex/actions/runs/41345498006) | FAIL before the final RSX ownership fix; no Windows filesystem acceptance |
| [Security 41345497976](https://github.com/aghorbani84/Aurex/actions/runs/41345497976) | FAIL: five advisories plus libfuzzer-sys 0.4.13 NCSA policy rejection identified; see security review |

**Final tested implementation revision:** `9bf74057269c6e869e004934b84c3ec793294af4`.

| Final real run | Observed completed result |
|---|---|
| [CI 41346292796](https://github.com/aghorbani84/Aurex/actions/runs/41346292796) | **SUCCESS**: Linux/Windows core checks and engineering tests, platform-scoped |
| [Linux 41346292730](https://github.com/aghorbani84/Aurex/actions/runs/41346292730) | **SUCCESS**: CLI runtime/package + native Dioxus release compilation (desktop job 5m39s); GUI runtime NOT TESTED |
| [Windows 41346292701](https://github.com/aghorbani84/Aurex/actions/runs/41346292701) | **SUCCESS**: MSVC core/CLI help + native Dioxus release compilation (6m12s); filesystem provider/runtime absent |
| [Security 41346292703](https://github.com/aghorbani84/Aurex/actions/runs/41346292703) | **FAIL**: workflow/secret scanner job PASS; all dependency graphs executed; desktop deny retains five advisory gates and NCSA policy rejection |

Documentation-only follow-ups do not change the tested implementation. Their newly
triggered checks must be read separately rather than borrowing a previous green run.

GUI type fixes disambiguate the core `Location` from Dioxus's prelude and call
struct-held Signals with `(explorer.field)()` rather than nonexistent methods.
A subsequent hosted E0382 identified RSX evaluation consuming the entries vector
before its selection callback cloned it; the callback now owns a distinct path-only
snapshot created before RSX construction. The final native Linux/Windows runs above
confirmed that follow-up compiles successfully; they do not validate GUI interaction.
The sandbox cannot reach GitHub's `results-receiver.actions.githubusercontent.com`
log-storage host. The new harness safely disables command interpretation while
printing tool output, then emits bounded escaped compiler/finding annotations
accessible through the GitHub API. Failures remain nonzero; no log-access limitation
is used to claim the underlying build passed. Full native GUI Clippy/runtime/UI
acceptance is not established by the core job or a build-only job.

## Artifacts

Actual [initial hosted CLI artifact](https://github.com/aghorbani84/Aurex/actions/runs/41343845985/artifacts/13098566301)
was uploaded after real compilation and package runtime tests. It belongs to the
initial PR merge revision, not necessarily the latest branch head. Current CLI runs
also upload independently checksummed candidates; use each run's provenance.

A newer [fresh-target CLI artifact](https://github.com/aghorbani84/Aurex/actions/runs/41345497988/artifacts/13098668734)
was uploaded from PR merge revision `d653eb60fa6d018f5631597077bb29717648a940`
(branch head `656574c745aba7926505a8301203819e59db6052`), 665,413 bytes,
GitHub artifact-container digest
`sha256:b240455c4d49d0e8fed11c8d788eefced4bfb4f25829dd41395d078c99b3553d`.
This is not the inner tar.gz checksum; that package has its own generated sidecar.

The final implementation's [validated CLI artifact](https://github.com/aghorbani84/Aurex/actions/runs/41346292730/artifacts/13098714885)
is 665,410 bytes, PR merge revision `a19260f3cb23b252dcabb32ed29d212e910b73d7`,
branch head `9bf74057269c6e869e004934b84c3ec793294af4`. GitHub artifact-container digest:
`sha256:668e1920d7becec92cfc005bd0b30042d86febd4ca766da53ed700c1e6a53327`.
No GUI executable was distributed: compiler diagnostics are separate from product artifacts.

Local ignored `artifacts/packages/aurex-cli-0.1.0-x86_64-unknown-linux-gnu.tar.gz`
has adjacent `.sha256`, `build-metadata.json` and `sbom.cdx.json`. Contents include
per-member hashes, source/toolchain/dirty metadata, original LICENSE, actual Rust
notices and dynamic native dependency inventory. Candidates are unsigned CLI-only,
not GUI/installers/static-all-distribution binaries or official Releases. The scoped
SBOM covers root Cargo.lock, not complete native composition/redistribution clearance.
See [release guide](../RELEASES.md). No Windows provider package or APK/AAB is shipped.

## Measured performance, no optimization claim

Actual original 11-trial Linux warm-cache release measurements:

| CLI workload | p50 ms | p95/p99 ms |
|---|---:|---:|
| Help/process startup | 1.479 | 1.936 |
| 1,000-entry listing | 7.100 | 7.457 |
| 8,388,606-byte copy + full verification/fsync | 19.157 | 21.608 |

With 11 trials both tail columns select the maximum and are not robust estimates.
No new optimization, before/after improvement, GUI latency, Windows/Android throughput,
battery or memory benchmark is claimed. [Method/gaps](../PERFORMANCE.md).

## Acceptance blockers / independent work

- Resolve remaining native GUI security/license findings via maintained compatible
  upstream dependencies, with migration/API/native-runtime evidence, not ignores.
- Hosted compilation fixes passed; run the GUI on a real display and test focus/keyboard/touch,
  active-job shutdown, accessibility, native dependencies and full GUI lint.
- Implement/test real Windows provider and Android SAF/mobile/native package path.
- Real disk-full/short-write/kernel fsync, hostile identity replacement/mount/hardlink
  boundaries and grant revocation exceed injected-test evidence. Root is not a sandbox.
- Complete native redistribution/notices review, install/uninstall tests, protected
  authorized signing/publication, immutable images and broader filesystem performance.
- Aarch64 Linux unimplemented; FAZORA playback/FFI/engine licensing belongs to its
  separate checkout. No workspaces, pipelines or package identities were merged.
