//! Small Linux-only syscall boundary. All entry names are single components; parent handles
//! are walked with O_DIRECTORY|O_NOFOLLOW. Never fall back to unchecked absolute paths.
use super::*;
use std::os::raw::{c_char, c_int, c_uint};
use std::{
    ffi::{CString, OsStr, OsString},
    fs::{File, Metadata},
    io::{Read, Seek, SeekFrom, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{ffi::OsStrExt, fs::MetadataExt},
    },
};

const O_RDONLY: c_int = 0;
const O_RDWR: c_int = 2;
const O_TMPFILE: c_int = 0o20000000 | O_DIRECTORY;
const AT_FDCWD: c_int = -100;
const AT_SYMLINK_FOLLOW: c_int = 0x400;
const O_NONBLOCK: c_int = 0o4000;
const O_DIRECTORY: c_int = 0o200000;
const O_NOFOLLOW: c_int = 0o400000;
const O_CLOEXEC: c_int = 0o2000000;
const AT_REMOVEDIR: c_int = 0x200;
const RENAME_NOREPLACE: c_uint = 1;
const MAX_ENTRIES: usize = 50_000;
const MAX_PREVIEW: u64 = 256 * 1024;

unsafe extern "C" {
    fn openat(dir: c_int, name: *const c_char, flags: c_int, ...) -> c_int;
    fn mkdirat(dir: c_int, name: *const c_char, mode: c_uint) -> c_int;
    fn unlinkat(dir: c_int, name: *const c_char, flags: c_int) -> c_int;
    fn linkat(
        olddir: c_int,
        old: *const c_char,
        newdir: c_int,
        new: *const c_char,
        flags: c_int,
    ) -> c_int;
    fn renameat2(
        olddir: c_int,
        old: *const c_char,
        newdir: c_int,
        new: *const c_char,
        flags: c_uint,
    ) -> c_int;
}
fn cname(name: &OsStr) -> Result<CString> {
    if name.is_empty() || name.as_bytes().contains(&b'/') || name == "." || name == ".." {
        return Err(Error::new(ErrorKind::InvalidLocation, "invalid entry name"));
    }
    CString::new(name.as_bytes()).map_err(|_| Error::new(ErrorKind::InvalidLocation, "NUL in path"))
}
fn syscall_result(rc: c_int, context: &str) -> Result<()> {
    if rc < 0 {
        Err(Error::io(context, std::io::Error::last_os_error()))
    } else {
        Ok(())
    }
}
fn open(parent: &File, name: &OsStr, flags: c_int) -> Result<File> {
    let name = cname(name)?;
    // SAFETY: parent is a live directory descriptor; name is NUL-terminated; mode is
    // supplied even when not used. O_NOFOLLOW blocks link traversal at this component.
    let fd = unsafe {
        openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            flags | O_NOFOLLOW | O_CLOEXEC,
            0o600 as c_uint,
        )
    };
    if fd < 0 {
        return Err(Error::io(
            "open entry (links are not followed)",
            std::io::Error::last_os_error(),
        ));
    }
    // SAFETY: successful openat returned a new owned descriptor, transferred exactly once.
    Ok(unsafe { File::from_raw_fd(fd) })
}
fn directory(root: &File, path: &Path) -> Result<File> {
    let mut dir = root
        .try_clone()
        .map_err(|e| Error::io("clone root descriptor", e))?;
    for part in path.components() {
        let Component::Normal(name) = part else {
            return Err(Error::new(
                ErrorKind::InvalidLocation,
                "invalid directory component",
            ));
        };
        dir = open(&dir, name, O_RDONLY | O_DIRECTORY)?;
    }
    Ok(dir)
}
pub(super) fn grant_root(path: &Path) -> Result<File> {
    let filesystem_root = File::open("/").map_err(|e| Error::io("open filesystem root", e))?;
    let relative = path
        .strip_prefix("/")
        .map_err(|_| Error::new(ErrorKind::InvalidLocation, "grant root must be absolute"))?;
    directory(&filesystem_root, relative)
}
fn parent(root: &File, path: &Path) -> Result<(File, OsString)> {
    let name = path.file_name().ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidLocation,
            "the granted root cannot be modified",
        )
    })?;
    Ok((
        directory(root, path.parent().unwrap_or(Path::new("")))?,
        name.to_owned(),
    ))
}
fn regular(dir: &File, name: &OsStr) -> Result<File> {
    let file = open(dir, name, O_RDONLY | O_NONBLOCK)?;
    if !file
        .metadata()
        .map_err(|e| Error::io("file metadata", e))?
        .is_file()
    {
        return Err(Error::new(
            ErrorKind::Unsupported,
            "only regular files can be copied or previewed",
        ));
    }
    Ok(file)
}
fn proc_path(dir: &File) -> PathBuf {
    PathBuf::from(format!("/proc/self/fd/{}", dir.as_raw_fd()))
}

