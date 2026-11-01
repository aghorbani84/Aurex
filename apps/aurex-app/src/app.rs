use crate::AppContext;
use aurex_core::{
    Capability, ConflictPolicy, EntryKind, ListOptions, Location, Operation, OperationKind,
    navigation::PaneState,
};
use aurex_fs::FilesystemProvider;
use aurex_tasks::{Cancellation, Progress};
use dioxus::prelude::*;
use std::path::{Component, Path, PathBuf};

const CSS: &str = include_str!("../assets/app.css");
#[derive(Clone, Copy)]
struct Explorer {
    left: Signal<PaneState>,
    right: Signal<PaneState>,
    active: Signal<bool>,
    refresh: Signal<u64>,
    busy: Signal<bool>,
    dialog: Signal<Option<OperationKind>>,
    input: Signal<String>,
    message: Signal<String>,
    log: Signal<Vec<String>>,
    preview: Signal<Option<String>>,
    bookmarks: Signal<Vec<PathBuf>>,
    progress: Signal<Progress>,
    cancel: Signal<Cancellation>,
}
impl Explorer {
    fn pane(&self) -> Signal<PaneState> {
        if *self.active.read() {
            self.right
        } else {
            self.left
        }
    }
    fn other(&self) -> Signal<PaneState> {
        if *self.active.read() {
            self.left
        } else {
            self.right
        }
    }
    fn ask(mut self, kind: OperationKind) {
        if *self.busy.read() {
            return;
        }
        let count = self.pane().read().selection.len();
        if kind != OperationKind::CreateDirectory && count == 0 {
            self.message.set("Select at least one item first.".into());
            return;
        }
        if kind == OperationKind::Rename && count != 1 {
            self.message
                .set("Rename requires exactly one selected item.".into());
            return;
        }
        self.input.set(String::new());
        self.dialog.set(Some(kind));
    }
}

