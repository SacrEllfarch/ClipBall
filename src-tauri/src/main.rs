// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod history_store;
mod settings_store;
mod window;

use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings as EnigoSettings};
use history_store::{HistoryItem, HistoryStore};
use settings_store::{Settings, SettingsStore};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_autostart::{ManagerExt, MacosLauncher};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

const BALL_BOUNDS: (u32, u32) = (72, 72);
const PANEL_BOUNDS: (u32, u32) = (360, 480);
const PANEL_MIN_BOUNDS: (u32, u32) = (300, 360);

#[tauri::command]
fn set_window_mode(app: AppHandle, mode: String) -> String {
    if let Some(window) = app.get_webview_window("main") {
        window::set_mode(&window, &mode, BALL_BOUNDS, PANEL_BOUNDS, PANEL_MIN_BOUNDS);
        if mode == "ball" {
            persist_ball_position(&app);
        }
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
        if next == "ball" {
            persist_ball_position(&app);
        }
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
        persist_ball_position(&app);
    }
}

fn persist_ball_position(app: &AppHandle) {
    if let (Some(window), Some(settings)) = (
        app.get_webview_window("main"),
        app.try_state::<Arc<SettingsStore>>(),
    ) {
        if let Ok(pos) = window.outer_position() {
            let _ = settings.update_ball_position(pos.x, pos.y);
        }
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

/// 快速粘贴：写剪贴板 → 收起到悬浮球（让前台获焦）→ 模拟 Ctrl+V。
/// 任一阶段失败都至少保证内容已写入剪贴板（退化为仅复制）。
#[tauri::command]
fn paste_history_item(app: AppHandle, body: String) -> Result<bool, String> {
    // 1. 写剪贴板（必须成功，否则即使粘贴也无内容）
    {
        let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
        clipboard.set_text(body).map_err(|e| e.to_string())?;
    }

    // 2. 收起到悬浮球，让上一个窗口重新获得焦点
    if let Some(window) = app.get_webview_window("main") {
        window::set_mode(&window, "ball", BALL_BOUNDS, PANEL_BOUNDS, PANEL_MIN_BOUNDS);
        let _ = app.emit("window:mode", "ball");
    }

    // 3. 异步模拟 Ctrl+V，等待焦点切换
    let app_clone = app.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(120));
        let result = simulate_paste();
        if let Err(e) = result {
            eprintln!("[clipball] simulate paste failed: {}", e);
            let _ = app_clone.emit("paste:fallback", e);
        } else {
            let _ = app_clone.emit("paste:done", ());
        }
    });

    Ok(true)
}

fn simulate_paste() -> Result<(), String> {
    let mut enigo = Enigo::new(&EnigoSettings::default()).map_err(|e| e.to_string())?;
    enigo
        .key(Key::Control, Direction::Press)
        .map_err(|e| e.to_string())?;
    enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| e.to_string())?;
    enigo
        .key(Key::Control, Direction::Release)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_settings(store: State<Arc<SettingsStore>>) -> Settings {
    store.get()
}

#[tauri::command]
fn update_settings(
    app: AppHandle,
    store: State<Arc<SettingsStore>>,
    settings: Settings,
) -> Result<Settings, String> {
    let saved = store.update(settings)?;

    // 同步开机启动状态
    let autostart_mgr = app.autolaunch();
    let currently = autostart_mgr.is_enabled().unwrap_or(false);
    if saved.autostart && !currently {
        let _ = autostart_mgr.enable();
    } else if !saved.autostart && currently {
        let _ = autostart_mgr.disable();
    }

    let _ = app.emit("settings:updated", saved.clone());
    Ok(saved)
}

fn main() {
    let toggle_shortcut =
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV);
    let quit_shortcut =
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyQ);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
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
            paste_history_item,
            get_settings,
            update_settings,
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // 初始化设置（先于 history 加载，决定 max_history）
            let settings_store = Arc::new(SettingsStore::new(app_handle.clone()));
            app.manage(settings_store.clone());

            let history_store =
                Arc::new(HistoryStore::new(app_handle.clone(), settings_store.clone()));
            app.manage(history_store.clone());

            clipboard::start_clipboard_monitor(
                app_handle.clone(),
                history_store.clone(),
                settings_store.clone(),
            );

            // 同步 autostart 实际状态
            let cur_settings = settings_store.get();
            let autostart_mgr = app.autolaunch();
            let actually_enabled = autostart_mgr.is_enabled().unwrap_or(false);
            if cur_settings.autostart && !actually_enabled {
                let _ = autostart_mgr.enable();
            } else if !cur_settings.autostart && actually_enabled {
                let _ = autostart_mgr.disable();
            }

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

            // 初始位置：优先读 settings 记忆位置，否则右下角
            if let Some(window) = app.get_webview_window("main") {
                let s = settings_store.get();
                if s.remember_position {
                    if let (Some(x), Some(y)) = (s.last_ball_x, s.last_ball_y) {
                        let _ = window.set_position(tauri::Position::Physical(
                            tauri::PhysicalPosition { x, y },
                        ));
                    } else {
                        window::position_ball_bottom_right(&window, BALL_BOUNDS);
                    }
                } else {
                    window::position_ball_bottom_right(&window, BALL_BOUNDS);
                }
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