pub(super) fn list(root: &File, path: &Path, options: ListOptions) -> Result<DirectoryPage> {
    let dir = directory(root, path)?;
    let mut entries = Vec::new();
    let mut warnings = Vec::new();
    let filter = options.filter.to_lowercase();
    let iter = std::fs::read_dir(proc_path(&dir))
        .map_err(|e| Error::io("read directory (procfs required)", e))?;
    for (examined, item) in iter.enumerate() {
        if examined >= MAX_ENTRIES {
            return Err(Error::new(
                ErrorKind::Limit,
                "directory exceeds the current 50,000-entry safety limit",
            ));
        }
        let item = item.map_err(|e| Error::io("read directory entry", e))?;
        let name = item.file_name();
        let display = name.to_string_lossy();
        if (!options.show_hidden && display.starts_with('.'))
            || !display.to_lowercase().contains(&filter)
        {
            continue;
        }
        let metadata = match std::fs::symlink_metadata(item.path()) {
            Ok(meta) => meta,
            Err(e) => {
                if warnings.len() < 32 {
                    warnings.push(format!("metadata unavailable: {e}"));
                }
                continue;
            }
        };
        let kind = if metadata.is_symlink() {
            EntryKind::Symlink
        } else if metadata.is_dir() {
            EntryKind::Directory
        } else if metadata.is_file() {
            EntryKind::File
        } else {
            EntryKind::Special
        };
        entries.push(Entry {
            location: Location::local(path.join(&name)),
            name,
            kind,
            size: if kind == EntryKind::File {
                Some(metadata.len())
            } else {
                None
            },
            modified: metadata.modified().ok(),
            readonly: metadata.permissions().readonly(),
        });
    }
    entries.sort_by(|a, b| {
        (a.kind != EntryKind::Directory)
            .cmp(&(b.kind != EntryKind::Directory))
            .then_with(|| a.name.cmp(&b.name))
    });
    if options.descending {
        entries.reverse();
    }
    let total = entries.len();
    let end = options.offset.saturating_add(options.limit).min(total);
    let next_offset = (end < total).then_some(end);
    let entries = entries
        .into_iter()
        .skip(options.offset)
        .take(options.limit)
        .collect();
    Ok(DirectoryPage {
        entries,
        total,
        next_offset,
        warnings,
    })
}

pub(super) fn preview(root: &File, path: &Path) -> Result<String> {
    let (dir, name) = parent(root, path)?;
    let file = regular(&dir, &name)?;
    if file
        .metadata()
        .map_err(|e| Error::io("preview metadata", e))?
        .len()
        > MAX_PREVIEW
    {
        return Err(Error::new(
            ErrorKind::Limit,
            "text preview is limited to 256 KiB",
        ));
    }
    let mut bytes = Vec::new();
    file.take(MAX_PREVIEW + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| Error::io("read preview", e))?;
    if bytes.len() as u64 > MAX_PREVIEW {
        return Err(Error::new(
            ErrorKind::Limit,
            "file grew beyond preview limit",
        ));
    }
    if bytes
        .iter()
        .any(|&b| b == 0 || (b < 0x20 && !matches!(b, b'\n' | b'\r' | b'\t')))
    {
        return Err(Error::new(
            ErrorKind::Unsupported,
            "binary/control data is not previewed as text",
        ));
    }
    String::from_utf8(bytes).map_err(|_| {
        Error::new(
            ErrorKind::Unsupported,
            "preview currently supports UTF-8 text only",
        )
    })
}

