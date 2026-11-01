//! Explicit startup grant: opening the installed app never grants the home directory.
use crate::{AppContext, app};
use aurex_fs::LocalProvider;
use aurex_tasks::Scheduler;
use dioxus::prelude::*;

#[derive(Clone)]
pub(crate) struct Startup(pub(crate) Option<AppContext>);

#[component]
pub fn Shell() -> Element {
    let startup = use_context::<Startup>();
    let mut selected = use_signal(|| startup.0);
    let mut writable = use_signal(|| false);
    let mut message = use_signal(|| {
        "Select one folder. Access is read-only unless you explicitly allow changes.".to_owned()
    });
    let mut busy = use_signal(|| false);
    if selected.read().is_some() {
        return rsx! { Workspace { selected } };
    }
    rsx! {
        style { {include_str!("../assets/app.css")} }
        div { class: "app", main { class: "startup-grant",
            h1 { "Welcome to AUREX" }
            p { "Your files stay on this device. Choose a folder to open a root-scoped workspace." }
            p { role: "status", "{message}" }
            label { input { r#type: "checkbox", checked: writable(), disabled: busy(), onchange: move |event| writable.set(event.checked()) } "Allow file changes in this folder" }
            p { "Deletion is permanent. Existing files are never silently replaced. Only use disposable files while release security gates remain open." }
            button { disabled: busy(), onclick: move |_| {
                busy.set(true);
                let writable = writable();
                spawn(async move {
                    let folder = rfd::AsyncFileDialog::new().set_title("Choose an AUREX workspace").pick_folder().await;
                    if let Some(folder) = folder {
                        let scheduler = Scheduler::new(3,32);
                        let worker = scheduler.clone();
                        match scheduler.submit(move || LocalProvider::new(folder.path(),writable,worker)) {
                            Ok(task) => match task.await {
                                Ok(Ok(provider)) => selected.set(Some(AppContext { provider, #[cfg(feature="gui-smoke")] smoke: false })),
                                Ok(Err(error)) => message.set(error.to_string()),
                                Err(error) => message.set(error.to_string()),
                            },
                            Err(error) => message.set(error.to_string()),
                        }
                    } else { message.set("Folder selection cancelled. No access was granted.".into()); }
                    busy.set(false);
                });
            }, if busy() { "Opening folder…" } else { "Choose a folder" } }
        } }
    }
}

#[component]
fn Workspace(selected: Signal<Option<AppContext>>) -> Element {
    let context = selected
        .read()
        .as_ref()
        .expect("workspace requires an explicit grant")
        .clone();
    use_context_provider(|| context);
    rsx! { app::App {} }
}
