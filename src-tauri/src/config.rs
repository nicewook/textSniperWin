use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use windows::Win32::System::Registry::*;
use windows::core::w;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub auto_start: bool,
    pub first_run: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_start: false,
            first_run: true,
        }
    }
}

impl AppConfig {
    pub fn config_dir() -> PathBuf {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(appdata).join("TextSniperWin")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load() -> Self {
        Self::load_from(&Self::config_path())
    }

    pub fn load_from(path: &PathBuf) -> Self {
        match std::fs::read_to_string(path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.save_to(&Self::config_path())
    }

    pub fn save_to(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

pub fn set_auto_start(enable: bool) -> Result<(), String> {
    unsafe {
        let mut hkey = HKEY::default();
        let err = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            w!(r"Software\Microsoft\Windows\CurrentVersion\Run"),
            None,
            KEY_SET_VALUE | KEY_QUERY_VALUE,
            &mut hkey,
        );
        if err.0 != 0 {
            return Err(format!("RegOpenKeyExW failed: {}", err.0));
        }

        if enable {
            let exe_path = std::env::current_exe()
                .map_err(|e| e.to_string())?;
            let value: Vec<u16> = exe_path.to_string_lossy()
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let bytes = std::slice::from_raw_parts(
                value.as_ptr() as *const u8,
                value.len() * 2,
            );
            let err = RegSetValueExW(
                hkey,
                w!("TextSniperWin"),
                None,
                REG_SZ,
                Some(bytes),
            );
            if err.0 != 0 {
                let _ = RegCloseKey(hkey);
                return Err(format!("RegSetValueExW failed: {}", err.0));
            }
        } else {
            let _ = RegDeleteValueW(hkey, w!("TextSniperWin"));
        }

        let _ = RegCloseKey(hkey);
        Ok(())
    }
}

pub fn is_auto_start_enabled() -> bool {
    unsafe {
        let mut hkey = HKEY::default();
        let err = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            w!(r"Software\Microsoft\Windows\CurrentVersion\Run"),
            None,
            KEY_QUERY_VALUE,
            &mut hkey,
        );
        if err.0 != 0 {
            return false;
        }

        let result = RegQueryValueExW(
            hkey,
            w!("TextSniperWin"),
            None,
            None,
            None,
            None,
        );

        let _ = RegCloseKey(hkey);
        result.0 == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_config_path() -> PathBuf {
        let dir = std::env::temp_dir().join("textsniper_test");
        fs::create_dir_all(&dir).unwrap();
        dir.join("config.json")
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(!config.auto_start);
        assert!(config.first_run);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let path = temp_config_path();
        let config = AppConfig {
            auto_start: true,
            first_run: false,
        };

        // Save
        let dir = path.parent().unwrap();
        fs::create_dir_all(dir).unwrap();
        let json = serde_json::to_string_pretty(&config).unwrap();
        fs::write(&path, json).unwrap();

        // Load
        let data = fs::read_to_string(&path).unwrap();
        let loaded: AppConfig = serde_json::from_str(&data).unwrap();
        assert_eq!(loaded, config);

        // Cleanup
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_load_missing_file_returns_default() {
        let path = std::env::temp_dir().join("textsniper_nonexistent").join("config.json");
        let _ = std::fs::remove_file(&path);
        // load_from should return default when file doesn't exist
        let config = AppConfig::load_from(&path);
        assert_eq!(config, AppConfig::default());
    }

    #[test]
    fn test_auto_start_toggle() {
        // 현재 상태 저장
        let was_enabled = is_auto_start_enabled();

        // 활성화
        set_auto_start(true).unwrap();
        assert!(is_auto_start_enabled());

        // 비활성화
        set_auto_start(false).unwrap();
        assert!(!is_auto_start_enabled());

        // 원래 상태 복원
        if was_enabled {
            set_auto_start(true).unwrap();
        }
    }
}
