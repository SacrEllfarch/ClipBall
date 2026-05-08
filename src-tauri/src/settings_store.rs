use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

const DEFAULT_MAX_HISTORY: usize = 100;
const MIN_MAX_HISTORY: usize = 10;
const MAX_MAX_HISTORY: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_max_history", rename = "maxHistory")]
    pub max_history: usize,
    #[serde(default)]
    pub paused: bool,
    #[serde(default, rename = "autostart")]
    pub autostart: bool,
    #[serde(default = "default_remember_position", rename = "rememberPosition")]
    pub remember_position: bool,
    #[serde(default, rename = "lastBallX")]
    pub last_ball_x: Option<i32>,
    #[serde(default, rename = "lastBallY")]
    pub last_ball_y: Option<i32>,
}

fn default_max_history() -> usize {
    DEFAULT_MAX_HISTORY
}

fn default_remember_position() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_history: DEFAULT_MAX_HISTORY,
            paused: false,
            autostart: false,
            remember_position: true,
            last_ball_x: None,
            last_ball_y: None,
        }
    }
}

pub struct SettingsStore {
    inner: Mutex<Settings>,
    app_handle: AppHandle,
}

impl SettingsStore {
    pub fn new(app_handle: AppHandle) -> Self {
        let store = Self {
            inner: Mutex::new(Settings::default()),
            app_handle,
        };
        let _ = store.load();
        store
    }

    fn data_path(&self) -> PathBuf {
        let app_data_dir = self
            .app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        app_data_dir.join("settings.json")
    }

    fn load(&self) -> Result<(), String> {
        let path = self.data_path();
        if !path.exists() {
            return Ok(());
        }
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let settings: Settings = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        let mut guard = self.inner.lock().map_err(|e| e.to_string())?;
        *guard = sanitize(settings);
        Ok(())
    }

    fn save(&self) -> Result<(), String> {
        let path = self.data_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let guard = self.inner.lock().map_err(|e| e.to_string())?;
        let content = serde_json::to_string_pretty(&*guard).map_err(|e| e.to_string())?;
        fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get(&self) -> Settings {
        self.inner
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    pub fn is_paused(&self) -> bool {
        self.inner.lock().map(|g| g.paused).unwrap_or(false)
    }

    pub fn max_history(&self) -> usize {
        self.inner
            .lock()
            .map(|g| g.max_history)
            .unwrap_or(DEFAULT_MAX_HISTORY)
    }

    pub fn update(&self, new_settings: Settings) -> Result<Settings, String> {
        let cleaned = sanitize(new_settings);
        {
            let mut guard = self.inner.lock().map_err(|e| e.to_string())?;
            *guard = cleaned.clone();
        }
        self.save()?;
        Ok(cleaned)
    }

    pub fn update_ball_position(&self, x: i32, y: i32) -> Result<(), String> {
        let mut should_save = false;
        {
            let mut guard = self.inner.lock().map_err(|e| e.to_string())?;
            if guard.remember_position {
                guard.last_ball_x = Some(x);
                guard.last_ball_y = Some(y);
                should_save = true;
            }
        }
        if should_save {
            self.save()?;
        }
        Ok(())
    }
}

fn sanitize(mut s: Settings) -> Settings {
    if s.max_history < MIN_MAX_HISTORY {
        s.max_history = MIN_MAX_HISTORY;
    }
    if s.max_history > MAX_MAX_HISTORY {
        s.max_history = MAX_MAX_HISTORY;
    }
    s
}
