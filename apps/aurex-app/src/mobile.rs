//! SAF-backed read-only Android UI. Content identities never enter LocalProvider.
use aurex_core::documents::{DocumentRequest, DocumentSession};
use aurex_tasks::Scheduler;
use dioxus::prelude::*;
use serde_json::Value;

#[manganis::ffi("android")]
extern "Kotlin" {
    pub type StoragePlugin;

}

/// Resolve through the current Activity's class loader, not FindClass on an
/// attached worker (which only sees the bootstrap loader). No local JNI refs escape.
fn plugin() -> Result<StoragePlugin, String> {
    use jni::{
        JavaVM,
        objects::{JObject, JString, JValue},
    };
    let context = ndk_context::android_context();
    // SAFETY: Dioxus initializes ndk-context from the live Android Activity before main.
    // JavaVM is a borrowed VM handle; JavaVM does not destroy the process VM on drop.
    let vm = unsafe { JavaVM::from_raw(context.vm().cast()) }.map_err(|e| e.to_string())?;
    let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
    // SAFETY: ndk-context owns the live Activity global reference. JObject is only
    // borrowed in this frame; we neither delete nor retain its raw reference.
    let activity = unsafe { JObject::from_raw(context.context().cast()) };
    let result = env.with_local_frame(32, |env| -> jni::errors::Result<_> {
        let loader = env
            .call_method(
                &activity,
                "getClassLoader",
                "()Ljava/lang/ClassLoader;",
                &[],
            )?
            .l()?;
        let name: JString = env.new_string("io.github.aghorbani84.aurex.storage.StoragePlugin")?;
        let class = env
            .call_method(
                loader,
                "loadClass",
                "(Ljava/lang/String;)Ljava/lang/Class;",
                &[JValue::Object(name.as_ref())],
            )?
            .l()?;
        let instance = env.new_object(
            jni::objects::JClass::from(class),
            "(Landroid/app/Activity;)V",
            &[JValue::Object(&activity)],
        )?;
        env.new_global_ref(instance)
    });
    if env.exception_check().unwrap_or(true) {
        let _ = env.exception_clear();
    }
    result
        .map(StoragePlugin::from_global_ref)
        .map_err(|e| format!("Android storage bridge: {e}"))
}

// Generated metadata bundles the native Gradle project. Invoke on our own attached
// frame to avoid the upstream helper's process-global cached Activity reference.
fn invoke(plugin: &StoragePlugin, method: Call, argument: &str) -> Result<String, String> {
    use jni::objects::{JString, JValue};
    let context = ndk_context::android_context();
    // SAFETY: borrowed VM initialized by Dioxus; the attach guard owns this thread attachment.
    let vm = unsafe { jni::JavaVM::from_raw(context.vm().cast()) }.map_err(|e| e.to_string())?;
    let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
    let result = env.with_local_frame(16, |env| -> jni::errors::Result<String> {
        let (name, has_argument) = match method {
            Call::State => ("stateJson", false),
            Call::Choose => ("chooseJson", false),
            Call::Forget => ("forgetJson", false),
            Call::List => ("listJson", true),
            Call::Preview => ("previewJson", true),
        };
        let argument = env.new_string(argument)?;
        let arguments = [JValue::Object(argument.as_ref())];
        let result = env
            .call_method(
                plugin.as_obj(),
                name,
                if has_argument {
                    "(Ljava/lang/String;)Ljava/lang/String;"
                } else {
                    "()Ljava/lang/String;"
                },
                if has_argument { &arguments } else { &[] },
            )?
            .l()?;
        let result = JString::from(result);
        let text: String = env.get_string(&result)?.into();
        Ok(text)
    });
    if env.exception_check().unwrap_or(true) {
        let _ = env.exception_clear();
    }
    result.map_err(|e| format!("Storage provider JNI error: {e}"))
}

