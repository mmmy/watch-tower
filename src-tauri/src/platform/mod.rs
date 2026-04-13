#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

use tauri::WebviewWindow;

pub fn default_click_through_support() -> (bool, Option<String>) {
    #[cfg(target_os = "windows")]
    {
        return (true, None);
    }

    #[cfg(target_os = "macos")]
    {
        return (true, None);
    }

    #[allow(unreachable_code)]
    (
        false,
        Some("Passive click-through is not enabled on this platform build.".into()),
    )
}

pub fn apply_click_through(window: &WebviewWindow, enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        return windows::apply_click_through(window, enabled);
    }

    #[cfg(target_os = "macos")]
    {
        return macos::apply_click_through(window, enabled);
    }

    #[allow(unreachable_code)]
    if enabled {
        Err("Passive click-through is not enabled on this platform build.".into())
    } else {
        Ok(())
    }
}
