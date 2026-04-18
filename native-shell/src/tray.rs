use std::sync::Arc;
use std::thread;

use tray_icon::menu::{ContextMenu, Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

#[derive(Clone, Copy, Debug)]
pub enum TrayCommand {
    ToggleMainWindow,
    ToggleWidgetWindow,
    ToggleAlwaysOnTop,
    RefreshSignals,
    Quit,
}

pub struct TrayHandles {
    _tray_icon: TrayIcon,
    _menu: Menu,
    _toggle_main: MenuItem,
    _toggle_widget: MenuItem,
    _toggle_pin: MenuItem,
    _refresh: MenuItem,
    _quit: MenuItem,
}

pub struct WidgetMenuHandles {
    menu: Menu,
    _toggle_main: MenuItem,
    _toggle_pin: MenuItem,
    _refresh: MenuItem,
    _quit: MenuItem,
}

pub fn install_tray<F>(dispatch: F) -> Result<TrayHandles, Box<dyn std::error::Error>>
where
    F: Fn(TrayCommand) + Send + Sync + 'static,
{
    let toggle_main = MenuItem::with_id("toggle-main", "Toggle Main Window", true, None);
    let toggle_widget = MenuItem::with_id("toggle-widget", "Toggle Widget", true, None);
    let toggle_pin = MenuItem::with_id("toggle-pin", "Toggle Pin", true, None);
    let refresh = MenuItem::with_id("refresh", "Refresh Signals", true, None);
    let quit = MenuItem::with_id("quit", "Quit", true, None);

    let menu = Menu::new();
    menu.append_items(&[&toggle_main, &toggle_widget, &toggle_pin, &refresh, &quit])?;

    let tray_icon = TrayIconBuilder::new()
        .with_tooltip("Signal Desk Native")
        .with_menu(Box::new(menu.clone()))
        .with_icon(make_icon()?)
        .build()?;

    let dispatch = Arc::new(dispatch);
    let menu_dispatch = dispatch.clone();
    let tray_dispatch = dispatch.clone();

    thread::spawn(move || {
        while let Ok(event) = MenuEvent::receiver().recv() {
            let command = command_from_menu_id(event.id().as_ref());
            if let Some(command) = command {
                menu_dispatch(command);
            }
        }
    });

    thread::spawn(move || {
        while let Ok(event) = TrayIconEvent::receiver().recv() {
            if let Some(command) = command_from_tray_icon_event(&event) {
                tray_dispatch(command);
            }
        }
    });

    Ok(TrayHandles {
        _tray_icon: tray_icon,
        _menu: menu,
        _toggle_main: toggle_main,
        _toggle_widget: toggle_widget,
        _toggle_pin: toggle_pin,
        _refresh: refresh,
        _quit: quit,
    })
}

pub fn create_widget_menu(
    always_on_top: bool,
) -> Result<WidgetMenuHandles, Box<dyn std::error::Error>> {
    let toggle_main = MenuItem::with_id("toggle-main", "Open Main Window", true, None);
    let toggle_pin = MenuItem::with_id(
        "toggle-pin",
        if always_on_top {
            "Unpin Window"
        } else {
            "Pin Window"
        },
        true,
        None,
    );
    let refresh = MenuItem::with_id("refresh", "Refresh Signals", true, None);
    let quit = MenuItem::with_id("quit", "Quit", true, None);

    let menu = Menu::new();
    menu.append_items(&[&toggle_main, &toggle_pin, &refresh, &quit])?;

    Ok(WidgetMenuHandles {
        menu,
        _toggle_main: toggle_main,
        _toggle_pin: toggle_pin,
        _refresh: refresh,
        _quit: quit,
    })
}

impl WidgetMenuHandles {
    #[cfg(target_os = "windows")]
    pub unsafe fn show_for_hwnd(&self, hwnd: isize) {
        let _ = self.menu.show_context_menu_for_hwnd(hwnd, None);
    }
}

pub fn command_from_menu_id(id: &str) -> Option<TrayCommand> {
    match id {
        "toggle-main" => Some(TrayCommand::ToggleMainWindow),
        "toggle-widget" => Some(TrayCommand::ToggleWidgetWindow),
        "toggle-pin" => Some(TrayCommand::ToggleAlwaysOnTop),
        "refresh" => Some(TrayCommand::RefreshSignals),
        "quit" => Some(TrayCommand::Quit),
        _ => None,
    }
}

fn command_from_tray_icon_event(event: &TrayIconEvent) -> Option<TrayCommand> {
    match event {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } => Some(TrayCommand::ToggleMainWindow),
        _ => None,
    }
}

fn make_icon() -> Result<Icon, tray_icon::BadIcon> {
    let size = 32;
    let mut rgba = vec![0_u8; size * size * 4];

    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            let edge = x < 3 || y < 3 || x > size - 4 || y > size - 4;
            let (r, g, b) = if edge { (15, 23, 42) } else { (37, 99, 235) };
            rgba[idx] = r;
            rgba[idx + 1] = g;
            rgba[idx + 2] = b;
            rgba[idx + 3] = 255;
        }
    }

    Icon::from_rgba(rgba, size as u32, size as u32)
}
