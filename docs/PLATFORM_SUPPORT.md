# Platform capability matrix

As of 2026-11-02. **Tested core** is not a supported production application.
The user definition of platform support requires a real app build + runtime validation;
no complete platform has passed that gate yet.

| Capability | Linux x86-64 | Windows | Android |
|---|---|---|---|
| Root core/CLI build + real native ops | Tested | Native build/core/help PASS; provider Unsupported | Not run; provider Unsupported |
| Local browsing/filter/metadata | Tested core | Not implemented | SAF implemented; runtime gate open |
| Regular file copy | Tested core, FS-dependent | Not implemented | Not implemented |
| Same-volume move/rename | Tested core | Not implemented | Not implemented |
| Cross-volume verified move | Unsupported | Not implemented | Not implemented |
| mkdir / nonrecursive permanent delete | Tested core | Not implemented | Not implemented |
| Symlink no-follow boundary | Tested core | Reparse policy not implemented | Provider-specific policy pending |
| Text preview | Tested core | Not implemented | SAF UTF-8 implementation; runtime gate open |
| Dual panes/tabs/adaptive UI | Native GUI compile + scoped WebView smoke PASS | Native GUI compile PASS; no native provider/runtime acceptance | ARM64/x86-64 native build PASS; separate read-only UI, runtime not accepted |
| Trash / restore | Not implemented | Not implemented | Not implemented |
| Permissions / ownership / ACL manager | Not implemented | Not implemented | Provider flags pending |
| Mount/device/storage dashboard | Not implemented | Not implemented | Not implemented |
| Archives / APK / remote / plugins | Not implemented | Not implemented | Not implemented |
| Native runtime validation / packaging | CLI archive and WebView smoke verified; full GUI acceptance open, no installers | Core/help only, no functional provider package | Debug APK verifier PASS for both ABIs; emulator step FAIL, no production package |

Linux implementation is architecture-gated to x86-64, uses procfs and Linux-specific
open/rename/link flags. Other architectures are not enabled merely because they run
Linux. File copying additionally needs O_TMPFILE plus hardlink publication support;
unsupported filesystems fail with source data preserved. Network mounts under a
local root do not gain a remote provider's semantics or support guarantee.

Native Linux/Windows CI ran successfully at source 9bf7405; actual run links are in
reports/AUREX_ENGINEERING_REPORT.md. GUI dependency policy still fails and GUI
runtime is not accepted. Android now has a real SAF/JNI project and debug APKs; build success does not establish
Android runtime support. See the [current report](reports/PRODUCTION_READINESS_2026-11-02.md)
for executed runs and remaining permission/provider/device/security gates.
