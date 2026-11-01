# Platform capability matrix

As of 2026-11-01. **Tested core** is not a supported production application.
The user definition of platform support requires a real app build + runtime validation;
no complete platform has passed that gate yet.

| Capability | Linux x86-64 | Windows | Android |
|---|---|---|---|
| Root core/CLI build + real native ops | Tested | Native build/core/help PASS; provider Unsupported | Not run; provider Unsupported |
| Local browsing/filter/metadata | Tested core | Not implemented | SAF not implemented |
| Regular file copy | Tested core, FS-dependent | Not implemented | Not implemented |
| Same-volume move/rename | Tested core | Not implemented | Not implemented |
| Cross-volume verified move | Unsupported | Not implemented | Not implemented |
| mkdir / nonrecursive permanent delete | Tested core | Not implemented | Not implemented |
| Symlink no-follow boundary | Tested core | Reparse policy not implemented | Provider-specific policy pending |
| Text preview | Tested core | Not implemented | Not implemented |
| Dual panes/tabs/adaptive UI | Native GUI compile PASS; runtime NOT TESTED | Native GUI compile PASS; no native provider/runtime acceptance | Mobile build not implemented |
| Trash / restore | Not implemented | Not implemented | Not implemented |
| Permissions / ownership / ACL manager | Not implemented | Not implemented | Provider flags pending |
| Mount/device/storage dashboard | Not implemented | Not implemented | Not implemented |
| Archives / APK / remote / plugins | Not implemented | Not implemented | Not implemented |
| Native runtime validation / packaging | CLI archive smoke verified; no GUI/installers | Core/help only, no functional provider package | None |

Linux implementation is architecture-gated to x86-64, uses procfs and Linux-specific
open/rename/link flags. Other architectures are not enabled merely because they run
Linux. File copying additionally needs O_TMPFILE plus hardlink publication support;
unsupported filesystems fail with source data preserved. Network mounts under a
local root do not gain a remote provider's semantics or support guarantee.

Native Linux/Windows CI ran successfully at source 9bf7405; actual run links are in
reports/AUREX_ENGINEERING_REPORT.md. GUI dependency policy still fails and GUI
runtime is not accepted. No Android build job claims a working package when the
bridge and mobile project do not exist.
