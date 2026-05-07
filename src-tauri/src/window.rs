use tauri::{Monitor, WebviewWindow};

/// Position the window at the bottom-right of the primary monitor's work area.
pub fn position_ball_bottom_right(window: &WebviewWindow, ball_bounds: (u32, u32)) {
    if let Some(monitor) = get_primary_monitor(window) {
        let work_area = monitor.work_area();
        let x = work_area.x + work_area.width as i32 - ball_bounds.0 as i32 - 32;
        let y = work_area.y + work_area.height as i32 - ball_bounds.1 as i32 - 32;
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x,
            y,
        }));
    }
}

/// Get the primary monitor for the window.
fn get_primary_monitor(window: &WebviewWindow) -> Option<Monitor> {
    window.primary_monitor().ok().flatten().or_else(|| {
        window.available_monitors().ok().and_then(|monitors| monitors.first().cloned())
    })
}

static mut CURRENT_MODE: &str = "ball";

/// Get current window mode.
pub fn get_mode(_window: &WebviewWindow) -> String {
    unsafe { CURRENT_MODE.to_string() }
}

/// Switch between ball and panel mode.
pub fn set_mode(
    window: &WebviewWindow,
    mode: &str,
    ball_bounds: (u32, u32),
    panel_bounds: (u32, u32),
    panel_min_bounds: (u32, u32),
) {
    unsafe {
        CURRENT_MODE = if mode == "panel" { "panel" } else { "ball" };
    }

    let current_pos = window.outer_position().unwrap_or(tauri::PhysicalPosition { x: 0, y: 0 });
    let current_size = window.outer_size().unwrap_or(tauri::PhysicalSize {
        width: ball_bounds.0,
        height: ball_bounds.1,
    });

    if mode == "panel" {
        let new_x = current_pos.x + current_size.width as i32 - panel_bounds.0 as i32;
        let new_y = current_pos.y + current_size.height as i32 - panel_bounds.1 as i32;
        let _ = window.set_min_size(Some(tauri::Size::Physical(tauri::PhysicalSize {
            width: panel_min_bounds.0,
            height: panel_min_bounds.1,
        })));
        let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: panel_bounds.0,
            height: panel_bounds.1,
        }));
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: new_x,
            y: new_y,
        }));
    } else {
        let new_x = current_pos.x + current_size.width as i32 - ball_bounds.0 as i32;
        let new_y = current_pos.y + current_size.height as i32 - ball_bounds.1 as i32;
        let _ = window.set_min_size(Some(tauri::Size::Physical(tauri::PhysicalSize {
            width: ball_bounds.0,
            height: ball_bounds.1,
        })));
        let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: ball_bounds.0,
            height: ball_bounds.1,
        }));
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: new_x,
            y: new_y,
        }));
    }
}

/// Resize the window by delta, clamped to min and screen bounds.
pub fn resize_by(window: &WebviewWindow, delta_width: i32, delta_height: i32, min_bounds: (u32, u32)) {
    let is_panel = unsafe { CURRENT_MODE == "panel" };
    if !is_panel {
        return;
    }

    let size = window.outer_size().unwrap_or(tauri::PhysicalSize { width: 0, height: 0 });
    let monitor = get_primary_monitor(window);
    let work_area = monitor.as_ref().map(|m| m.work_area());

    let next_width = (size.width as i32 + delta_width).max(min_bounds.0 as i32) as u32;
    let next_height = (size.height as i32 + delta_height).max(min_bounds.1 as i32) as u32;

    let max_width = work_area.map(|a| a.width).unwrap_or(1920);
    let max_height = work_area.map(|a| a.height).unwrap_or(1080);

    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
        width: next_width.min(max_width),
        height: next_height.min(max_height),
    }));
}
