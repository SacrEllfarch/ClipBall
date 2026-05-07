// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod window;

use tauri::{AppHandle, Manager, WindowEvent};

const BALL_BOUNDS: (u32, u32) = (72, 72);
const PANEL_BOUNDS: (u32, u32) = (380, 540);
const PANEL_MIN_BOUNDS: (u32, u32) = (300, 380);

#[tauri::command]
fn set_window_mode(app: AppHandle, mode: String) -> String {
    let window = app.get_webview_window("main").unwrap();
    window::set_mode(&window, &mode, BALL_BOUNDS, PANEL_BOUNDS, PANEL_MIN_BOUNDS);
    mode
}

#[tauri::command]
fn get_window_mode(app: AppHandle) -> String {
    let window = app.get_webview_window("main").unwrap();
    window::get_mode(&window)
}

#[tauri::command]
fn resize_window_by(app: AppHandle, width: i32, height: i32) {
    let window = app.get_webview_window("main").unwrap();
    window::resize_by(&window, width, height, PANEL_MIN_BOUNDS);
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            set_window_mode,
            get_window_mode,
            resize_window_by,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window::position_ball_bottom_right(&window, BALL_BOUNDS);
            window.show().unwrap();
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { .. } = event {
                window.hide().unwrap();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
