//! Root-scoped local provider. Linux operations use directory descriptors and never follow symlinks.
//! Other providers must supply their own semantics and capability discovery.
use aurex_core::*;
use aurex_tasks::{Cancellation, Progress, Scheduler, Task};
use std::path::{Component, Path, PathBuf};
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::sync::Arc;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod linux;

pub trait FilesystemProvider: Send + Sync {
    fn capabilities(&self) -> Vec<Capability>;
    fn list(&self, location: Location, options: ListOptions)
    -> Result<Task<Result<DirectoryPage>>>;
    fn execute(
        &self,
        operation: Operation,
        cancel: Cancellation,
        progress: Progress,
    ) -> Result<Task<Result<Outcome>>>;
    fn preview_text(&self, location: Location) -> Result<Task<Result<String>>>;
}

#[derive(Clone)]
pub struct LocalProvider {
    root: PathBuf,
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    directory: Arc<std::fs::File>,
    writable: bool,
    scheduler: Scheduler,
}
impl LocalProvider {
    pub fn new(root: impl AsRef<Path>, writable: bool, scheduler: Scheduler) -> Result<Self> {
        let root = std::fs::canonicalize(root).map_err(|e| Error::io("grant root", e))?;
        if !root.is_dir() {
            return Err(Error::new(
                ErrorKind::InvalidLocation,
                "root must be a directory",
            ));
        }
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            let directory = Arc::new(linux::grant_root(&root)?);
            Ok(Self {
                root,
                directory,
                writable,
                scheduler,
            })
        }
        #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
        {
            let _ = (root, writable, scheduler);
            Err(Error::new(
                ErrorKind::Unsupported,
                "native provider is currently verified only on Linux",
            ))
        }
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    fn schedule<T: Send + 'static>(
        &self,
        work: impl FnOnce() -> Result<T> + Send + 'static,
    ) -> Result<Task<Result<T>>> {
        self.scheduler
            .submit(work)
            .map_err(|e| Error::new(ErrorKind::Limit, e.to_string()))
    }
}
impl FilesystemProvider for LocalProvider {
    fn capabilities(&self) -> Vec<Capability> {
        let mut caps = vec![Capability::List, Capability::Read, Capability::PreviewText];
        if self.writable {
            caps.extend([
                Capability::CreateDirectory,
                Capability::CopyFile,
                Capability::Move,
                Capability::Rename,
                Capability::DeleteEmpty,
            ]);
        }
        caps
    }
    fn list(
        &self,
        location: Location,
        options: ListOptions,
    ) -> Result<Task<Result<DirectoryPage>>> {
        let path = local_path(&location)?;
        if options.filter.len() > 1024 {
            return Err(Error::new(
                ErrorKind::Limit,
                "filename filter exceeds 1024 bytes",
            ));
        }
        if options.limit == 0 || options.limit > 2000 {
            return Err(Error::new(ErrorKind::Limit, "page size must be 1..=2000"));
        }
        let provider = self.clone();
        self.schedule(move || {
            #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
            {
                linux::list(&provider.directory, &path, options)
            }
            #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
            {
                let _ = (provider, path, options);
                Err(unsupported())
            }
        })
    }
    fn execute(
        &self,
        operation: Operation,
        cancel: Cancellation,
        progress: Progress,
    ) -> Result<Task<Result<Outcome>>> {
        if !self.writable {
            return Err(Error::new(
                ErrorKind::Permission,
                "provider was granted read-only access",
            ));
        }
        local_path(&operation.source)?;
        if let Some(location) = &operation.destination {
            local_path(location)?;
        }
        let provider = self.clone();
        self.schedule(move || {
            if cancel.is_cancelled() {
                return Err(Error::new(ErrorKind::Cancelled, "operation cancelled"));
            }
            #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
            {
                linux::execute(&provider.directory, operation, &cancel, &progress)
            }
            #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
            {
                let _ = (provider, operation, cancel, progress);
                Err(unsupported())
            }
        })
    }
    fn preview_text(&self, location: Location) -> Result<Task<Result<String>>> {
        let path = local_path(&location)?;
        let provider = self.clone();
        self.schedule(move || {
            #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
            {
                linux::preview(&provider.directory, &path)
            }
            #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
            {
                let _ = (provider, path);
                Err(unsupported())
            }
        })
    }
}
#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
fn unsupported() -> Error {
    Error::new(
        ErrorKind::Unsupported,
        "provider is not implemented on this platform",
    )
}

fn local_path(location: &Location) -> Result<PathBuf> {
    let Location::Local(path) = location else {
        return Err(Error::new(
            ErrorKind::Unsupported,
            "no provider registered for this location type",
        ));
    };
    if path.as_os_str().len() > 16 * 1024 {
        return Err(Error::new(ErrorKind::Limit, "relative path exceeds 16 KiB"));
    }
    if path
        .components()
        .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Err(Error::new(
            ErrorKind::InvalidLocation,
            "use a root-relative path without parent components",
        ));
    }
    Ok(path
        .components()
        .filter(|c| !matches!(c, Component::CurDir))
        .collect())
}
