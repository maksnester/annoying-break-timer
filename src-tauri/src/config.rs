use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Settings {
    pub focus_minutes: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self { focus_minutes: 25 }
    }
}

impl Settings {
    fn config_path(app: &AppHandle) -> tauri::Result<PathBuf> {
        let dir = app.path().app_config_dir()?;
        std::fs::create_dir_all(&dir)?;
        Ok(dir.join("settings.json"))
    }

    pub fn load(app: &AppHandle) -> Self {
        match Self::config_path(app) {
            Ok(path) => load_from_path(&path),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, app: &AppHandle) -> tauri::Result<()> {
        let path = Self::config_path(app)?;
        save_to_path(self, &path)
    }
}

pub fn load_from_path(path: &Path) -> Settings {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_to_path(settings: &Settings, path: &Path) -> tauri::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings)?;
    std::fs::write(path, json)?;
    Ok(())
}

#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> Settings {
    Settings::load(&app)
}

#[tauri::command]
pub fn set_settings(settings: Settings, app: tauri::AppHandle) -> tauri::Result<()> {
    settings.save(&app)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "macos-timer-config-test-{}-{}",
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn default_focus_minutes_is_25() {
        assert_eq!(Settings::default().focus_minutes, 25);
    }

    #[test]
    fn saves_and_loads_focus_minutes_round_trip() {
        let dir = unique_test_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");

        let settings = Settings { focus_minutes: 42 };
        save_to_path(&settings, &path).unwrap();

        let loaded = load_from_path(&path);
        assert_eq!(loaded.focus_minutes, 42);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_returns_default_for_missing_file() {
        let path = unique_test_dir().join("missing.json");
        assert_eq!(load_from_path(&path), Settings::default());
    }
}
