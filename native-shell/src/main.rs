#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod runtime;
mod tray;

use std::sync::{Arc, Mutex};

use app_state::{AppState, UiSnapshot};
use slint::{CloseRequestResponse, ComponentHandle, LogicalPosition, SharedString, VecModel, Weak};
use slint::winit_030::{winit, WinitWindowAccessor};
use runtime::RuntimeHandles;
use tray::{TrayCommand, TrayHandles};

#[cfg(target_os = "windows")]
use slint::winit_030::winit::platform::windows::WindowExtWindows;
#[cfg(target_os = "windows")]
use slint::winit_030::winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

slint::include_modules!();

type SharedAppState = Arc<Mutex<AppState>>;

#[derive(Clone)]
struct UiBridge {
    main_window: Weak<MainWindow>,
    widget_window: Weak<WidgetWindow>,
    state: SharedAppState,
    runtime: RuntimeHandles,
}

impl UiBridge {
    fn new(
        main_window: Weak<MainWindow>,
        widget_window: Weak<WidgetWindow>,
        state: SharedAppState,
        runtime: RuntimeHandles,
    ) -> Self {
        Self {
            main_window,
            widget_window,
            state,
            runtime,
        }
    }

    fn refresh_ui(&self) {
        let snapshot = self.state.lock().expect("state poisoned").snapshot();

        if let Some(main_window) = self.main_window.upgrade() {
            apply_snapshot_to_main(&main_window, &snapshot);
        }

        if let Some(widget_window) = self.widget_window.upgrade() {
            apply_snapshot_to_widget(&widget_window, &snapshot);
        }
    }

    fn set_main_visibility(&self, visible: bool) {
        {
            let mut state = self.state.lock().expect("state poisoned");
            state.set_main_visible(visible);
        }

        if let Some(main_window) = self.main_window.upgrade() {
            if visible {
                let _ = main_window.show();
                let _ = main_window
                    .window()
                    .with_winit_window(|window: &winit::window::Window| {
                        window.request_redraw();
                    });
            } else {
                let _ = main_window.hide();
            }
        }

        self.refresh_ui();
    }

    fn set_widget_visibility(&self, visible: bool) {
        {
            let mut state = self.state.lock().expect("state poisoned");
            state.set_widget_visible(visible);
        }

        if let Some(widget_window) = self.widget_window.upgrade() {
            if visible {
                let _ = widget_window.show();
            } else {
                let _ = widget_window.hide();
            }
        }

        self.refresh_ui();
    }

    fn toggle_main(&self) {
        let visible = {
            let mut state = self.state.lock().expect("state poisoned");
            state.toggle_main_visible()
        };

        self.set_main_visibility(visible);
    }

    fn toggle_widget(&self) {
        let visible = {
            let mut state = self.state.lock().expect("state poisoned");
            state.toggle_widget_visible()
        };

        self.set_widget_visibility(visible);
    }

    fn mark_main_hidden(&self) {
        {
            let mut state = self.state.lock().expect("state poisoned");
            state.set_main_visible(false);
        }

        self.refresh_ui();
    }

    fn mark_widget_hidden(&self) {
        {
            let mut state = self.state.lock().expect("state poisoned");
            state.set_widget_visible(false);
        }

        self.refresh_ui();
    }

    fn handle_tray_command(&self, command: TrayCommand) {
        match command {
            TrayCommand::ToggleMainWindow => self.toggle_main(),
            TrayCommand::ToggleWidgetWindow => self.toggle_widget(),
            TrayCommand::RefreshSignals => self.runtime.request_refresh(),
            TrayCommand::Quit => {
                self.runtime.request_quit();
                let _ = slint::quit_event_loop();
            }
        }
    }

