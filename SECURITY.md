# Security model and limitations

This is pre-release software, **not independently audited**. Start with disposable
fixtures, least-privilege OS accounts and a narrowly selected root.

## Implemented defenses

- A provider root is an explicit grant; writes additionally require `--write`.
- Reject absolute/parent/NUL paths. Component-by-component directory handles plus
  `O_NOFOLLOW` block parent symlink traversal. Root identity stays anchored if its
  visible path is later renamed. Links are listed but never opened for reading.
- Reads for copying/previews are regular-file-only and nonblocking on open.
- Anonymous staging cannot be replaced by swapping a named temporary file. It is
  reclaimed by the kernel when the FD closes, including on cancellation/crash.
- Destination publication and move/rename never replace existing paths. No automatic
  retry with overwrite. Equal sizes are not used as proof of equal contents.
- Copy verification uses bounded buffers; previews and directory enumeration have
  limits; concurrency and queues are bounded. No shell interpolation.
- Deletion is nonrecursive. CLI requires `--confirm`; UI source requires confirmation.
- Previewed files are never executed; text is rendered as text, not trusted HTML.
- No network filesystem service, external uploads, plugins, stored credentials or
  elevated operations. No signing keys or passwords are required or committed.

## Boundaries, not sandbox claims

A root grant is a **filesystem capability**, not an OS security sandbox. Choosing an
initial symlink root grants its canonical target. Hardlinks within a granted tree
can refer to data also named elsewhere; mount points can cross volumes. A parent
handle remains authority over that directory if another process moves it outside
the tree. The original displayed root string may then be stale. Same-UID malicious
processes, hostile mount namespaces, a compromised procfs/kernel, and privileged
attackers are outside the current threat model.

No directory snapshot is obtained. Entries can disappear between list and action;
metadata may be stale. A named source/destination can be replaced by another process
before a rename/delete; no file-ID confirmation is implemented. Source mutation is
detected during ordinary copy using metadata and full readback, but there is no
cryptographic or immutable snapshot guarantee against adversarial concurrent writes.
Other writers may alter committed files later. Avoid shared untrusted writable trees.

Copy output deliberately uses mode 0600; it does not preserve executable bits,
ownership, timestamps, ACLs, xattrs, sparse layout or hardlink relationships. No
cross-filesystem move, overwrite, trash, recursive traversal, archive extraction,
plugin loading or privileged backend exists. Those need separate threat models.

Cancellation cannot revoke an already committed operation. A directory fsync failure
after copy publication is reported explicitly as **destination committed, durability
sync failed**, with typed `Durability` and a `committed` location; inspect the
destination before retrying. Deterministic fault tests cover this distinction. Rename/delete/mkdir are not
journaled or promised crash-durable. The UI has no shutdown guard or durable audit
trail yet. The GUI is not accepted as functional until compilation, runtime,
keyboard/focus and untrusted-content tests pass.

## Reporting

Do not post sensitive filenames, contents or credentials in public issues. Use a
repository private vulnerability report if enabled; otherwise ask the maintainer
for a private reporting channel. No private reporting SLA is currently established.

See [scoped engineering audit](docs/reports/SECURITY_AUDIT.md) for findings, fixes and open risks.