#[component]
pub fn App() -> Element {
    let ctx = use_context::<AppContext>();
    let left = use_signal(PaneState::default);
    let right = use_signal(PaneState::default);
    let active = use_signal(|| false);
    let refresh = use_signal(|| 0u64);
    let busy = use_signal(|| false);
    let dialog = use_signal(|| None);
    let input = use_signal(String::new);
    let message =
        use_signal(|| "Ready. Files stay on this device. Links are never followed.".to_string());
    let log = use_signal(Vec::new);
    let preview = use_signal(|| None);
    let bookmarks = use_signal(Vec::new);
    let progress = use_signal(Progress::default);
    let cancel = use_signal(Cancellation::default);
    let mut explorer = Explorer {
        left,
        right,
        active,
        refresh,
        busy,
        dialog,
        input,
        message,
        log,
        preview,
        bookmarks,
        progress,
        cancel,
    };
    use_context_provider(|| explorer.clone());
    let writable = ctx.provider.capabilities().contains(&Capability::CopyFile);
    let root = ctx.provider.root().display().to_string();
    let selected = explorer.pane().read().selection.len();
    let mut light = use_signal(|| false);
    let mut mobile_right = use_signal(|| false);
    let mut byte_count = use_signal(|| 0u64);
    let poll = explorer.clone();
    use_future(move || {
        let poll = poll.clone();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(120)).await;
                let value = poll.progress.read().snapshot().0;
                if *byte_count.peek() != value {
                    byte_count.set(value);
                }
            }
        }
    });
    let keyboard = explorer.clone();
    rsx! {
        style { "{CSS}" }
        div { class: if light() { "app light" } else { "app" },
            onkeydown: move |event| {
                // Do not hijack typing in address/filter/dialog fields. Only function keys are global.
                match event.key() {
                    Key::F5 if writable => keyboard.clone().ask(OperationKind::Copy),
                    Key::F6 if writable => keyboard.clone().ask(OperationKind::Move),
                    Key::F7 if writable => keyboard.clone().ask(OperationKind::CreateDirectory),
                    Key::Escape => { let mut state = keyboard.clone(); state.dialog.set(None); state.preview.set(None); },
                    _ => {}
                }
            },
            header { class: "topbar",
                div { class: "brand", span { class: "brand-mark", "A" } "aurex" span { class: "edition", "WORKSPACE" } }
                div { class: "root-grant", title: "Explicitly granted filesystem root", "{root}" }
                span { class: if writable { "badge write" } else { "badge" }, if writable { "Write enabled" } else { "Read only" } }
                button { class: "icon-button", title: "Switch light or dark theme", "aria-label": "Switch theme", onclick: move |_| light.toggle(), if light() { "◐" } else { "☼" } }
            }
            div { class: "workspace",
                aside { class: "sidebar",
                    div { class: "section-label", "YOUR WORKSPACE" }
                    button { class: "sidebar-link selected", onclick: move |_| explorer.pane().write().navigate(PathBuf::new()), span { "⌂" } "Granted root" }
                    div { class: "section-label bookmark-title", "SESSION BOOKMARKS" }
                    for (index, path) in bookmarks().iter().enumerate() {
                        { let destination = path.clone(); let label = label(path); rsx! {
                            div { class: "bookmark",
                                button { class: "sidebar-link", onclick: move |_| explorer.pane().write().navigate(destination.clone()), "◇ {label}" }
                                button { class: "icon-button", "aria-label": "Remove bookmark", onclick: move |_| { explorer.bookmarks.write().remove(index); }, "×" }
                            }
                        } }
                    }
                    button { class: "sidebar-link subdued", onclick: move |_| {
                        let path = explorer.pane().read().path();
                        if !explorer.bookmarks.read().contains(&path) { explorer.bookmarks.write().push(path); }
                    }, "+ Bookmark location" }
                    div { class: "sidebar-note", span { class: "safe-dot", "●" } "Local by design" p { "Only your granted root is accessible. No uploads. No automatic file execution." } }
                }
                main {
                    div { class: "workspace-title",
                        div { h1 { "A place for everything." } p { "Two workspaces. One clear view." } }
                        div { class: "view-actions",
                            button { disabled: busy(), onclick: move |_| { let l = (explorer.left)().clone(); let r = (explorer.right)().clone(); explorer.left.set(r); explorer.right.set(l); }, title: "Exchange both workspaces", "⇄ Exchange" }
                            button { disabled: busy(), onclick: move |_| { let path = explorer.pane().read().path(); explorer.other().write().navigate(path); }, title: "Set other pane to the active location", "⟳ Sync location" }
                        }
                    }
                    div { class: "mobile-switch",
                        button { onclick: move |_| { mobile_right.set(false); explorer.active.set(false); }, "Left workspace" }
                        button { onclick: move |_| { mobile_right.set(true); explorer.active.set(true); }, "Right workspace" }
                    }
                    div { class: if mobile_right() { "panes mobile-right" } else { "panes" },
                        Pane { state: left, right_side: false }
                        Pane { state: right, right_side: true }
                    }
                    div { class: "actionbar",
                        span { class: "selection-count", "{selected} selected" }
                        for (kind, title, shortcut) in [(OperationKind::Copy, "Copy to other pane", "F5"), (OperationKind::Move, "Move to other pane", "F6"), (OperationKind::Rename, "Rename", ""), (OperationKind::CreateDirectory, "New folder", "F7"), (OperationKind::DeleteEmpty, "Delete permanently", "")] {
                            { let action = explorer.clone(); rsx! {
                                button { class: if kind == OperationKind::DeleteEmpty { "danger-quiet" } else { "" }, disabled: !writable || busy() || (kind != OperationKind::CreateDirectory && selected == 0), onclick: move |_| action.clone().ask(kind), "{title}" kbd { "{shortcut}" } }
                            } }
                        }
                    }
                    div { class: "activity",
                        div { class: "activity-title", span { "SESSION ACTIVITY" } span { "{byte_count} bytes streamed" } }
                        if busy() { div { class: "working", span { class: "spinner" } "Operation running…" button { onclick: move |_| explorer.cancel.read().cancel(), "Cancel remaining work" } } }
                        p { role: "status", "{message}" }
                        for line in log().iter().rev().take(8) { div { class: "log-line", "{line}" } }
                    }
                }
            }
            footer { span { "AUREX  /  LOCAL PROVIDER" } span { "UTF-8 previews · no overwrite · root-scoped access" } }
            if let Some(kind) = dialog() { OperationDialog { kind } }
            if let Some(text) = preview() {
                div { class: "modal-backdrop", div { class: "modal preview-modal", role: "dialog", "aria-modal": "true", "aria-label": "Read-only text preview",
                    div { class: "modal-heading", h2 { "Text preview" } button { autofocus: true, onclick: move |_| explorer.preview.set(None), "Close" } }
                    p { "Read only · UTF-8 · maximum 256 KiB" } pre { "{text}" }
                } }
            }
        }
    }
}
fn label(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Root".into())
}
fn bytes(size: Option<u64>) -> String {
    match size {
        None => "—".into(),
        Some(n) if n >= 1_048_576 => format!("{:.1} MB", n as f64 / 1_048_576.0),
        Some(n) if n >= 1024 => format!("{:.1} KB", n as f64 / 1024.0),
        Some(n) => format!("{n} B"),
    }
}
fn valid_path(input: &str) -> bool {
    !input.contains('\0')
        && Path::new(input)
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
}