#[derive(Clone, Copy)]
enum Call {
    State,
    Choose,
    Forget,
    List,
    Preview,
}
async fn call(scheduler: Scheduler, method: Call, argument: String) -> Result<Value, String> {
    let task = scheduler
        .submit(move || {
            let plugin = plugin()?;
            let text = invoke(&plugin, method, &argument)?;
            if text.len() > 2 * 1024 * 1024 {
                return Err("Native response exceeds limits".into());
            }
            let value: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
            if let Some(error) = value.get("error").and_then(Value::as_str) {
                return Err(error.to_owned());
            }
            Ok(value)
        })
        .map_err(|e| e.to_string())?;
    task.await.map_err(|e| e.to_string())?
}

#[component]
pub fn App() -> Element {
    let scheduler = use_hook(|| Scheduler::new(2, 8));
    let mut session = use_signal(|| None::<DocumentSession>);
    let mut generation = use_signal(|| 0u64);
    let mut picking = use_signal(|| false);
    let mut message = use_signal(|| "Choose a folder. Only read access is requested.".to_owned());
    let mut preview = use_signal(|| None::<String>);
    let mut filter = use_signal(String::new);
    let mut version = use_signal(|| 0u64);
    let polling = scheduler.clone();
    use_future(move || {
        let polling = polling.clone();
        async move {
            loop {
                match call(polling.clone(), Call::State, String::new()).await {
                    Ok(state) => {
                        let root = state.get("root").and_then(Value::as_str);
                        let previous = session.peek().as_ref().map(|s| s.root().to_owned());
                        let epoch = state["generation"].as_u64().unwrap_or(0);
                        if root != previous.as_deref() || *generation.peek() != epoch {
                            generation.set(epoch);
                            session.set(root.and_then(|uri| {
                                DocumentSession::with_generation(uri.to_owned(), epoch).ok()
                            }));
                            preview.set(None);
                            filter.set(String::new());
                        }
                        let active = state["picking"].as_bool().unwrap_or(false);
                        if *picking.peek() != active {
                            picking.set(active);
                        }
                        if let Some(notice) = state.get("notice").and_then(Value::as_str) {
                            if *message.peek() != notice {
                                message.set(notice.to_owned());
                            }
                        }
                    }
                    Err(error) => {
                        session.set(None);
                        preview.set(None);
                        message.set(error);
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    });
    let request = use_memo(move || session.read().as_ref().map(DocumentSession::request));
    let listing_scheduler = scheduler.clone();
    let listing = use_resource(move || {
        let request = request();
        let _refresh = version();
        let scheduler = listing_scheduler.clone();
        async move {
            let Some(request) = request else {
                return Ok::<_, String>(None);
            };
            let page = call(scheduler, Call::List, request.folder.clone()).await?;
            if page["generation"].as_u64() != Some(request.generation) {
                return Err("Folder permission changed; waiting for the current grant".into());
            }
            Ok(Some((request, page)))
        }
    });
    let page = listing.read().clone();
    let page_request = request();
    // Authorize rows before exposing them to event handlers. Never show a stale folder's rows.
    let displayed = match &page {
        Some(Ok(Some((token, value)))) if Some(token) == page_request.as_ref() => value
            .get("documents")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    let accept = displayed
        .iter()
        .filter_map(|d| Some((d["uri"].as_str()?.to_owned(), d["directory"].as_bool()?)))
        .collect::<Vec<_>>();
    let acceptance = if let Some(token) = page_request.as_ref() {
        session.peek().as_ref().cloned().map(|mut current| {
            current.accept_page(token, &accept)?;
            Ok::<_, aurex_core::Error>(current)
        })
    } else {
        None
    };
    // Updating the session during render would invalidate hooks. Register the exact visible
    // page atomically inside each event, then authorize its selected identity.
    let allowed = matches!(acceptance, Some(Ok(_)));
    let authorized_session = acceptance.and_then(Result::ok).map(std::sync::Arc::new);
    let choose_scheduler = scheduler.clone();
    let forget_scheduler = scheduler.clone();
    let filter_text = filter().to_lowercase();
    let truncated = match &page {
        Some(Ok(Some((_, v)))) => v["truncated"].as_bool().unwrap_or(false),
        _ => false,
    };
    rsx! {
        style { {include_str!("../assets/app.css")} {include_str!("../assets/mobile.css")} }
        main { class: "app mobile-app",
            header { class: "topbar", div { class: "brand", "aurex" } span { class: "badge", "Read only" } }
            section { class: "mobile-content",
                h1 { "Your files. Your permission." }
                p { "Android Storage Access Framework · no broad storage permission" }
                div { class: "mobile-actions",
                    button { disabled: picking(), onclick: move |_| { let scheduler = choose_scheduler.clone(); spawn(async move {
                        if let Err(error) = call(scheduler, Call::Choose, String::new()).await { message.set(error); }
                    }); }, if picking() { "Choosing folder…" } else { "Choose a folder" } }
                    if session.read().is_some() {
                        button { disabled: picking(), onclick: move |_| { let scheduler = forget_scheduler.clone(); spawn(async move {
                            match call(scheduler, Call::Forget, String::new()).await {
                                Ok(_) => { session.set(None); preview.set(None); message.set("Folder permission forgotten. Files were not deleted.".into()); }
                                Err(error) => message.set(error),
                            }
                        }); }, "Forget permission" }
                    }
                }
                p { role: "status", "{message}" }
                if session.read().is_some() {
                    div { class: "mobile-actions",
                        button { disabled: !session.read().as_ref().is_some_and(DocumentSession::can_go_back), onclick: move |_| {
                            if let Some(current) = session.write().as_mut() { current.back(); }
                        }, "Back" }
                        button { onclick: move |_| version += 1, "Refresh" }
                        input { "aria-label": "Filter documents", placeholder: "Filter these files…", value: "{filter}", oninput: move |e| filter.set(e.value()) }
                    }
                    if let Some(Err(error)) = &page { p { role: "alert", "{error}" } }
                    if truncated { p { role: "status", "Showing the first 500 provider results. Narrow the folder on Android for additional files." } }
                    if !allowed { p { "Loading or validating folder…" } }
                    if allowed {
                        div { role: "list", "aria-label": "Folder documents",
                            for document in displayed.iter().filter(|d| d["name"].as_str().unwrap_or("").to_lowercase().contains(&filter_text)) {
                                { let uri=document["uri"].as_str().unwrap_or("").to_owned(); let name=document["name"].as_str().unwrap_or("Unnamed document").to_owned();
                                  let directory=document["directory"].as_bool().unwrap_or(false); let virtual_file=document["virtual"].as_bool().unwrap_or(true);
                                  let current=authorized_session.clone(); let scheduler=scheduler.clone(); let row_key=uri.clone(); rsx! {
                                    button { class: "mobile-document", key: "{row_key}", role: "listitem", disabled: virtual_file,
                                        onclick: move |_| {
                                            let Some(current)=current.clone() else { return; };
                                            if directory {
                                                let mut next=(*current).clone();
                                                match next.enter(&uri) { Ok(()) => { session.set(Some(next)); filter.set(String::new()); }, Err(e) => message.set(e.to_string()) }
                                            } else {
                                                if let Err(e)=current.authorize_preview(&uri) { message.set(e.to_string()); return; }
                                                let scheduler=scheduler.clone(); let uri=uri.clone(); let token:DocumentRequest=current.request();
                                                spawn(async move {
                                                    match call(scheduler, Call::Preview, uri).await {
                                                        Ok(value) if value["generation"].as_u64()==Some(token.generation) && session.peek().as_ref().is_some_and(|s| s.request()==token) => preview.set(value["text"].as_str().map(str::to_owned)),
                                                        Ok(_) => {}, Err(error) => message.set(error),
                                                    }
                                                });
                                            }
                                        }, span { if directory { "▰" } else { "▤" } } span { "{name}" }
                                        if virtual_file { span { "Provider-specific document" } }
                                    }
                                  } }
                            }
                        }
                        if displayed.is_empty() { p { "This folder is empty." } }
                    }
                } else { p { "Choose a folder using Android's system picker. Cancelling grants no new access." } }
            }
            if let Some(text) = preview() {
                div { class: "modal-backdrop", div { class: "modal preview-modal", role: "dialog", "aria-label": "Read-only document preview", "aria-modal": "true",
                    button { onclick: move |_| preview.set(None), "Close preview" }
                    p { "UTF-8 text only · maximum 256 KiB" } pre { "{text}" }
                } }
            }
        }
    }
}
