#[cfg(not(target_os = "android"))]
mod app;
#[cfg(not(target_os = "android"))]
mod desktop_shell;
#[cfg(target_os = "android")]
mod mobile;
#[cfg(feature = "gui-smoke")]
mod smoke;
#[cfg(not(target_os = "android"))]
use aurex_fs::LocalProvider;
#[cfg(not(target_os = "android"))]
use aurex_tasks::Scheduler;
#[cfg(not(target_os = "android"))]
use dioxus::prelude::*;

#[cfg(not(target_os = "android"))]
#[derive(Clone)]
pub(crate) struct AppContext {
    provider: LocalProvider,
    #[cfg(feature = "gui-smoke")]
    smoke: bool,
}
#[cfg(not(target_os = "android"))]
fn main() {
    let mut args = std::env::args_os().skip(1);
    let root = args.next();
    let Some(root) = root else {
        LaunchBuilder::desktop()
            .with_cfg(
                dioxus::desktop::Config::new()
                    .with_window(dioxus::desktop::WindowBuilder::new().with_title("AUREX")),
            )
            .with_context(desktop_shell::Startup(None))
            .launch(desktop_shell::Shell);
        return;
    };
    #[cfg(feature = "gui-smoke")]
    let mut smoke = false;
    let writable = match args.next() {
        None => false,
        Some(arg) if arg == "--write" && args.next().is_none() => true,
        #[cfg(feature = "gui-smoke")]
        Some(arg) if arg == "--smoke-test" && args.next().is_none() => {
            smoke = true;
            false
        }
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
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(dioxus::desktop::WindowBuilder::new().with_title("AUREX")),
        )
        .with_context(desktop_shell::Startup(Some(AppContext {
            provider,
            #[cfg(feature = "gui-smoke")]
            smoke,
        })))
        .launch(desktop_shell::Shell);
}

#[cfg(target_os = "android")]
fn main() {
    dioxus::launch(mobile::App);
}
