use aurex_core::*;
use aurex_fs::{FilesystemProvider, LocalProvider};
use aurex_tasks::{Cancellation, Progress, Scheduler};
use std::{env, ffi::OsString, path::PathBuf};

fn main() {
    if let Err(e) = run() {
        eprintln!("aurex: {e}");
        std::process::exit(1);
    }
}
fn usage() -> &'static str {
    "AUREX • root-scoped local file tools\n\nUsage: aurex-cli ROOT [--write] COMMAND [ARGS]\n\n  list [PATH] [FILTER]       List directory, including hidden entries\n  cat PATH                   Bounded UTF-8 text preview\n  mkdir PATH                 Create private directory\n  copy SOURCE DEST           Verified regular-file copy; never overwrite\n  move SOURCE DEST           Same-filesystem move; never overwrite\n  rename SOURCE DEST         Rename; never overwrite\n  delete PATH --confirm      PERMANENTLY delete a file or empty directory\n\nPaths are relative to ROOT. Links are never followed. Mutations require --write.\nDirectory copy, recursive deletion and cross-filesystem moves are intentionally unavailable."
}
fn run() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<OsString> = env::args_os().skip(1).collect();
    if args.is_empty() || args[0] == "--help" {
        println!("{}", usage());
        return Ok(());
    }
    let root = args.remove(0);
    let writable = args.first().is_some_and(|x| x == "--write");
    if writable {
        args.remove(0);
    }
    if args.is_empty() {
        return Err(usage().into());
    }
    let command = args.remove(0);
    let provider = LocalProvider::new(root, writable, Scheduler::new(2, 16))?;
    let location = |index: usize| -> std::result::Result<Location, Box<dyn std::error::Error>> {
        Ok(Location::local(PathBuf::from(
            args.get(index).ok_or("missing path argument")?,
        )))
    };
    if command == "list" {
        if args.len() > 2 {
            return Err("list takes at most PATH and FILTER".into());
        }
        let path = args.first().map(PathBuf::from).unwrap_or_default();
        let filter = args
            .get(1)
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut offset = 0;
        loop {
            let page = provider
                .list(
                    Location::local(&path),
                    ListOptions {
                        show_hidden: true,
                        filter: filter.clone(),
                        offset,
                        ..Default::default()
                    },
                )?
                .wait()??;
            for entry in page.entries {
                println!(
                    "{:?}\t{}\t{:?}",
                    entry.kind,
                    entry
                        .size
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "—".into()),
                    entry.name
                );
            }
            for warning in page.warnings {
                eprintln!("warning: {warning}");
            }
            match page.next_offset {
                Some(next) => offset = next,
                None => break,
            }
        }
        return Ok(());
    }
    if command == "cat" {
        if args.len() != 1 {
            return Err("cat takes one PATH".into());
        }
        print!("{}", provider.preview_text(location(0)?)?.wait()??);
        return Ok(());
    }
    let kind = match command.to_str() {
        Some("mkdir") if args.len() == 1 => OperationKind::CreateDirectory,
        Some("copy") if args.len() == 2 => OperationKind::Copy,
        Some("move") if args.len() == 2 => OperationKind::Move,
        Some("rename") if args.len() == 2 => OperationKind::Rename,
        Some("delete") if args.len() == 2 && args[1] == "--confirm" => OperationKind::DeleteEmpty,
        _ => return Err(usage().into()),
    };
    let progress = Progress::default();
    let outcome = provider
        .execute(
            Operation {
                kind,
                source: location(0)?,
                destination: if matches!(
                    kind,
                    OperationKind::Copy | OperationKind::Move | OperationKind::Rename
                ) {
                    Some(location(1)?)
                } else {
                    None
                },
                conflict: ConflictPolicy::Fail,
            },
            Cancellation::default(),
            progress.clone(),
        )?
        .wait()??;
    println!(
        "{outcome:?} • {} bytes copied • {} item completed",
        progress.snapshot().0,
        progress.snapshot().1
    );
    Ok(())
}