fn cancelled(cancel: &Cancellation) -> Result<()> {
    if cancel.is_cancelled() {
        Err(Error::new(ErrorKind::Cancelled, "cancelled before commit"))
    } else {
        Ok(())
    }
}
fn same_snapshot(a: &Metadata, b: &Metadata) -> bool {
    a.dev() == b.dev()
        && a.ino() == b.ino()
        && a.len() == b.len()
        && a.mtime() == b.mtime()
        && a.mtime_nsec() == b.mtime_nsec()
        && a.ctime() == b.ctime()
        && a.ctime_nsec() == b.ctime_nsec()
}
/// Internal deterministic checkpoints exercise the production copy path without
/// global fault flags, timing races, or filling the host filesystem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CopyCheckpoint {
    ChunkWritten,
    FileSync,
    BeforeVerify,
    BeforePublish,
    DirectorySync,
}

fn copy_file(
    src_dir: &File,
    src_name: &OsStr,
    dst_dir: &File,
    dst_name: &OsStr,
    cancel: &Cancellation,
    progress: &Progress,
) -> Result<()> {
    copy_with_checkpoints(
        src_dir,
        src_name,
        dst_dir,
        dst_name,
        cancel,
        progress,
        &mut |_| Ok(()),
    )
}
fn copy_with_checkpoints(
    src_dir: &File,
    src_name: &OsStr,
    dst_dir: &File,
    dst_name: &OsStr,
    cancel: &Cancellation,
    progress: &Progress,
    checkpoint: &mut impl FnMut(CopyCheckpoint) -> Result<()>,
) -> Result<()> {
    let mut source = regular(src_dir, src_name)?;
    let before = source
        .metadata()
        .map_err(|e| Error::io("source metadata", e))?;
    // Anonymous staging cannot be swapped by another writer and is automatically
    // reclaimed on error, cancellation, panic or process exit. No insecure fallback.
    // SAFETY: live directory descriptor, fixed NUL-terminated ".", Linux O_TMPFILE
    // and a valid mode. The returned descriptor is owned exactly once below.
    let fd = unsafe {
        openat(
            dst_dir.as_raw_fd(),
            c".".as_ptr(),
            O_TMPFILE | O_RDWR | O_CLOEXEC,
            0o600 as c_uint,
        )
    };
    if fd < 0 {
        return Err(Error::io(
            "create anonymous staging file (filesystem must support O_TMPFILE)",
            std::io::Error::last_os_error(),
        ));
    }
    // SAFETY: fd is the unique owned descriptor returned by successful openat.
    let mut output = unsafe { File::from_raw_fd(fd) };
    let mut buffer = [0u8; 128 * 1024];
    let mut copied = 0u64;
    loop {
        cancelled(cancel)?;
        let n = source
            .read(&mut buffer)
            .map_err(|e| Error::io("read source", e))?;
        if n == 0 {
            break;
        }
        copied += n as u64;
        if copied > before.len() {
            return Err(Error::new(ErrorKind::Changed, "source grew while copying"));
        }
        output
            .write_all(&buffer[..n])
            .map_err(|e| Error::io("write staging file", e))?;
        progress.add_bytes(n as u64);
        checkpoint(CopyCheckpoint::ChunkWritten)?;
    }
    checkpoint(CopyCheckpoint::FileSync)?;
    output
        .sync_all()
        .map_err(|e| Error::io("sync staging file", e))?;
    checkpoint(CopyCheckpoint::BeforeVerify)?;
    // Verify every copied byte using bounded buffers, rather than assuming equal sizes mean equal contents.
    source
        .seek(SeekFrom::Start(0))
        .map_err(|e| Error::io("rewind source", e))?;
    output
        .seek(SeekFrom::Start(0))
        .map_err(|e| Error::io("rewind staging file", e))?;
    let mut other = [0u8; 128 * 1024];
    let mut remaining = before.len();
    while remaining > 0 {
        cancelled(cancel)?;
        let n = remaining.min(buffer.len() as u64) as usize;
        source
            .read_exact(&mut buffer[..n])
            .map_err(|e| Error::io("verify source", e))?;
        output
            .read_exact(&mut other[..n])
            .map_err(|e| Error::io("verify destination", e))?;
        if buffer[..n] != other[..n] {
            return Err(Error::new(
                ErrorKind::Changed,
                "source changed during verification",
            ));
        }
        remaining -= n as u64;
    }
    if copied != before.len()
        || !same_snapshot(
            &before,
            &source
                .metadata()
                .map_err(|e| Error::io("check source", e))?,
        )
    {
        return Err(Error::new(
            ErrorKind::Changed,
            "source changed during copy; destination was not committed",
        ));
    }
    checkpoint(CopyCheckpoint::BeforePublish)?;
    cancelled(cancel)?;
    let dst_name = cname(dst_name)?;
    let staged_descriptor = CString::new(format!("/proc/self/fd/{}", output.as_raw_fd()))
        .expect("descriptor contains no NUL");
    // SAFETY: source is a controlled procfs link to our still-open anonymous file,
    // not a user-supplied symlink. Target is one validated name relative to a live
    // directory handle. linkat refuses existing destinations, including symlinks.
    let rc = unsafe {
        linkat(
            AT_FDCWD,
            staged_descriptor.as_ptr(),
            dst_dir.as_raw_fd(),
            dst_name.as_ptr(),
            AT_SYMLINK_FOLLOW,
        )
    };
    syscall_result(rc, "commit verified copy without overwrite")?;
    checkpoint(CopyCheckpoint::DirectorySync)
        .and_then(|()| {
            dst_dir
                .sync_all()
                .map_err(|e| Error::io("directory durability sync", e))
        })
        .map_err(|mut error| {
            error.kind = ErrorKind::Durability;
            error.message = format!(
                "destination committed, but durability sync failed: {}",
                error.message
            );
            error
        })?;
    Ok(())
}

