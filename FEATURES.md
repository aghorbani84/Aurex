# Feature availability

**Tested** means covered locally on Linux x86-64 core/CLI. **UI source** means wired
implementation exists but has not passed compilation/runtime acceptance.

| Feature | State | Limits |
|---|---|---|
| Root-scoped listing and metadata | Tested | procfs, 50,000 examined entries; size/kind/mtime/readonly |
| Native names, hidden files, substring filtering | Tested | No regex or recursive search |
| Name sort and paging | Tested | Entire bounded directory sorted before paging |
| Copy | Tested | Regular files; O_TMPFILE + link publication needed; private output mode |
| Move / rename | Tested | Same filesystem; no replacement |
| Conflict fail / skip / keep both | Tested | Transfers only; keep-both suffix up to 1000 attempts |
| Directory creation | Tested | Mode 0700 subject to umask |
| Delete | Tested | Permanent file/link or empty-directory removal; no trash |
| Cancellation / progress | Tested core | Cooperative, not rollback after commit; byte count is attempted I/O |
| Text preview | Tested | UTF-8, 256 KiB; binary/control bytes rejected |
| Independent history / tabs / selection model | Tested core | Session-only; bounded |
| Dual panes / responsive switching / themes | UI source | No verified GUI runtime |
| UI dialogs / batch transfers / activity / bookmarks | UI source | Sequential batch; per-file error display; in-memory only |
| Persistent sessions / bookmarks / operation history | Not implemented | No database yet |
| Recursive copy/delete, verified cross-volume move | Not implemented | Explicit refusal, not unsafe fallback |
| Trash / restore / file associations / clipboard / drag-drop | Not implemented | No decorative controls offered |
| Advanced sorting, tree, preview formats, editors | Not implemented | Text preview only |
| Bulk rename / compare / sync / hashing / duplicates | Not implemented | No destructive plans generated |
| Archives / APK workbench | Not implemented | No archive parsing or extraction |
| Device/mount/native permissions manager | Not implemented | No privilege escalation |
| Windows filesystem / Android SAF | Not implemented | Provider constructor explicitly refuses |
| Remote providers / plugins / assistant tools | Not implemented | No credential or network exposure |
| Installers / signing / release packages | Not implemented | No platform release claim |

The long-term target is the phased roadmap, not a description of this build.
