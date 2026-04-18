use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::runtime::{AppConfig, WatchGroup};

fn default_config() -> AppConfig {
    AppConfig {
        groups: vec![WatchGroup::default()],
        ..Default::default()
    }
}

fn push_config_candidates(candidates: &mut Vec<PathBuf>, base: PathBuf) {
    let mut current = Some(base.as_path());
    let mut depth = 0;

    while let Some(path) = current {
        candidates.push(path.join("config.yaml"));
        candidates.push(path.join("config.yaml.example"));
        current = path.parent();
        depth += 1;

        if depth >= 8 {
            break;
        }
    }
}

pub fn resolve_config_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(current) = std::env::current_dir() {
        push_config_candidates(&mut candidates, current);
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            push_config_candidates(&mut candidates, parent.to_path_buf());
        }
    }

    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .filter(|candidate| seen.insert(candidate.clone()))
        .collect()
}

pub fn load_config_from_candidates(candidates: &[PathBuf]) -> AppConfig {
    for candidate in candidates {
        if let Ok(content) = fs::read_to_string(candidate) {
            if let Ok(config) = serde_yaml::from_str::<AppConfig>(&content) {
                return config;
            }
        }
    }

    default_config()
}

pub fn load_config() -> AppConfig {
    load_config_from_candidates(&resolve_config_candidates())
}

pub fn resolve_config_path_for_write() -> PathBuf {
    for candidate in resolve_config_candidates() {
        if candidate.file_name().and_then(|value| value.to_str()) == Some("config.yaml")
            && candidate.exists()
        {
            return candidate;
        }
    }

    if let Ok(current) = std::env::current_dir() {
        if current.file_name().and_then(|value| value.to_str()) == Some("native-shell") {
            if let Some(parent) = current.parent() {
                return parent.join("config.yaml");
            }
        }

        return current.join("config.yaml");
    }

    PathBuf::from("config.yaml")
}

pub fn save_config_to_path(config: &AppConfig, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let content = serde_yaml::to_string(config).map_err(|err| err.to_string())?;
    fs::write(path, content).map_err(|err| format!("failed to write {}: {}", path.display(), err))
}

pub fn config_location_hint() -> String {
    resolve_config_candidates()
        .into_iter()
        .find(|path| Path::new(path).exists())
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "config.yaml.example not found".to_string())
}
