use crate::history_store::HistoryStore;
use crate::settings_store::SettingsStore;
use arboard::Clipboard;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub fn start_clipboard_monitor(
    app_handle: AppHandle,
    store: Arc<HistoryStore>,
    settings: Arc<SettingsStore>,
) {
    thread::spawn(move || {
        let mut clipboard = match Clipboard::new() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[clipball] Failed to initialize clipboard: {}", e);
                return;
            }
        };

        let mut last_text: Option<String> = None;
        let poll_interval = Duration::from_millis(500);

        loop {
            if !settings.is_paused() {
                if let Ok(text) = clipboard.get_text() {
                    let trimmed = text.trim().to_string();
                    if !trimmed.is_empty() {
                        let changed = last_text.as_ref() != Some(&trimmed);
                        if changed {
                            last_text = Some(trimmed.clone());
                            match store.add(&trimmed) {
                                Ok(Some(item)) => {
                                    let _ = app_handle.emit("history:updated", item);
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    eprintln!("[clipball] Failed to add history: {}", e);
                                }
                            }
                        }
                    }
                }
            }

            thread::sleep(poll_interval);
        }
    });
}
