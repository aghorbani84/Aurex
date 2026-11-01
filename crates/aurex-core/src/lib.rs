//! Provider-neutral identities, capabilities and operation contracts.
use std::{fmt, io, path::PathBuf, time::SystemTime};

/// Native paths are never converted to UTF-8 for filesystem operations.
/// Local paths are relative to the provider's explicitly granted root.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Location {
    Local(PathBuf),
    DocumentUri(String),
    Archive {
        archive: Box<Location>,
        entry: String,
    },
    Remote {
        scheme: String,
        authority: String,
        path: String,
    },
}
impl Location {
    pub fn local(path: impl Into<PathBuf>) -> Self {
        Self::Local(path.into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capability {
    List,
    Read,
    CreateDirectory,
    CopyFile,
    Move,
    Rename,
    DeleteEmpty,
    PreviewText,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
    Symlink,
    Special,
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub location: Location,
    pub name: std::ffi::OsString,
    pub kind: EntryKind,
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
    pub readonly: bool,
}

#[derive(Clone, Debug)]
pub struct ListOptions {
    pub descending: bool,
    pub show_hidden: bool,
    pub filter: String,
    pub offset: usize,
    pub limit: usize,
}
impl Default for ListOptions {
    fn default() -> Self {
        Self {
            descending: false,
            show_hidden: false,
            filter: String::new(),
            offset: 0,
            limit: 500,
        }
    }
}
#[derive(Clone, Debug)]
pub struct DirectoryPage {
    pub entries: Vec<Entry>,
    pub total: usize,
    pub next_offset: Option<usize>,
    /// Raced/disappeared entries are reported rather than treated as empty metadata.
    pub warnings: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConflictPolicy {
    Fail,
    Skip,
    KeepBoth,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationKind {
    Copy,
    Move,
    Rename,
    DeleteEmpty,
    CreateDirectory,
}
#[derive(Clone, Debug)]
pub struct Operation {
    pub kind: OperationKind,
    pub source: Location,
    pub destination: Option<Location>,
    pub conflict: ConflictPolicy,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Done(Location),
    Skipped(Location),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Io,
    Permission,
    InvalidLocation,
    Unsupported,
    Conflict,
    Cancelled,
    Limit,
    Changed,
    /// Data was published, but crash durability could not be confirmed. Do not retry blindly.
    Durability,
}
#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
    pub source: Option<io::Error>,
    /// Destination already exists despite this error; earlier successful work is not rolled back.
    pub committed: Option<Location>,
}
impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            source: None,
            committed: None,
        }
    }
    /// Attach a published destination to an error without losing the original OS cause.
    pub fn after_commit(mut self, destination: Location) -> Self {
        self.kind = ErrorKind::Durability;
        self.committed = Some(destination);
        self
    }
    pub fn io(context: &str, error: io::Error) -> Self {
        let kind = match error.kind() {
            io::ErrorKind::PermissionDenied => ErrorKind::Permission,
            io::ErrorKind::AlreadyExists => ErrorKind::Conflict,
            io::ErrorKind::Unsupported => ErrorKind::Unsupported,
            _ => ErrorKind::Io,
        };
        Self {
            kind,
            message: format!("{context}: {error}"),
            source: Some(error),
            committed: None,
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)?;
        if let Some(destination) = &self.committed {
            write!(
                f,
                "; committed destination: {destination:?}; inspect before retry"
            )?;
        }
        Ok(())
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e as _)
    }
}
pub type Result<T> = std::result::Result<T, Error>;

pub mod navigation;

pub mod documents;
