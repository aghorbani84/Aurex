#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use aurex_core::*;
use aurex_fs::{FilesystemProvider, LocalProvider};
use aurex_tasks::{Cancellation, Progress, Scheduler};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
static ID: AtomicU64 = AtomicU64::new(0);
struct Lab(PathBuf);
impl Lab {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "aurex-test-{}-{}",
            std::process::id(),
            ID.fetch_add(1, Ordering::Relaxed)
        ));
        // create_dir (not create_dir_all) ensures tests never adopt an existing directory.
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self, name: impl AsRef<Path>) -> PathBuf {
        self.0.join(name)
    }
    fn provider(&self, write: bool) -> LocalProvider {
        LocalProvider::new(&self.0, write, Scheduler::new(2, 16)).unwrap()
    }
}
impl Drop for Lab {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn op(kind: OperationKind, source: &str, dest: Option<&str>) -> Operation {
    Operation {
        kind,
        source: Location::local(source),
        destination: dest.map(Location::local),
        conflict: ConflictPolicy::Fail,
    }
}
fn run(provider: &LocalProvider, operation: Operation) -> Result<Outcome> {
    provider
        .execute(operation, Cancellation::default(), Progress::default())?
        .wait()
        .unwrap()
}
fn all(provider: &LocalProvider, show_hidden: bool, filter: &str) -> DirectoryPage {
    provider
        .list(
            Location::local(""),
            ListOptions {
                show_hidden,
                filter: filter.into(),
                ..Default::default()
            },
        )
        .unwrap()
        .wait()
        .unwrap()
        .unwrap()
}
#[test]
fn browse_sort_filter_hidden_metadata_and_unicode() {
    let lab = Lab::new();
    fs::write(lab.path("z.txt"), b"hello").unwrap();
    fs::write(lab.path(".secret"), b"secret").unwrap();
    fs::create_dir(lab.path("a-dir")).unwrap();
    fs::write(lab.path("日本語-🦀.txt"), b"unicode").unwrap();
    let provider = lab.provider(false);
    let page = all(&provider, false, "");
    assert_eq!(page.total, 3);
    assert_eq!(page.entries[0].kind, EntryKind::Directory);
    let page = all(&provider, true, ".TXT");
    assert_eq!(page.total, 2);
    let text = page.entries.iter().find(|e| e.name == "z.txt").unwrap();
    assert_eq!(text.size, Some(5));
    assert!(text.modified.is_some());
    assert_eq!(all(&provider, true, "").total, 4);
}
#[test]
fn paging_is_bounded() {
    let lab = Lab::new();
    for n in 0..7 {
        fs::write(lab.path(format!("{n}")), []).unwrap();
    }
    let provider = lab.provider(false);
    assert_eq!(
        provider
            .list(
                Location::local(""),
                ListOptions {
                    filter: "x".repeat(1025),
                    ..Default::default()
                }
            )
            .err()
            .unwrap()
            .kind,
        ErrorKind::Limit
    );
    let page = provider
        .list(
            Location::local(""),
            ListOptions {
                offset: 2,
                limit: 3,
                ..Default::default()
            },
        )
        .unwrap()
        .wait()
        .unwrap()
        .unwrap();
    assert_eq!(page.entries.len(), 3);
    assert_eq!(page.total, 7);
    assert_eq!(page.next_offset, Some(5));
    assert!(
        provider
            .list(
                Location::local(""),
                ListOptions {
                    limit: 0,
                    ..Default::default()
                }
            )
            .is_err()
    );
}
#[test]
fn complete_file_lifecycle_and_progress() {
    let lab = Lab::new();
    fs::write(lab.path("original"), b"real bytes\n").unwrap();
    let provider = lab.provider(true);
    run(&provider, op(OperationKind::CreateDirectory, "dest", None)).unwrap();
    let progress = Progress::default();
    provider
        .execute(
            op(OperationKind::Copy, "original", Some("dest/copy")),
            Cancellation::default(),
            progress.clone(),
        )
        .unwrap()
        .wait()
        .unwrap()
        .unwrap();
    assert_eq!(fs::read(lab.path("dest/copy")).unwrap(), b"real bytes\n");
    assert_eq!(progress.snapshot(), (11, 1));
    run(
        &provider,
        op(OperationKind::Rename, "dest/copy", Some("dest/renamed")),
    )
    .unwrap();
    run(
        &provider,
        op(OperationKind::Move, "dest/renamed", Some("moved")),
    )
    .unwrap();
    assert!(!lab.path("dest/renamed").exists());
    run(&provider, op(OperationKind::DeleteEmpty, "moved", None)).unwrap();
    run(&provider, op(OperationKind::DeleteEmpty, "dest", None)).unwrap();
    assert_eq!(fs::read(lab.path("original")).unwrap(), b"real bytes\n");
}
#[test]
fn conflicts_never_overwrite() {
    let lab = Lab::new();
    fs::write(lab.path("source"), b"new").unwrap();
    fs::write(lab.path("dest"), b"precious").unwrap();
    let p = lab.provider(true);
    for kind in [
        OperationKind::Copy,
        OperationKind::Move,
        OperationKind::Rename,
    ] {
        assert_eq!(
            run(&p, op(kind, "source", Some("dest"))).unwrap_err().kind,
            ErrorKind::Conflict
        );
        assert_eq!(fs::read(lab.path("dest")).unwrap(), b"precious");
        assert!(lab.path("source").exists());
    }
    let mut operation = op(OperationKind::Copy, "source", Some("dest"));
    operation.conflict = ConflictPolicy::Skip;
    assert!(matches!(run(&p, operation).unwrap(), Outcome::Skipped(_)));
    let mut operation = op(OperationKind::Copy, "source", Some("dest"));
    operation.conflict = ConflictPolicy::KeepBoth;
    assert!(matches!(run(&p, operation).unwrap(), Outcome::Done(_)));
    assert_eq!(fs::read(lab.path("dest (copy 1)")).unwrap(), b"new");
    assert!(!fs::read_dir(&lab.0).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".aurex-copy-")
    }));
}
#[test]
fn cancellation_preserves_source_and_destination() {
    let lab = Lab::new();
    fs::write(lab.path("source"), b"data").unwrap();
    let p = lab.provider(true);
    let cancel = Cancellation::default();
    cancel.cancel();
    let error = p
        .execute(
            op(OperationKind::Copy, "source", Some("dest")),
            cancel,
            Progress::default(),
        )
        .unwrap()
        .wait()
        .unwrap()
        .unwrap_err();
    assert_eq!(error.kind, ErrorKind::Cancelled);
    assert!(!lab.path("dest").exists());
    assert!(lab.path("source").exists());
}
#[test]
fn readonly_grants_and_unregistered_providers_are_explicit() {
    let lab = Lab::new();
    let p = lab.provider(false);
    assert!(!p.capabilities().contains(&Capability::CopyFile));
    assert_eq!(
        p.execute(
            op(OperationKind::CreateDirectory, "x", None),
            Cancellation::default(),
            Progress::default()
        )
        .err()
        .unwrap()
        .kind,
        ErrorKind::Permission
    );
    assert_eq!(
        p.list(
            Location::DocumentUri("content://example/tree".into()),
            ListOptions::default()
        )
        .err()
        .unwrap()
        .kind,
        ErrorKind::Unsupported
    );
}
#[test]
fn traversal_absolute_paths_nul_and_root_mutations_rejected() {
    let lab = Lab::new();
    let p = lab.provider(true);
    for path in ["../escape", "/tmp/escape", "a/../../escape"] {
        assert_eq!(
            p.list(Location::local(path), ListOptions::default())
                .err()
                .unwrap()
                .kind,
            ErrorKind::InvalidLocation
        );
    }
    for path in ["", ".", "bad\0name"] {
        assert!(run(&p, op(OperationKind::CreateDirectory, path, None)).is_err());
    }
    assert_eq!(
        p.list(Location::local("x".repeat(16_385)), ListOptions::default())
            .err()
            .unwrap()
            .kind,
        ErrorKind::Limit
    );
    assert!(lab.0.is_dir());
}
#[test]
fn symlink_escape_and_broken_links_never_followed() {
    use std::os::unix::fs::symlink;
    let lab = Lab::new();
    let outside = Lab::new();
    fs::write(outside.path("private"), b"private").unwrap();
    symlink(&outside.0, lab.path("escape")).unwrap();
    symlink(outside.path("private"), lab.path("file-link")).unwrap();
    symlink("missing", lab.path("broken")).unwrap();
    let p = lab.provider(true);
    fs::write(lab.path("safe"), b"safe").unwrap();
    assert_eq!(
        run(&p, op(OperationKind::Copy, "safe", Some("file-link")))
            .unwrap_err()
            .kind,
        ErrorKind::Conflict
    );
    assert!(
        p.preview_text(Location::local("file-link"))
            .unwrap()
            .wait()
            .unwrap()
            .is_err()
    );
    assert!(
        p.list(Location::local("escape"), ListOptions::default())
            .unwrap()
            .wait()
            .unwrap()
            .is_err()
    );
    assert!(run(&p, op(OperationKind::Copy, "file-link", Some("copied"))).is_err());
    assert!(run(&p, op(OperationKind::Copy, "file-link", Some("escape/out"))).is_err());
    assert!(run(&p, op(OperationKind::CreateDirectory, "escape/out", None)).is_err());
    assert_eq!(
        all(&p, true, "")
            .entries
            .iter()
            .filter(|e| e.kind == EntryKind::Symlink)
            .count(),
        3
    );
    run(&p, op(OperationKind::DeleteEmpty, "file-link", None)).unwrap();
    assert_eq!(fs::read(outside.path("private")).unwrap(), b"private");
    assert!(!outside.path("out").exists());
}
#[test]
fn directory_copy_and_recursive_delete_are_not_pretended() {
    let lab = Lab::new();
    fs::create_dir(lab.path("folder")).unwrap();
    fs::write(lab.path("folder/child"), b"keep").unwrap();
    let p = lab.provider(true);
    assert_eq!(
        run(&p, op(OperationKind::Copy, "folder", Some("copy")))
            .unwrap_err()
            .kind,
        ErrorKind::Unsupported
    );
    assert!(run(&p, op(OperationKind::DeleteEmpty, "folder", None)).is_err());
    assert!(lab.path("folder/child").exists());
}
#[test]
fn bounded_text_preview_rejects_binary_and_oversized_inputs() {
    let lab = Lab::new();
    fs::write(lab.path("text"), "hello\n日本語").unwrap();
    fs::write(lab.path("binary"), b"a\0b").unwrap();
    fs::write(lab.path("large"), vec![b'x'; 262145]).unwrap();
    let p = lab.provider(false);
    assert_eq!(
        p.preview_text(Location::local("text"))
            .unwrap()
            .wait()
            .unwrap()
            .unwrap(),
        "hello\n日本語"
    );
    assert_eq!(
        p.preview_text(Location::local("binary"))
            .unwrap()
            .wait()
            .unwrap()
            .unwrap_err()
            .kind,
        ErrorKind::Unsupported
    );
    assert_eq!(
        p.preview_text(Location::local("large"))
            .unwrap()
            .wait()
            .unwrap()
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
}
#[test]
fn streaming_copy_handles_multiple_buffers_and_empty_files() {
    let lab = Lab::new();
    let p = lab.provider(true);
    let bytes: Vec<u8> = (0..1_000_003).map(|i| (i % 251) as u8).collect();
    fs::write(lab.path("large"), &bytes).unwrap();
    fs::write(lab.path("empty"), []).unwrap();
    for name in ["large", "empty"] {
        run(
            &p,
            op(OperationKind::Copy, name, Some(&format!("{name}-copy"))),
        )
        .unwrap();
    }
    assert_eq!(fs::read(lab.path("large-copy")).unwrap(), bytes);
    assert_eq!(fs::metadata(lab.path("empty-copy")).unwrap().len(), 0);
}
#[test]
fn non_utf8_names_keep_native_identity() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let lab = Lab::new();
    let name = OsString::from_vec(vec![b'f', 0xff]);
    fs::write(lab.path(&name), b"native").unwrap();
    let p = lab.provider(false);
    let page = all(&p, true, "");
    assert_eq!(page.entries[0].name, name);
    assert_eq!(
        p.preview_text(page.entries[0].location.clone())
            .unwrap()
            .wait()
            .unwrap()
            .unwrap(),
        "native"
    );
}
#[test]
fn missing_source_does_not_leave_staging_files() {
    let lab = Lab::new();
    let p = lab.provider(true);
    assert!(run(&p, op(OperationKind::Copy, "absent", Some("dest"))).is_err());
    assert_eq!(all(&p, true, "").total, 0);
}
#[test]
fn file_modes_are_private_not_inherited_executable_or_setuid() {
    use std::os::unix::fs::PermissionsExt;
    let lab = Lab::new();
    fs::write(lab.path("src"), b"x").unwrap();
    fs::set_permissions(lab.path("src"), fs::Permissions::from_mode(0o755)).unwrap();
    run(
        &lab.provider(true),
        op(OperationKind::Copy, "src", Some("out")),
    )
    .unwrap();
    assert_eq!(
        fs::metadata(lab.path("out")).unwrap().permissions().mode() & 0o7777,
        0o600
    );
}