fn annotate_copy_error(error: Error, destination: PathBuf) -> Error {
    if error.kind == ErrorKind::Durability {
        error.after_commit(Location::local(destination))
    } else {
        error
    }
}

pub(super) fn execute(
    root: &File,
    op: Operation,
    cancel: &Cancellation,
    progress: &Progress,
) -> Result<Outcome> {
    let source = local_path(&op.source)?;
    let (src_dir, src_name) = parent(root, &source)?;
    cancelled(cancel)?;
    if op.kind == OperationKind::CreateDirectory {
        let name = cname(&src_name)?;
        // SAFETY: validated single-component name and live parent directory.
        let rc = unsafe { mkdirat(src_dir.as_raw_fd(), name.as_ptr(), 0o700) };
        syscall_result(rc, "create directory")?;
        progress.finish_file();
        return Ok(Outcome::Done(op.source));
    }
    if op.kind == OperationKind::DeleteEmpty {
        let name = cname(&src_name)?;
        // First unlink files/links without following them. EISDIR triggers empty-directory removal only.
        // SAFETY: valid parent and name. unlinkat never follows the named entry's symlink.
        let rc = unsafe { unlinkat(src_dir.as_raw_fd(), name.as_ptr(), 0) };
        if rc < 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(21) {
                return Err(Error::io("delete file", error));
            }
            // SAFETY: same validated parent/name; AT_REMOVEDIR cannot remove a nonempty directory.
            let rc = unsafe { unlinkat(src_dir.as_raw_fd(), name.as_ptr(), AT_REMOVEDIR) };
            syscall_result(
                rc,
                "remove empty directory (recursive delete is not supported)",
            )?;
        }
        progress.finish_file();
        return Ok(Outcome::Done(op.source));
    }
    let destination = op
        .destination
        .as_ref()
        .ok_or_else(|| Error::new(ErrorKind::InvalidLocation, "destination required"))?;
    let dest_path = local_path(destination)?;
    let (dst_dir, dst_name) = parent(root, &dest_path)?;
    if source == dest_path {
        return Err(Error::new(
            ErrorKind::Conflict,
            "source and destination are identical",
        ));
    }
    for attempt in 0..=1000 {
        cancelled(cancel)?;
        let mut candidate = dst_name.clone();
        if attempt > 0 {
            candidate.push(format!(" (copy {attempt})"));
        }
        // Cheap advisory conflict detection avoids copying large sources repeatedly for
        // already occupied names. The final no-replace syscall still closes the race.
        let available = match std::fs::symlink_metadata(proc_path(&dst_dir).join(&candidate)) {
            Ok(_) => Err(Error::new(
                ErrorKind::Conflict,
                "destination already exists",
            )),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(Error::io("inspect destination", e)),
        };
        let result = available.and_then(|()| match op.kind {
            OperationKind::Copy => {
                copy_file(&src_dir, &src_name, &dst_dir, &candidate, cancel, progress)
                    .map_err(|error| annotate_copy_error(error, dest_path.with_file_name(&candidate)))
            }
            OperationKind::Move | OperationKind::Rename => {
                let from = cname(&src_name)?;
                let to = cname(&candidate)?;
                // SAFETY: both names are valid components and parent descriptors remain live.
                // RENAME_NOREPLACE is atomic within a filesystem and never silently overwrites.
                let rc = unsafe {
                    renameat2(
                        src_dir.as_raw_fd(),
                        from.as_ptr(),
                        dst_dir.as_raw_fd(),
                        to.as_ptr(),
                        RENAME_NOREPLACE,
                    )
                };
                if rc < 0 && std::io::Error::last_os_error().raw_os_error() == Some(18) {
                    Err(Error::new(
                        ErrorKind::Unsupported,
                        "cross-filesystem move is not yet implemented; source was preserved (use copy, verify, then delete)",
                    ))
                } else {
                    syscall_result(rc, "move/rename without overwrite")
                }
            }
            _ => unreachable!(),
        });
        match result {
            Ok(()) => {
                progress.finish_file();
                return Ok(Outcome::Done(Location::local(
                    dest_path.with_file_name(candidate),
                )));
            }
            Err(e) if e.kind == ErrorKind::Conflict => match op.conflict {
                ConflictPolicy::Fail => return Err(e),
                ConflictPolicy::Skip => return Ok(Outcome::Skipped(destination.clone())),
                ConflictPolicy::KeepBoth => continue,
            },
            Err(e) => return Err(e),
        }
    }
    Err(Error::new(
        ErrorKind::Limit,
        "more than 1000 filename conflicts",
    ))
}

