// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod history_store;
mod window;

use arboard::Clipboard;
use history_store::{HistoryItem, HistoryStore};
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

const BALL_BOUNDS: (u32, u32) = (72, 72);
const PANEL_BOUNDS: (u32, u32) = (360, 480);
const PANEL_MIN_BOUNDS: (u32, u32) = (300, 360);

#[tauri::command]
fn set_window_mode(app: AppHandle, mode: String) -> String {
    if let Some(window) = app.get_webview_window("main") {
        window::set_mode(&window, &mode, BALL_BOUNDS, PANEL_BOUNDS, PANEL_MIN_BOUNDS);
    }
    mode
}

#[tauri::command]
fn toggle_panel(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let current = window::get_mode(&window);
        let next = if current == "panel" { "ball" } else { "panel" };
        window::set_mode(&window, next, BALL_BOUNDS, PANEL_BOUNDS, PANEL_MIN_BOUNDS);
        let _ = window.show();
        let _ = window.set_focus();
        let _ = app.emit("window:mode", next);
    }
}

fn toggle_to_panel(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        window::set_mode(&window, "panel", BALL_BOUNDS, PANEL_BOUNDS, PANEL_MIN_BOUNDS);
        let _ = window.show();
        let _ = window.set_focus();
        let _ = app.emit("window:mode", "panel");
    }
}

fn toggle_to_ball(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        window::set_mode(&window, "ball", BALL_BOUNDS, PANEL_BOUNDS, PANEL_MIN_BOUNDS);
        let _ = window.show();
        let _ = app.emit("window:mode", "ball");
    }
}

#[tauri::command]
fn start_dragging(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "window not found".to_string())?;
    window.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn get_history(store: State<Arc<HistoryStore>>) -> Result<Vec<HistoryItem>, String> {
    store.get_all()
}

#[tauri::command]
fn delete_history_item(
    app: AppHandle,
    store: State<Arc<HistoryStore>>,
    id: String,
) -> Result<(), String> {
    store.delete(&id)?;
    let _ = app.emit("history:deleted", id);
    Ok(())
}

#[tauri::command]
fn clear_history(app: AppHandle, store: State<Arc<HistoryStore>>) -> Result<(), String> {
    store.clear()?;
    let _ = app.emit("history:cleared", ());
    Ok(())
}

#[tauri::command]
fn copy_to_clipboard(body: String) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(body).map_err(|e| e.to_string())?;
    Ok(())
}

fn main() {
    let toggle_shortcut =
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV);
    let quit_shortcut =
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyQ);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    if shortcut == &toggle_shortcut {
                        toggle_panel(app.clone());
                    } else if shortcut == &quit_shortcut {
                        app.exit(0);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            set_window_mode,
            toggle_panel,
            start_dragging,
            quit_app,
            get_history,
            delete_history_item,
            clear_history,
            copy_to_clipboard,
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let store = Arc::new(HistoryStore::new(app_handle.clone()));
            app.manage(store.clone());

            clipboard::start_clipboard_monitor(app_handle.clone(), store.clone());

            // 注册全局快捷键
            let gs = app.global_shortcut();
            if let Err(e) = gs.register(toggle_shortcut) {
                eprintln!("[clipball] Failed to register toggle shortcut: {}", e);
            }
            if let Err(e) = gs.register(quit_shortcut) {
                eprintln!("[clipball] Failed to register quit shortcut: {}", e);
            }

            // 构建托盘菜单
            let open_item = MenuItem::with_id(app, "open", "打开面板", true, None::<&str>)?;
            let show_ball = MenuItem::with_id(app, "ball", "显示悬浮球", true, None::<&str>)?;
            let clear_item =
                MenuItem::with_id(app, "clear", "清空历史", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出 ClipBall", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[&open_item, &show_ball, &clear_item, &quit_item],
            )?;

            let _tray = TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("ClipBall - 剪贴板历史")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => toggle_to_panel(app.clone()),
                    "ball" => toggle_to_ball(app.clone()),
                    "clear" => {
                        if let Some(store) = app.try_state::<Arc<HistoryStore>>() {
                            let _ = store.clear();
                            let _ = app.emit("history:cleared", ());
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_panel(tray.app_handle().clone());
                    }
                })
                .build(app)?;

            if let Some(window) = app.get_webview_window("main") {
                window::position_ball_bottom_right(&window, BALL_BOUNDS);
                let _ = window.show();
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
