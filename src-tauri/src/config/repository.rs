use crate::app_state::AppConfig;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ConfigRepository {
    path: PathBuf,
}

impl ConfigRepository {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<Option<AppConfig>, String> {
        if !self.path.exists() {
            return Ok(None);
        }

        let raw = fs::read_to_string(&self.path).map_err(|error| error.to_string())?;
        let config = serde_json::from_str(&raw).map_err(|error| error.to_string())?;
        Ok(Some(config))
    }

    pub fn save(&self, config: &AppConfig) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }

        let raw = serde_json::to_string_pretty(config).map_err(|error| error.to_string())?;
        fs::write(&self.path, raw).map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::ConfigRepository;
    use crate::app_state::{AppConfig, WatchGroupConfig};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_config() -> AppConfig {
        AppConfig {
            api_base_url: "https://example.com".into(),
            api_key: "secret".into(),
            polling_interval_seconds: 60,
            selected_group_id: "btcusdt".into(),
            groups: vec![WatchGroupConfig {
                id: "btcusdt".into(),
                symbol: "BTCUSDT".into(),
                signal_types: vec!["vegas".into()],
                periods: vec!["60".into()],
                selected_timeline_period: "60".into(),
            }],
            dashboard: crate::app_state::DashboardPreferences {
                layout_preset: "table".into(),
                density: "comfortable".into(),
            },
            window_policy: crate::app_state::WindowPolicyConfig {
                dock_side: "right".into(),
                widget_width: 280,
                widget_height: 720,
                top_offset: 96,
            },
        }
    }

    #[test]
    fn saves_and_loads_config() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("duration")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("watch-tower-test-{unique}.json"));
        let repository = ConfigRepository::new(path.clone());
        let config = test_config();

        repository.save(&config).expect("save");
        let loaded = repository.load().expect("load").expect("config");

        assert_eq!(loaded.selected_group_id, config.selected_group_id);

        let _ = std::fs::remove_file(path);
    }
}