#[cfg(test)]
mod fault_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static ID: AtomicU64 = AtomicU64::new(0);
    struct Lab(PathBuf);
    impl Lab {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "aurex-fault-{}-{}",
                std::process::id(),
                ID.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            std::fs::write(path.join("source"), vec![b'a'; 400_000]).unwrap();
            Self(path)
        }
        fn run(
            &self,
            cancel: &Cancellation,
            hook: &mut impl FnMut(CopyCheckpoint) -> Result<()>,
        ) -> Result<()> {
            let dir = grant_root(&self.0).unwrap();
            copy_with_checkpoints(
                &dir,
                OsStr::new("source"),
                &dir,
                OsStr::new("destination"),
                cancel,
                &Progress::default(),
                hook,
            )
            .map_err(|error| annotate_copy_error(error, PathBuf::from("destination")))
        }
        fn uncommitted(&self) {
            assert!(!self.0.join("destination").exists());
            assert_eq!(
                std::fs::read_dir(&self.0).unwrap().count(),
                1,
                "anonymous staging must leave no entry"
            );
        }
    }
    impl Drop for Lab {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn disk_full() -> Error {
        Error::io(
            "injected disk exhaustion",
            std::io::Error::from_raw_os_error(28),
        )
    }
    #[test]
    fn mid_stream_cancel_removes_staging_and_preserves_source() {
        let lab = Lab::new();
        let cancel = Cancellation::default();
        let error = lab
            .run(&cancel, &mut |stage| {
                if stage == CopyCheckpoint::ChunkWritten {
                    cancel.cancel();
                }
                Ok(())
            })
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Cancelled);
        assert!(error.committed.is_none());
        lab.uncommitted();
        assert_eq!(
            std::fs::metadata(lab.0.join("source")).unwrap().len(),
            400_000
        );
    }
    #[test]
    fn changed_source_is_not_published() {
        let lab = Lab::new();
        let error = lab
            .run(&Cancellation::default(), &mut |stage| {
                if stage == CopyCheckpoint::BeforeVerify {
                    std::fs::write(lab.0.join("source"), vec![b'b'; 400_000]).unwrap();
                }
                Ok(())
            })
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Changed);
        assert!(error.committed.is_none());
        lab.uncommitted();
        assert_eq!(
            std::fs::read(lab.0.join("source")).unwrap(),
            vec![b'b'; 400_000]
        );
    }
    #[test]
    fn injected_disk_full_and_file_sync_errors_leave_no_destination() {
        for failure in [CopyCheckpoint::ChunkWritten, CopyCheckpoint::FileSync] {
            let lab = Lab::new();
            let error = lab
                .run(&Cancellation::default(), &mut |stage| {
                    if stage == failure {
                        Err(disk_full())
                    } else {
                        Ok(())
                    }
                })
                .unwrap_err();
            assert_eq!(error.kind, ErrorKind::Io);
            assert!(error.committed.is_none());
            lab.uncommitted();
            assert_eq!(
                std::fs::read(lab.0.join("source")).unwrap(),
                vec![b'a'; 400_000]
            );
        }
    }
    #[test]
    fn last_precommit_cancel_is_not_a_partial_commit() {
        let lab = Lab::new();
        let cancel = Cancellation::default();
        let error = lab
            .run(&cancel, &mut |stage| {
                if stage == CopyCheckpoint::BeforePublish {
                    cancel.cancel();
                }
                Ok(())
            })
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Cancelled);
        assert!(error.committed.is_none());
        lab.uncommitted();
    }
    #[test]
    fn postcommit_sync_failure_reports_published_destination() {
        let lab = Lab::new();
        let error = lab
            .run(&Cancellation::default(), &mut |stage| {
                if stage == CopyCheckpoint::DirectorySync {
                    Err(disk_full())
                } else {
                    Ok(())
                }
            })
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Durability);
        assert_eq!(error.committed, Some(Location::local("destination")));
        assert_eq!(
            std::fs::read(lab.0.join("destination")).unwrap(),
            vec![b'a'; 400_000]
        );
        assert!(error.to_string().contains("inspect before retry"));
        assert_eq!(error.source.as_ref().unwrap().raw_os_error(), Some(28));
    }
    #[test]
    fn cancellation_after_publication_does_not_roll_back_completed_copy() {
        let lab = Lab::new();
        let cancel = Cancellation::default();
        lab.run(&cancel, &mut |stage| {
            if stage == CopyCheckpoint::DirectorySync {
                cancel.cancel();
            }
            Ok(())
        })
        .unwrap();
        assert!(cancel.is_cancelled());
        assert_eq!(
            std::fs::read(lab.0.join("destination")).unwrap(),
            vec![b'a'; 400_000]
        );
    }
}
