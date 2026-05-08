use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Monitor, WebviewWindow};

static IS_PANEL_MODE: AtomicBool = AtomicBool::new(false);

/// Position the window at the bottom-right of the primary monitor's work area.
pub fn position_ball_bottom_right(window: &WebviewWindow, ball_bounds: (u32, u32)) {
    if let Some(monitor) = get_primary_monitor(window) {
        let work_area = monitor.work_area();
        let x = work_area.position.x + work_area.size.width as i32 - ball_bounds.0 as i32 - 32;
        let y = work_area.position.y + work_area.size.height as i32 - ball_bounds.1 as i32 - 32;
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x,
            y,
        }));
    }
}

/// Get the primary monitor for the window.
fn get_primary_monitor(window: &WebviewWindow) -> Option<Monitor> {
    window.primary_monitor().ok().flatten().or_else(|| {
        window
            .available_monitors()
            .ok()
            .and_then(|monitors| monitors.first().cloned())
    })
}

/// Get current window mode.
pub fn get_mode(_window: &WebviewWindow) -> &'static str {
    if IS_PANEL_MODE.load(Ordering::Relaxed) {
        "panel"
    } else {
        "ball"
    }
}

/// Switch between ball and panel mode.
pub fn set_mode(
    window: &WebviewWindow,
    mode: &str,
    ball_bounds: (u32, u32),
    panel_bounds: (u32, u32),
    panel_min_bounds: (u32, u32),
) {
    let is_panel = mode == "panel";
    IS_PANEL_MODE.store(is_panel, Ordering::Relaxed);

    let current_pos = window
        .outer_position()
        .unwrap_or(tauri::PhysicalPosition { x: 0, y: 0 });
    let current_size = window.outer_size().unwrap_or(tauri::PhysicalSize {
        width: ball_bounds.0,
        height: ball_bounds.1,
    });

    let (target_w, target_h) = if is_panel { panel_bounds } else { ball_bounds };
    let (min_w, min_h) = if is_panel {
        panel_min_bounds
    } else {
        ball_bounds
    };

    let new_x = current_pos.x + current_size.width as i32 - target_w as i32;
    let new_y = current_pos.y + current_size.height as i32 - target_h as i32;

    let _ = window.set_min_size(Some(tauri::Size::Physical(tauri::PhysicalSize {
        width: min_w,
        height: min_h,
    })));
    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
        width: target_w,
        height: target_h,
    }));
    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
        x: new_x,
        y: new_y,
    }));
}
