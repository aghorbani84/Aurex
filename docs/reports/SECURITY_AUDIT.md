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