    fn mark_signal_read_at(&self, index: i32) {
        if index < 0 {
            return;
        }

        let signal_key = {
            let mut state = self.state.lock().expect("state poisoned");
            let signal_key = state.signal_key_at(index as usize);
            if let Some(ref key) = signal_key {
                state.set_pending_mark_read(key.clone());
            }
            signal_key
        };

        if let Some(signal_key) = signal_key {
            self.refresh_ui();
            self.runtime.request_mark_read(signal_key, true);
        }
    }

    fn show_widget_menu(&self) {
        if let Some(widget_window) = self.widget_window.upgrade() {
            #[cfg(target_os = "windows")]
            {
                let _ = widget_window
                    .window()
                    .with_winit_window(|window: &winit::window::Window| unsafe {
                        if let Ok(menu) = tray::create_widget_menu() {
                            if let Ok(handle) = window.window_handle() {
                                if let RawWindowHandle::Win32(win32) = handle.as_raw() {
                                    menu.show_for_hwnd(win32.hwnd.get() as isize);
                                }
                            }
                        }
                    });
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let initial_snapshot = runtime::load_runtime_snapshot();
    let state = Arc::new(Mutex::new(AppState::new(initial_snapshot)));

    let main_window = MainWindow::new()?;
    let widget_window = WidgetWindow::new()?;

    let runtime_state = state.clone();
    let runtime_ui_handle = main_window.as_weak();
    let runtime_widget_handle = widget_window.as_weak();
    let runtime = runtime::spawn_runtime_loop(
        move |snapshot| {
            let runtime_state = runtime_state.clone();
            let runtime_ui_handle = runtime_ui_handle.clone();
            let runtime_widget_handle = runtime_widget_handle.clone();
            let _ = slint::invoke_from_event_loop(move || {
                {
                    let mut guard = runtime_state.lock().expect("state poisoned");
                    guard.update_runtime_snapshot(snapshot);
                }

                let snapshot = runtime_state.lock().expect("state poisoned").snapshot();
                if let Some(main_window) = runtime_ui_handle.upgrade() {
                    apply_snapshot_to_main(&main_window, &snapshot);
                }
                if let Some(widget_window) = runtime_widget_handle.upgrade() {
                    apply_snapshot_to_widget(&widget_window, &snapshot);
                }
            });
        },
        {
            let runtime_state = state.clone();
            let runtime_ui_handle = main_window.as_weak();
            let runtime_widget_handle = widget_window.as_weak();
            move |error| {
                let runtime_state = runtime_state.clone();
                let runtime_ui_handle = runtime_ui_handle.clone();
                let runtime_widget_handle = runtime_widget_handle.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    {
                        let mut guard = runtime_state.lock().expect("state poisoned");
                        guard.set_runtime_error(error);
                    }

                    let snapshot = runtime_state.lock().expect("state poisoned").snapshot();
                    if let Some(main_window) = runtime_ui_handle.upgrade() {
                        apply_snapshot_to_main(&main_window, &snapshot);
                    }
                    if let Some(widget_window) = runtime_widget_handle.upgrade() {
                        apply_snapshot_to_widget(&widget_window, &snapshot);
                    }
                });
            }
        },
    );

    let bridge = UiBridge::new(main_window.as_weak(), widget_window.as_weak(), state, runtime);

    wire_main_window(&main_window, bridge.clone());
    wire_widget_window(&widget_window, bridge.clone());

    let _tray = install_tray_bridge(bridge.clone())?;

    bridge.refresh_ui();
    main_window.show()?;
    widget_window.show()?;
    configure_widget_window(&widget_window);
    let _ = widget_window
        .window()
        .set_position(LogicalPosition::new(1320.0, 120.0));

    slint::run_event_loop_until_quit()?;
    Ok(())
}

fn install_tray_bridge(bridge: UiBridge) -> Result<TrayHandles, Box<dyn std::error::Error>> {
    tray::install_tray(move |command| {
        let bridge = bridge.clone();
        let _ = slint::invoke_from_event_loop(move || {
            bridge.handle_tray_command(command);
        });
    })
}

fn wire_main_window(main_window: &MainWindow, bridge: UiBridge) {
    let refresh_bridge = bridge.clone();
    main_window.on_refresh_data(move || {
        refresh_bridge.runtime.request_refresh();
    });

    let widget_bridge = bridge.clone();
    main_window.on_toggle_widget(move || {
        widget_bridge.toggle_widget();
    });

    let hide_bridge = bridge.clone();
    main_window.on_hide_main(move || {
        hide_bridge.set_main_visibility(false);
    });

    let mark_read_bridge = bridge.clone();
    main_window.on_activate_signal(move |index| {
        mark_read_bridge.mark_signal_read_at(index);
    });

    let close_bridge = bridge;
    main_window.window().on_close_requested(move || {
        close_bridge.mark_main_hidden();
        CloseRequestResponse::HideWindow
    });
}

fn wire_widget_window(widget_window: &WidgetWindow, bridge: UiBridge) {
    let open_bridge = bridge.clone();
    widget_window.on_open_main(move || {
        open_bridge.set_main_visibility(true);
    });

    let drag_window_handle = widget_window.as_weak();
    widget_window.on_start_widget_drag(move || {
        if let Some(widget_window) = drag_window_handle.upgrade() {
            let _ = widget_window.window().with_winit_window(|window: &winit::window::Window| {
                let _ = window.drag_window();
            });
        }
    });

    let widget_menu_bridge = bridge.clone();
    widget_window.on_show_widget_menu(move || {
        widget_menu_bridge.show_widget_menu();
    });

    let refresh_bridge = bridge.clone();
    widget_window.on_refresh_data(move || {
        refresh_bridge.runtime.request_refresh();
    });

    let close_bridge = bridge;
    widget_window.window().on_close_requested(move || {
        close_bridge.mark_widget_hidden();
        CloseRequestResponse::HideWindow
    });
}

fn apply_snapshot_to_main(main_window: &MainWindow, snapshot: &UiSnapshot) {
    main_window.set_unread_count(snapshot.unread_count);
    main_window.set_status_text(SharedString::from(snapshot.status_text.as_str()));
    main_window.set_refresh_label(SharedString::from(snapshot.refresh_label.as_str()));
    main_window.set_runtime_summary(SharedString::from(snapshot.runtime_summary.as_str()));
    main_window.set_stats_primary(SharedString::from(snapshot.stats_primary.as_str()));
    main_window.set_stats_secondary(SharedString::from(snapshot.stats_secondary.as_str()));
    main_window.set_stats_tertiary(SharedString::from(snapshot.stats_tertiary.as_str()));
    main_window.set_stats_quaternary(SharedString::from(snapshot.stats_quaternary.as_str()));
    let signal_rows: Vec<SignalRowData> = snapshot
        .signal_rows
        .iter()
        .map(|row| SignalRowData {
            title: SharedString::from(row.title.as_str()),
            meta: SharedString::from(row.meta.as_str()),
            is_header: row.is_header,
            unread: row.unread,
            pending: row.pending,
        })
        .collect();
    main_window.set_signal_rows(VecModel::from_slice(&signal_rows));
    main_window.set_main_visible(snapshot.main_visible);
    main_window.set_widget_visible(snapshot.widget_visible);
}

fn apply_snapshot_to_widget(widget_window: &WidgetWindow, snapshot: &UiSnapshot) {
    widget_window.set_unread_count(snapshot.unread_count);
    widget_window.set_status_text(SharedString::from(snapshot.status_text.as_str()));
    widget_window.set_widget_visible(snapshot.widget_visible);
}

fn configure_widget_window(widget_window: &WidgetWindow) {
    #[cfg(target_os = "windows")]
    {
        let _ = widget_window
            .window()
            .with_winit_window(|window: &winit::window::Window| {
                window.set_skip_taskbar(true);
            });
    }
}
