# File operation contract

## Grant and identity

`LocalProvider::new(root, writable, scheduler)` chooses the only initial storage
scope. Operations accept relative `Location::Local` paths; `..`, absolute and NUL
paths are rejected. Relative paths are capped at 16 KiB; filename filters at
1024 bytes. Initial root symlinks resolve during the explicit grant;
subsequent parent/file symlinks are not followed. An unsupported provider is an
error. OS syscall permissions are authoritative even when write consent is given.

## Listing / preview

At most 50,000 directory entries are examined. All retained metadata is sorted by
native name (directories first in ascending order) and optionally reversed before
paging. There is no filesystem snapshot: paging can change if the directory changes.
Limit is 1..2000 (default/UI 500); filtering is current-directory substring matching.
Disappeared metadata generates up to 32 warnings. No permission failure becomes an
empty directory silently. The implementation is bounded, not incremental/virtualized.

Text preview opens regular files only, caps both metadata and streamed reads at
256 KiB, accepts UTF-8, and refuses NUL/control bytes except tab/newline/carriage
return. No file is executed and no text is parsed as HTML. Symlinks/FIFOs/devices
cannot be copied/previewed as regular files.

## Copy

1. Open root-relative source and destination-parent directory descriptors.
2. Open source no-follow/nonblocking and verify regular-file type.
3. Create an anonymous mode-0600 `O_TMPFILE` on the destination filesystem.
4. Stream using a fixed 128 KiB buffer, checking cancellation per chunk; fsync file.
5. Rewind both descriptors and compare every byte with a second fixed buffer.
   Compare source device/inode/size/mtime/ctime before and after copying.
6. Check cancellation, then publish the verified FD using no-replace `linkat` via
   its controlled procfs descriptor path. Existing destinations are never replaced.
7. Fsync the destination directory. If this fails, report **committed, durability
   sync failed** in the error, with `ErrorKind::Durability` and `Error.committed` naming the
   published destination. It exists despite the error; inspect before retry.

FD ownership cleans up anonymous staging on ordinary errors, cancellation and process
exit. Source is never deleted. Destination filesystem must support anonymous files
and hardlink publication; errors do not trigger an insecure named-file fallback.
No cross-filesystem atomicity is claimed. Output intentionally does **not** preserve
permissions/ownership/timestamps/xattrs/ACLs/sparse layout/executable bits; mode 0600
is subject to the process umask. Source files must be stable: no immutable snapshot
or defense against adversarial concurrent modification is promised.

## Move and rename

`renameat2(RENAME_NOREPLACE)` provides same-filesystem namespace movement without
replacement. Moving directories does not recursively traverse their contents.
EXDEV returns `Unsupported`, preserving the source. Verified cross-volume move and
identity-safe deletion of the source are future work. There is no overwrite option.

## Conflicts

Transfers support `Fail`, `Skip`, `KeepBoth`. Keep-both appends ` (copy N)` to the
entire proposed filename (including after its extension), tries at most 1000 names,
and still uses no-replace syscalls to handle races. An advisory no-follow existence
check avoids unnecessary copy I/O on already occupied names. A skip is not successful data
transfer. The CLI always selects Fail. Directory creation reports existing-name
errors; transfer conflict policies do not apply to mkdir/delete.

## Delete / create

Delete removes a named file or symlink itself. Directories must be empty; there is
no recursive traversal. Permanent means **no undo, trash, or restore**. CLI needs
`ROOT --write delete PATH --confirm`. GUI source shows scope and confirmation.
No operation may modify the granted root entry itself. Folders are created mode
0700 subject to umask. mkdir/move/delete are not journaled or promised crash-durable.

## Progress, cancellation, partial batches

Queue capacity is bounded; callers must surface/retry backpressure instead of
pretending a job was accepted. Each task returns its own result. Shared byte progress
counts attempted streamed copy bytes, including copies that later conflict/fail;
completed item counts increment only on success. Verification reads are not included.
Progress is cumulative, not a percentage/ETA. UI batches process one item at a time,
record errors individually, and stop submitting after cancellation. Dropping a task
is not cancellation. Cancellation cannot roll back committed operations, or undo
previous successful items. There is no durable recovery, transaction across files,
automatic retry, or persistent audit history yet.