#[test]
fn descending_sort_happens_before_pagination() {
    let lab = Lab::new();
    for name in ["a", "b", "c", "d"] {
        fs::write(lab.path(name), []).unwrap();
    }
    let page = lab
        .provider(false)
        .list(
            Location::local(""),
            ListOptions {
                descending: true,
                limit: 2,
                ..Default::default()
            },
        )
        .unwrap()
        .wait()
        .unwrap()
        .unwrap();
    assert_eq!(
        page.entries
            .iter()
            .map(|e| e.name.to_str().unwrap())
            .collect::<Vec<_>>(),
        ["d", "c"]
    );
}
#[test]
fn simultaneous_copies_have_exactly_one_winner() {
    let lab = Lab::new();
    fs::write(lab.path("a"), vec![b'a'; 300_000]).unwrap();
    fs::write(lab.path("b"), vec![b'b'; 300_000]).unwrap();
    let p = lab.provider(true);
    let a = p
        .execute(
            op(OperationKind::Copy, "a", Some("out")),
            Cancellation::default(),
            Progress::default(),
        )
        .unwrap();
    let b = p
        .execute(
            op(OperationKind::Copy, "b", Some("out")),
            Cancellation::default(),
            Progress::default(),
        )
        .unwrap();
    let (a, b) = (a.wait().unwrap(), b.wait().unwrap());
    assert_ne!(a.is_ok(), b.is_ok());
    let expected = if a.is_ok() { b'a' } else { b'b' };
    assert_eq!(fs::read(lab.path("out")).unwrap(), vec![expected; 300_000]);
}
#[test]
fn fifo_is_rejected_without_blocking() {
    let lab = Lab::new();
    assert!(
        std::process::Command::new("mkfifo")
            .arg(lab.path("pipe"))
            .status()
            .unwrap()
            .success()
    );
    let p = lab.provider(true);
    assert_eq!(
        p.preview_text(Location::local("pipe"))
            .unwrap()
            .wait()
            .unwrap()
            .unwrap_err()
            .kind,
        ErrorKind::Unsupported
    );
    assert_eq!(
        run(&p, op(OperationKind::Copy, "pipe", Some("copy")))
            .unwrap_err()
            .kind,
        ErrorKind::Unsupported
    );
}
#[test]
fn os_permissions_remain_authoritative() {
    use std::os::unix::fs::PermissionsExt;
    let lab = Lab::new();
    fs::write(lab.path("private"), b"secret").unwrap();
    fs::set_permissions(lab.path("private"), fs::Permissions::from_mode(0o0)).unwrap();
    let p = lab.provider(false);
    let result = p
        .preview_text(Location::local("private"))
        .unwrap()
        .wait()
        .unwrap();
    // Privileged runners can read mode-000 files. Do not report that as permission enforcement.
    let uid = std::process::Command::new("id").arg("-u").output().unwrap();
    assert!(uid.status.success());
    if String::from_utf8_lossy(&uid.stdout).trim() != "0" {
        assert_eq!(result.unwrap_err().kind, ErrorKind::Permission);
    }
}
#[test]
fn root_handle_remains_anchored_after_path_is_renamed() {
    let lab = Lab::new();
    fs::create_dir(lab.path("grant")).unwrap();
    fs::write(lab.path("grant/inside"), b"original").unwrap();
    let p = LocalProvider::new(lab.path("grant"), false, Scheduler::new(1, 2)).unwrap();
    fs::rename(lab.path("grant"), lab.path("old-grant")).unwrap();
    fs::create_dir(lab.path("grant")).unwrap();
    fs::write(lab.path("grant/inside"), b"replacement").unwrap();
    assert_eq!(
        p.preview_text(Location::local("inside"))
            .unwrap()
            .wait()
            .unwrap()
            .unwrap(),
        "original"
    );
}
