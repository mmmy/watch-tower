use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config;

pub const WIDGET_EDGE_THRESHOLD: f64 = 56.0;
pub const WIDGET_PEEK: f64 = 24.0;
pub const WIDGET_VISIBLE_MARGIN: f64 = 12.0;
pub const WIDGET_MARGIN_X: f64 = 28.0;
pub const WIDGET_MARGIN_Y: f64 = 32.0;
pub const WIDGET_MIN_VISIBLE_MARGIN: f64 = 12.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WidgetDockSide {
    Free,
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WidgetPlacement {
    pub x: f64,
    pub y: f64,
    pub dock: WidgetDockSide,
    pub auto_hidden: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkArea {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowPlacement {
    pub x: f64,
    pub y: f64,
}

pub fn widget_state_path() -> PathBuf {
    let config_path = config::resolve_config_path_for_write();
    if let Some(parent) = config_path.parent() {
        return parent.join("widget-placement.json");
    }

    PathBuf::from("widget-placement.json")
}

pub fn save_widget_placement(path: &Path, placement: &WidgetPlacement) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let content = serde_json::to_string_pretty(placement).map_err(|err| err.to_string())?;
    fs::write(path, content).map_err(|err| format!("failed to write {}: {}", path.display(), err))
}

pub fn load_widget_placement(path: &Path) -> Option<WidgetPlacement> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn widget_anchor_position(work_area: WorkArea, width: f64, height: f64) -> WindowPlacement {
    let min_x = work_area.x + WIDGET_MIN_VISIBLE_MARGIN;
    let min_y = work_area.y + WIDGET_MIN_VISIBLE_MARGIN;
    let max_x = (work_area.x + work_area.width - width - WIDGET_MIN_VISIBLE_MARGIN).max(min_x);
    let max_y = (work_area.y + work_area.height - height - WIDGET_MIN_VISIBLE_MARGIN).max(min_y);

    WindowPlacement {
        x: (work_area.x + work_area.width - width - WIDGET_MARGIN_X).clamp(min_x, max_x),
        y: (work_area.y + WIDGET_MARGIN_Y).clamp(min_y, max_y),
    }
}

pub fn build_widget_placement(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    work_area: Option<WorkArea>,
) -> WidgetPlacement {
    let Some(work_area) = work_area else {
        return WidgetPlacement {
            x,
            y,
            dock: WidgetDockSide::Free,
            auto_hidden: false,
        };
    };

    let min_x = work_area.x;
    let min_y = work_area.y;
    let max_x = min_x + work_area.width - width;
    let max_y = min_y + work_area.height - height;

    let clamped_x = clamp(x, min_x, max_x);
    let clamped_y = clamp(y, min_y, max_y);

    let nearest = [
        (WidgetDockSide::Left, (clamped_x - min_x).abs()),
        (WidgetDockSide::Right, (max_x - clamped_x).abs()),
        (WidgetDockSide::Top, (clamped_y - min_y).abs()),
        (WidgetDockSide::Bottom, (max_y - clamped_y).abs()),
    ]
    .into_iter()
    .min_by(|left, right| left.1.partial_cmp(&right.1).unwrap())
    .unwrap();

    if nearest.1 > WIDGET_EDGE_THRESHOLD {
        return WidgetPlacement {
            x: clamped_x,
            y: clamped_y,
            dock: WidgetDockSide::Free,
            auto_hidden: false,
        };
    }

    match nearest.0 {
        WidgetDockSide::Left => WidgetPlacement {
            x: min_x - width + WIDGET_PEEK,
            y: clamped_y,
            dock: WidgetDockSide::Left,
            auto_hidden: true,
        },
        WidgetDockSide::Right => WidgetPlacement {
            x: min_x + work_area.width - WIDGET_PEEK,
            y: clamped_y,
            dock: WidgetDockSide::Right,
            auto_hidden: true,
        },
        WidgetDockSide::Top => WidgetPlacement {
            x: clamped_x,
            y: min_y,
            dock: WidgetDockSide::Top,
            auto_hidden: false,
        },
        WidgetDockSide::Bottom => WidgetPlacement {
            x: clamped_x,
            y: max_y,
            dock: WidgetDockSide::Bottom,
            auto_hidden: false,
        },
        WidgetDockSide::Free => WidgetPlacement {
            x: clamped_x,
            y: clamped_y,
            dock: WidgetDockSide::Free,
            auto_hidden: false,
        },
    }
}

pub fn reveal_widget_placement(
    placement: WidgetPlacement,
    width: f64,
    work_area: Option<WorkArea>,
) -> WidgetPlacement {
    let Some(work_area) = work_area else {
        return placement;
    };

    if !placement.auto_hidden {
        return placement;
    }

    let min_x = work_area.x;
    let max_x = min_x + work_area.width - width;

    match placement.dock {
        WidgetDockSide::Left => WidgetPlacement {
            x: min_x + WIDGET_VISIBLE_MARGIN,
            auto_hidden: false,
            ..placement
        },
        WidgetDockSide::Right => WidgetPlacement {
            x: max_x - WIDGET_VISIBLE_MARGIN,
            auto_hidden: false,
            ..placement
        },
        _ => placement,
    }
}

pub fn hide_widget_placement(
    placement: WidgetPlacement,
    width: f64,
    work_area: Option<WorkArea>,
) -> WidgetPlacement {
    let Some(work_area) = work_area else {
        return placement;
    };

    match placement.dock {
        WidgetDockSide::Left => WidgetPlacement {
            x: work_area.x - width + WIDGET_PEEK,
            auto_hidden: true,
            ..placement
        },
        WidgetDockSide::Right => WidgetPlacement {
            x: work_area.x + work_area.width - WIDGET_PEEK,
            auto_hidden: true,
            ..placement
        },
        _ => placement,
    }
}

pub fn restore_widget_placement(
    saved: WidgetPlacement,
    width: f64,
    height: f64,
    work_area: Option<WorkArea>,
) -> WidgetPlacement {
    let mut placement = build_widget_placement(saved.x, saved.y, width, height, work_area);

    if matches!(saved.dock, WidgetDockSide::Left | WidgetDockSide::Right) {
        placement.dock = saved.dock;
        placement.auto_hidden = saved.auto_hidden;
        if saved.auto_hidden {
            placement = hide_widget_placement(placement, width, work_area);
        }
    }

    placement
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}
