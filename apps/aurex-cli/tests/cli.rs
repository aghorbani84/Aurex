#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::os::unix::fs::DirBuilderExt;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
static ID: AtomicU64 = AtomicU64::new(0);
struct Lab(PathBuf);
impl Lab {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "aurex-cli-{}-{}",
            std::process::id(),
            ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        Self(root)
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_aurex-cli"))
            .arg(&self.0)
            .args(args)
            .output()
            .unwrap()
    }
}
impl Drop for Lab {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn cli_roundtrip_preserves_data() {
    let lab = Lab::new();
    fs::write(lab.0.join("source with spaces"), b"hello\n").unwrap();
    for args in [
        vec!["--write", "mkdir", "folder"],
        vec!["--write", "copy", "source with spaces", "folder/copy"],
        vec!["--write", "move", "folder/copy", "moved"],
        vec!["--write", "rename", "moved", "renamed"],
    ] {
        let out = lab.run(&args);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    assert_eq!(lab.run(&["cat", "renamed"]).stdout, b"hello\n");
    assert!(
        lab.run(&["--write", "delete", "renamed", "--confirm"])
            .status
            .success()
    );
    assert_eq!(
        fs::read(lab.0.join("source with spaces")).unwrap(),
        b"hello\n"
    );
}
#[test]
fn destructive_commands_require_grant_and_confirmation() {
    let lab = Lab::new();
    fs::write(lab.0.join("keep"), b"keep").unwrap();
    assert!(!lab.run(&["delete", "keep", "--confirm"]).status.success());
    assert!(!lab.run(&["--write", "delete", "keep"]).status.success());
    assert!(lab.0.join("keep").exists());
    assert!(!lab.run(&["mkdir", "blocked"]).status.success());
    assert!(!lab.0.join("blocked").exists());
}
#[test]
fn cli_conflict_and_traversal_errors_are_nonzero() {
    let lab = Lab::new();
    fs::write(lab.0.join("source"), b"new").unwrap();
    fs::write(lab.0.join("dest"), b"keep").unwrap();
    assert!(
        !lab.run(&["--write", "copy", "source", "dest"])
            .status
            .success()
    );
    assert_eq!(fs::read(lab.0.join("dest")).unwrap(), b"keep");
    assert!(!lab.run(&["list", "../"]).status.success());
}
#[test]
fn crafted_names_are_literal_not_shell_commands() {
    let lab = Lab::new();
    assert!(
        lab.run(&["--write", "mkdir", "$(touch SHOULD_NOT_EXIST)"])
            .status
            .success()
    );
    assert!(lab.0.join("$(touch SHOULD_NOT_EXIST)").is_dir());
    assert!(!lab.0.join("SHOULD_NOT_EXIST").exists());
}
