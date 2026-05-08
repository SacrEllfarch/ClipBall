use crate::settings_store::SettingsStore;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(rename = "typeLabel")]
    pub type_label: String,
    pub icon: String,
    #[serde(rename = "timeLabel")]
    pub time_label: String,
    pub body: String,
    pub hash: String,
    pub created_at: u64,
}

pub struct HistoryStore {
    items: Mutex<Vec<HistoryItem>>,
    app_handle: AppHandle,
    settings: Arc<SettingsStore>,
}

impl HistoryStore {
    pub fn new(app_handle: AppHandle, settings: Arc<SettingsStore>) -> Self {
        let store = Self {
            items: Mutex::new(Vec::new()),
            app_handle,
            settings,
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
        app_data_dir.join("history.json")
    }

    fn load(&self) -> Result<(), String> {
        let path = self.data_path();
        if !path.exists() {
            return Ok(());
        }
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let items: Vec<HistoryItem> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        let mut guard = self.items.lock().map_err(|e| e.to_string())?;
        *guard = items;
        Ok(())
    }

    fn save(&self) -> Result<(), String> {
        let path = self.data_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let guard = self.items.lock().map_err(|e| e.to_string())?;
        let content = serde_json::to_string_pretty(&*guard).map_err(|e| e.to_string())?;
        fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_all(&self) -> Result<Vec<HistoryItem>, String> {
        let guard = self.items.lock().map_err(|e| e.to_string())?;
        let mut items = guard.clone();
        drop(guard);
        for item in &mut items {
            item.time_label = format_relative_time(item.created_at);
        }
        Ok(items)
    }

    pub fn add(&self, body: &str) -> Result<Option<HistoryItem>, String> {
        let hash = compute_hash(body);

        let mut guard = self.items.lock().map_err(|e| e.to_string())?;

        // 去重：如果第一条（最新）记录的 hash 相同，不添加
        if let Some(first) = guard.first() {
            if first.hash == hash {
                return Ok(None);
            }
        }

        let (r#type, type_label, icon) = classify_content(body);
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let id = format!("{}-{}", r#type, &hash[..8.min(hash.len())]);

        let item = HistoryItem {
            id,
            r#type,
            type_label,
            icon,
            time_label: "刚刚".to_string(),
            body: body.to_string(),
            hash,
            created_at,
        };

        guard.insert(0, item.clone());

        let max_count = self.settings.max_history();
        if guard.len() > max_count {
            guard.truncate(max_count);
        }

        drop(guard);
        self.save()?;
        Ok(Some(item))
    }

    pub fn delete(&self, id: &str) -> Result<(), String> {
        let mut guard = self.items.lock().map_err(|e| e.to_string())?;
        guard.retain(|item| item.id != id);
        drop(guard);
        self.save()?;
        Ok(())
    }

    pub fn clear(&self) -> Result<(), String> {
        let mut guard = self.items.lock().map_err(|e| e.to_string())?;
        guard.clear();
        drop(guard);
        self.save()?;
        Ok(())
    }
}

fn compute_hash(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn classify_content(body: &str) -> (String, String, String) {
    let trimmed = body.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        ("link".to_string(), "链接".to_string(), "🔗".to_string())
    } else {
        ("text".to_string(), "文本".to_string(), "📝".to_string())
    }
}

fn format_relative_time(created_at: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let diff_ms = now.saturating_sub(created_at);
    let diff_min = diff_ms / 60000;
    let diff_hour = diff_min / 60;
    let diff_day = diff_hour / 24;

    if diff_min < 1 {
        "刚刚".to_string()
    } else if diff_min < 60 {
        format!("{} 分钟前", diff_min)
    } else if diff_hour < 24 {
        format!("{} 小时前", diff_hour)
    } else if diff_day < 30 {
        format!("{} 天前", diff_day)
    } else {
        "很久以前".to_string()
    }
}
