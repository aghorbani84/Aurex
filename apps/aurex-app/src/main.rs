mod app;
use aurex_fs::LocalProvider;
use aurex_tasks::Scheduler;
use dioxus::prelude::*;

#[derive(Clone)]
struct AppContext {
    provider: LocalProvider,
}
fn main() {
    let mut args = std::env::args_os().skip(1);
    let Some(root) = args.next() else {
        eprintln!(
            "Usage: aurex-app ROOT [--write]\nThe selected root is the only granted filesystem scope. Default: read-only."
        );
        std::process::exit(2);
    };
    let writable = match args.next() {
        None => false,
        Some(arg) if arg == "--write" && args.next().is_none() => true,
        _ => {
            eprintln!("Expected only ROOT [--write]");
            std::process::exit(2);
        }
    };
    let provider = LocalProvider::new(root, writable, Scheduler::new(3, 32)).unwrap_or_else(|e| {
        eprintln!("Cannot grant root: {e}");
        std::process::exit(1);
    });
    LaunchBuilder::desktop()
        .with_context(AppContext { provider })
        .launch(app::App);
}