#[component]
fn Pane(mut state: Signal<PaneState>, right_side: bool) -> Element {
    let ctx = use_context::<AppContext>();
    let mut explorer = use_context::<Explorer>();
    let provider = ctx.provider.clone();
    let mut address = use_signal(String::new);
    let request = use_memo(move || {
        let snapshot = state.read();
        (
            snapshot.path(),
            snapshot.show_hidden,
            snapshot.filter.clone(),
            snapshot.offset,
            snapshot.descending,
        )
    });
    let listing = use_resource(move || {
        let (path, show_hidden, filter, offset, descending) = request();
        let _version = *explorer.refresh.read();
        let provider = provider.clone();
        async move {
            provider
                .list(
                    Location::local(path),
                    ListOptions {
                        descending,
                        show_hidden,
                        filter,
                        offset,
                        limit: 500,
                    },
                )
                .map_err(|e| e.to_string())?
                .await
                .map_err(|e| e.to_string())?
                .map_err(|e| e.to_string())
        }
    });
    let page = listing.read().clone();
    let snapshot = state.read().clone();
    let current = snapshot.path();
    let is_active = (explorer.active)() == right_side;
    let title = if right_side {
        "RIGHT WORKSPACE"
    } else {
        "LEFT WORKSPACE"
    };
    let entries = page
        .as_ref()
        .and_then(|p| p.as_ref().ok())
        .map(|p| p.entries.clone())
        .unwrap_or_default();
    rsx! {
        section { class: if is_active { "pane active" } else { "pane" }, onfocusin: move |_| explorer.active.set(right_side), onclick: move |_| explorer.active.set(right_side),
            div { class: "pane-title", span { class: "pane-dot" } "{title}" span { class: "pane-path-label", "{label(&current)}" } }
            div { class: "tabs",
                for (index, tab) in snapshot.tabs.iter().enumerate() {
                    button { class: if index == snapshot.active { "tab current" } else { "tab" }, onclick: move |_| state.write().activate(index), "{label(tab.path())}" }
                }
                button { class: "icon-button", disabled: snapshot.tabs.len() >= 32, "aria-label": "Duplicate current tab", onclick: move |_| state.write().new_tab(), "+" }
                button { class: "icon-button", disabled: snapshot.tabs.len() <= 1, "aria-label": "Close current tab", onclick: move |_| state.write().close_tab(), "×" }
            }
            div { class: "navigation",
                button { class: "icon-button", disabled: !snapshot.tab().can_back(), "aria-label": "Back", onclick: move |_| state.write().back(), "←" }
                button { class: "icon-button", disabled: !snapshot.tab().can_forward(), "aria-label": "Forward", onclick: move |_| state.write().forward(), "→" }
                button { class: "icon-button", disabled: current.as_os_str().is_empty(), "aria-label": "Parent directory", onclick: move |_| state.write().parent(), "↑" }
                button { class: "icon-button", "aria-label": "Refresh directory", onclick: move |_| explorer.refresh += 1, "⟳" }
                span { class: "breadcrumb", title: "Path relative to granted root", "Root / {current.display()}" }
            }
            form { class: "address", onsubmit: move |_| {
                let value = address(); if valid_path(&value) { state.write().navigate(PathBuf::from(value)); } else { explorer.message.set("Enter a relative path without parent components.".into()); }
            },
                input { "aria-label": "Root-relative directory path", placeholder: "Go to a path within this root…", value: "{address}", oninput: move |e| address.set(e.value()) }
                button { r#type: "submit", "Go" }
            }
            div { class: "filterbar",
                span { "⌕" }
                input { "aria-label": "Filter directory by filename", placeholder: "Filter files and folders…", value: "{snapshot.filter}", oninput: move |e| { let mut s = state.write(); s.filter = e.value(); s.offset = 0; s.selection.clear(); } }
                label { title: "Show hidden files", input { r#type: "checkbox", checked: snapshot.show_hidden, onchange: move |e| { let mut s = state.write(); s.show_hidden = e.checked(); s.offset = 0; s.selection.clear(); } } "Hidden" }
            }
            div { class: "list-head",
                button { "aria-label": "Select all visible items", onclick: {
                    let visible = entries.clone(); move |_| { for entry in &visible { if let Location::Local(path) = &entry.location { state.write().selection.insert(path.clone()); } } }
                }, "+" }
                button { onclick: move |_| { let descending = state.read().descending; state.write().descending = !descending; }, "Name ↕" }
                span { "Size" } span { "Type" }
            }
            div { class: "file-list", role: "list", "aria-label": title,
                match &page {
                    None => rsx! { div { class: "empty-state", span { class: "spinner" } "Loading directory…" } },
                    Some(Err(error)) => rsx! { div { class: "empty-state error", role: "alert", "Unable to read this location" p { "{error}" } button { onclick: move |_| state.write().back(), "Go back" } } },
                    Some(Ok(loaded)) if loaded.entries.is_empty() => rsx! { div { class: "empty-state", "Nothing here yet." p { "This directory is empty, or no names match your filter." } } },
                    Some(Ok(_)) => rsx! {
                        for entry in entries {
                            { let path = match &entry.location { Location::Local(path) => path.clone(), _ => PathBuf::new() };
                              let selected = snapshot.selection.contains(&path); let checkbox_path = path.clone();
                              let kind = entry.kind; let name = entry.name.to_string_lossy().into_owned(); let location = entry.location.clone();
                              let provider = ctx.provider.clone();
                              rsx! {
                                div { class: if selected { "file-row selected" } else { "file-row" }, role: "listitem",
                                    input { r#type: "checkbox", "aria-label": "Select {name}", checked: selected, onchange: move |_| state.write().toggle(checkbox_path.clone()) }
                                    button { class: "file-name", title: "Open directory or read-only text preview", onclick: move |_| {
                                        if kind == EntryKind::Directory { state.write().navigate(path.clone()); }
                                        else if kind == EntryKind::File {
                                            let provider = provider.clone(); let location = location.clone();
                                            spawn(async move { let result = async { provider.preview_text(location).map_err(|e| e.to_string())?.await.map_err(|e| e.to_string())?.map_err(|e| e.to_string()) }.await;
                                                match result { Ok(text) => explorer.preview.set(Some(text)), Err(error) => explorer.message.set(error) }
                                            });
                                        } else { explorer.message.set("Links and special files are not opened automatically.".into()); }
                                    },
                                        span { class: if kind == EntryKind::Directory { "file-icon folder" } else { "file-icon" }, if kind == EntryKind::Directory { "▰" } else if kind == EntryKind::Symlink { "↗" } else { "▤" } }
                                        span { "{name}" }
                                    }
                                    span { class: "file-size", "{bytes(entry.size)}" }
                                    span { class: "file-type", "{kind:?}" }
                                }
                              }
                            }
                        }
                    },
                }
            }
            div { class: "pane-footer",
                span { "{snapshot.selection.len()} selected" }
                if let Some(Ok(loaded)) = page {
                    span { "{loaded.total} items" }
                    if !loaded.warnings.is_empty() { span { title: "Some metadata could not be read", "⚠ {loaded.warnings.len()} warnings" } }
                    button { disabled: snapshot.offset == 0, onclick: move |_| { let offset = state.read().offset; state.write().offset = offset.saturating_sub(500); state.write().selection.clear(); }, "Previous" }
                    if let Some(next) = loaded.next_offset { button { onclick: move |_| { state.write().offset = next; state.write().selection.clear(); }, "Next" } }
                }
            }
        }
    }
}

#[component]
fn OperationDialog(kind: OperationKind) -> Element {
    let ctx = use_context::<AppContext>();
    let mut explorer = use_context::<Explorer>();
    let pane = explorer.pane();
    let selections: Vec<_> = pane.read().selection.iter().cloned().collect();
    let source_dir = pane.read().path();
    let target_dir = explorer.other().read().path();
    let input = (explorer.input)();
    let named = matches!(kind, OperationKind::Rename | OperationKind::CreateDirectory);
    let valid_name = !input.is_empty()
        && Path::new(&input).components().count() == 1
        && matches!(
            Path::new(&input).components().next(),
            Some(Component::Normal(_))
        )
        && !input.contains('\0');
    let can_commit = !named || valid_name;
    let heading = match kind {
        OperationKind::Copy => "Copy to other workspace",
        OperationKind::Move => "Move to other workspace",
        OperationKind::Rename => "Rename item",
        OperationKind::CreateDirectory => "Create a folder",
        OperationKind::DeleteEmpty => "Permanently delete selected items?",
    };
    let mut keep_both = use_signal(|| false);
    rsx! {
        div { class: "modal-backdrop",
            div { class: "modal", role: "dialog", "aria-modal": "true", "aria-label": heading,
                h2 { "{heading}" }
                if kind == OperationKind::DeleteEmpty { p { class: "error", "This cannot be undone. Nonempty directories will be refused. No recursive deletion." } }
                if matches!(kind, OperationKind::Copy | OperationKind::Move) {
                    p { "Destination: Root / {target_dir.display()}" }
                    p { "Existing items are never overwritten. Directory copies and cross-filesystem moves are not yet supported." }
                    label { input { r#type: "checkbox", checked: keep_both(), onchange: move |e| keep_both.set(e.checked()) } "Keep both on filename conflicts" }
                }
                ul { class: "scope-list", for path in &selections { li { "{path.display()}" } } }
                if named { input { autofocus: true, "aria-label": "New entry name", placeholder: "New name (single component)", value: "{input}", oninput: move |e| explorer.input.set(e.value()) } }
                div { class: "dialog-actions",
                    button { onclick: move |_| explorer.dialog.set(None), "Cancel" }
                    button { class: if kind == OperationKind::DeleteEmpty { "danger" } else { "primary" }, disabled: !can_commit, onclick: move |_| {
                        let provider = ctx.provider.clone();
                        let mut queue = Vec::new();
                        if kind == OperationKind::CreateDirectory {
                            queue.push(Operation { kind, source: Location::local(source_dir.join((explorer.input)())), destination: None, conflict: ConflictPolicy::Fail });
                        } else {
                            for path in &selections {
                                let destination = match kind {
                                    OperationKind::Rename => Some(Location::local(path.with_file_name((explorer.input)()))),
                                    OperationKind::Copy | OperationKind::Move => path.file_name().map(|name| Location::local(target_dir.join(name))),
                                    _ => None,
                                };
                                queue.push(Operation { kind, source: Location::local(path), destination, conflict: if keep_both() { ConflictPolicy::KeepBoth } else { ConflictPolicy::Fail } });
                            }
                        }
                        explorer.busy.set(true); explorer.dialog.set(None);
                        // Each batch owns a fresh cancellation token. Context's token is replaced for Cancel.
                        // Reset is safe because only one operation batch can be active.
                        explorer.cancel.set(Cancellation::default());
                        let cancel = (explorer.cancel)(); let progress = (explorer.progress)();
                        spawn(async move {
                            let mut completed = 0usize; let mut failed = 0usize;
                            for operation in queue {
                                if cancel.is_cancelled() { break; }
                                let description = format!("{:?} {:?}", operation.kind, operation.source);
                                let outcome = async { provider.execute(operation, cancel.clone(), progress.clone()).map_err(|e| e.to_string())?.await.map_err(|e| e.to_string())?.map_err(|e| e.to_string()) }.await;
                                let line = match outcome { Ok(result) => { completed += 1; format!("{description}: {result:?}") }, Err(error) => { failed += 1; format!("{description}: {error}") } };
                                { let mut log = explorer.log.write(); log.push(line); if log.len() > 100 { log.remove(0); } }
                            }
                            explorer.busy.set(false); explorer.refresh += 1;
                            explorer.left.write().selection.clear(); explorer.right.write().selection.clear();
                            explorer.message.set(format!("{completed} finished · {failed} failed{}", if cancel.is_cancelled() { " · remaining work cancelled" } else { "" }));
                        });
                    }, "Confirm" }
                }
            }
        }
    }
}
