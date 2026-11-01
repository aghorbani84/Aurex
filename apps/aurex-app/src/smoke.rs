//! Explicit opt-in, feature-gated test probe. No mutation or production auto-exit path.
use dioxus::prelude::*;
use dioxus_document as document;
pub fn install(enabled: bool) {
    use_future(move || async move {
        if !enabled {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        match document::eval(include_str!("smoke.js")).await {
            Ok(value) if value.as_bool() == Some(true) => {
                println!(
                    "AUREX_GUI_SMOKE_PASS: native DOM, browsing, tabs, filter, preview, read-only controls"
                );
                dioxus::desktop::window().close();
            }
            outcome => {
                eprintln!(
                    "AUREX_GUI_SMOKE_FAIL: native UI assertion or evaluation failed: {outcome:?}"
                );
                std::process::exit(1);
            }
        }
    });
}
