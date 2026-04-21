use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config;

const DEFAULT_MAIN_WINDOW_WIDTH: f32 = 420.0;
const DEFAULT_MAIN_WINDOW_HEIGHT: f32 = 900.0;
const MIN_MAIN_WINDOW_WIDTH: f32 = 40.0;
const MIN_MAIN_WINDOW_HEIGHT: f32 = 100.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MainWindowState {
    pub width: f32,
    pub height: f32,
    pub maximized: bool,
}

impl Default for MainWindowState {
    fn default() -> Self {
        Self {
            width: DEFAULT_MAIN_WINDOW_WIDTH,
            height: DEFAULT_MAIN_WINDOW_HEIGHT,
            maximized: false,
        }
    }
}

impl MainWindowState {
    pub fn sanitized(self) -> Self {
        Self {
            width: sanitize_dimension(
                self.width,
                MIN_MAIN_WINDOW_WIDTH,
                DEFAULT_MAIN_WINDOW_WIDTH,
            ),
            height: sanitize_dimension(
                self.height,
                MIN_MAIN_WINDOW_HEIGHT,
                DEFAULT_MAIN_WINDOW_HEIGHT,
            ),
            maximized: self.maximized,
        }
    }
}

pub fn main_window_state_path() -> PathBuf {
    let config_path = config::resolve_config_path_for_write();
    if let Some(parent) = config_path.parent() {
        return parent.join("main-window-state.json");
    }

    PathBuf::from("main-window-state.json")
}

pub fn save_main_window_state(path: &Path, state: &MainWindowState) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let content =
        serde_json::to_string_pretty(&state.sanitized()).map_err(|err| err.to_string())?;
    fs::write(path, content).map_err(|err| format!("failed to write {}: {}", path.display(), err))
}

pub fn load_main_window_state(path: &Path) -> Option<MainWindowState> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str::<MainWindowState>(&content)
        .ok()
        .map(MainWindowState::sanitized)
}

fn sanitize_dimension(value: f32, minimum: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.max(minimum)
    } else {
        fallback
    }
}
