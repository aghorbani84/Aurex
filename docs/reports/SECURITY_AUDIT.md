# AUREX scoped security review — 2026-11-01

This is an engineering review with regression evidence, **not an independent audit,
certification or assertion of absence of vulnerabilities**.

| ID | Priority / finding | Disposition and evidence |
|---|---|---|
| AX-01 | P1 copy published then fsync fails: generic error could encourage blind retry | FIXED: typed Durability + committed destination + retained OS cause/display warning; deterministic postcommit regression |
| AX-02 | P1 midstream cancel/source-change/disk-full failure paths untested | IMPROVED: six private-copy checkpoint tests run the same production engine; no real disk exhaustion; remaining short-write/kernel integration gaps documented |
| AX-03 | P2 checkout credentials persist in workflow workspaces | FIXED: all checkouts persist-credentials false; baseline zizmor found 3 medium warnings, current auditor scan reports no findings |
| AX-04 | P2 unsigned artifacts lacked validation/provenance | IMPROVED: CLI-only runtime smoke, deterministic archive, SHA-256, metadata, scoped SBOM and actual Rust notices; no official publication or signing claim |
| AX-05 | P1 crafted package PAX header/decompression could consume memory before ordinary member checks | FIXED in new verifier: bounded expansion to a private stream, bounded read/seek wrapper before tar parsing, name/type/count/size/duplicate/hash validation; malicious fixtures pass |
| AX-06 | P1 native Windows/Android and GUI runtime unverified | OPEN: no functional provider/SDK; fail-closed Android report; no falsely usable platform installer |
| AX-07 | P1 hostile concurrent rename/delete identity replacement; mount/hardlink boundary | OPEN: root is capability scope, not an OS sandbox; no immutable identity-conditional delete; avoid shared hostile writable roots |
| AX-08 | P2 dependency vulnerabilities/native redistribution obligations not fully audited | OPEN pending actual cargo-audit/deny jobs; no exception list or success-by-ignore |

## Reviewed boundaries

- Root and entry opens use live descriptors and no-follow single components;
  absolute/parent/NUL/oversized locations and filters fail before queued I/O.
- All Linux unsafe blocks reviewed for Linux x86-64 flags, CString lifetime, live FD
  ownership, variadic mode type and single ownership transfer. File drops reclaim FDs.
  There are no native callbacks or Rust pointers retained by C. Panic catching stays
  around Rust jobs, not across an FFI callback ABI. This is source review, not Miri/
  sanitizer proof of all platform behavior.
- Copy stages are anonymous O_TMPFILE; bounded buffers and full readback precede
  link publication. Existing names are never replaced. EXDEV is refused for moves;
  no source deletion occurs after a failed copy. Post-publication cancellation does
  not pretend to roll back committed data. Source snapshots remain best-effort.
- Recursive deletion, archive extraction, ZIP/APK editing, remote credentials,
  Android SAF and Windows reparse paths are absent, not implicitly considered safe.
  Package archive validation is separate from an implemented archive file manager.
- User text is bounded/plain UTF-8; filenames retain native identity and are escaped
  by UI/text output. Shell interpolation is not used in operational code.
- Directory/preview/queue limits, per-file errors and private fixtures are real;
  file-size streaming can still consume destination disk until OS failure. Injected
  ENOSPC is not proof of every actual filesystem's disk-full semantics.
- GUI active-job shutdown, focus/accessibility and WebView/host OS security remain
  runtime acceptance work. No entire-filesystem AI/network endpoint exists.

## Scanner evidence

`artifacts/evidence/baseline-zizmor.log`: exit 13, 3 medium credential-persistence
warnings. `artifacts/evidence/zizmor.log`: exit 0, auditor persona, "No findings to
report". `actionlint.log`: exit 0 (shellcheck/pyflakes unavailable locally, core
workflow structural/type checks enabled). These are scoped workflow results, not
a declaration that dependency or application security is complete.

Gitleaks built from official v8.30.1 tagged source was also run locally: exit 0,
"2 commits scanned", "no leaks found" (redacted report). Local Go dependency metadata
was adapted for the egress-constrained build; no scanner/native binary is shipped in
AUREX. GitHub independently ran the normal official-source scanner job successfully.
Rust fixtures now explicitly use mode-0700 exclusive roots, not just unique names.

## Confirmed hosted dependency findings (not a clean bill of health)

Run [41344252493](https://github.com/aghorbani84/Aurex/actions/runs/41344252493)
executed every graph. Only **desktop-deny** failed; both cargo-audit invocations and
core deny exited 0. Deny catches stricter unsound/unmaintained policy than audit's
default. The visible annotation confirms Tokio 1.48.0's **RUSTSEC-2026-0386** I/O
safety issue and old rand 0.7.3 via phf_generator → selectors → kuchikiki → wry.
No ignore was added. Tokio is narrowly updated to **1.51.6**, a fixed branch named
in the advisory (registry MSRV 1.71/checksum verified), requiring tokio-macros 2.7.2.
The lock diff updates only those two packages; this is not a mass dependency upgrade.
The remaining obsolete transitive GUI ecosystem is a release blocker pending full
annotated rerun. The root 0.21.2 audit failure disappeared with verified 0.22.2;
the original inaccessible diagnostic was never established, so no causal claim is made.
