use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;

use crate::app_state::{AppState, UiSnapshot};
use crate::runtime::RuntimeHandles;
use crate::tray::{TrayCommand, TrayHandles};
use crate::widget_state::{
    build_widget_placement, hide_widget_placement, load_widget_placement, restore_widget_placement,
    reveal_widget_placement, save_widget_placement, widget_anchor_position, widget_state_path,
    WidgetDockSide, WidgetPlacement, WorkArea,
};
use slint::winit_030::{winit, WinitWindowAccessor};
use slint::{
    CloseRequestResponse, ComponentHandle, LogicalPosition, SharedString, Timer, TimerMode,
    VecModel, Weak,
};

#[cfg(target_os = "windows")]
use slint::winit_030::winit::platform::windows::WindowAttributesExtWindows;
#[cfg(target_os = "windows")]
use slint::winit_030::winit::platform::windows::WindowExtWindows;
#[cfg(target_os = "windows")]
use slint::winit_030::winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

slint::include_modules!();

type SharedAppState = Arc<Mutex<AppState>>;

#[derive(Default)]
struct WidgetControllerState {
    current_placement: Option<WidgetPlacement>,
    last_observed_position: Option<(i32, i32)>,
    stable_polls: u8,
}

struct WidgetAnimationState {
    from: WidgetPlacement,
    to: WidgetPlacement,
    started_at: Instant,
    duration: Duration,
    persist: bool,
}

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
            TrayCommand::ToggleAlwaysOnTop => {
                let next = {
                    let state = self.state.lock().expect("state poisoned");
                    !state.always_on_top()
                };
                self.runtime.request_set_always_on_top(next);
            }
            TrayCommand::RefreshSignals => self.runtime.request_refresh(),
            TrayCommand::Quit => {
                self.runtime.request_quit();
                let _ = slint::quit_event_loop();
            }
        }
    }

    fn toggle_signal_read_at(&self, index: i32) {
        if index < 0 {
            return;
        }

        let toggle_action = {
            let mut state = self.state.lock().expect("state poisoned");
            state.toggle_signal_row_at(index as usize)
        };

        if let Some((signal_key, read)) = toggle_action {
            self.refresh_ui();
            self.runtime.request_mark_read(signal_key, read);
        }
    }

    fn show_widget_menu(&self) {
        if let Some(widget_window) = self.widget_window.upgrade() {
            let always_on_top = {
                let state = self.state.lock().expect("state poisoned");
                state.always_on_top()
            };
            #[cfg(target_os = "windows")]
            {
                let _ = widget_window.window().with_winit_window(
                    |window: &winit::window::Window| unsafe {
                        if let Ok(menu) = crate::tray::create_widget_menu(always_on_top) {
                            if let Ok(handle) = window.window_handle() {
                                if let RawWindowHandle::Win32(win32) = handle.as_raw() {
                                    menu.show_for_hwnd(win32.hwnd.get() as isize);
                                }
                            }
                        }
                    },
                );
            }
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    let widget_owner = Rc::new(Cell::new(None::<isize>));
    configure_winit_backend(
        #[cfg(target_os = "windows")]
        widget_owner.clone(),
    )?;

    let initial_snapshot = crate::runtime::load_runtime_snapshot();
    let state = Arc::new(Mutex::new(AppState::new(initial_snapshot)));

    let main_window = MainWindow::new()?;
    apply_snapshot_to_main(&main_window, &state.lock().expect("state poisoned").snapshot());
    main_window.show()?;
    #[cfg(target_os = "windows")]
    remember_main_window_handle(&main_window, &widget_owner);

    let widget_window = WidgetWindow::new()?;

    let runtime_state = state.clone();
    let runtime_ui_handle = main_window.as_weak();
    let runtime_widget_handle = widget_window.as_weak();
    let runtime = crate::runtime::spawn_runtime_loop(
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

    let bridge = UiBridge::new(
        main_window.as_weak(),
        widget_window.as_weak(),
        state,
        runtime,
    );
    let widget_controller = Rc::new(RefCell::new(WidgetControllerState::default()));

    wire_main_window(&main_window, bridge.clone());
    wire_widget_window(&widget_window, bridge.clone(), widget_controller.clone());

    let _tray = install_tray_bridge(bridge.clone())?;

    bridge.refresh_ui();
    widget_window.show()?;
    configure_widget_window(&widget_window);
    position_widget_window(&widget_window, &widget_controller);

    slint::run_event_loop_until_quit()?;
    Ok(())
}

fn configure_winit_backend(
    #[cfg(target_os = "windows")] widget_owner: Rc<Cell<Option<isize>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut backend = slint::BackendSelector::new().backend_name("winit".into());

    #[cfg(target_os = "windows")]
    {
        backend = backend.with_winit_window_attributes_hook(move |attributes| {
            if let Some(hwnd) = widget_owner.get() {
                attributes.with_owner_window(hwnd as _).with_skip_taskbar(true)
            } else {
                attributes
            }
        });
    }

    backend.select()?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn remember_main_window_handle(main_window: &MainWindow, widget_owner: &Rc<Cell<Option<isize>>>) {
    let _ = main_window
        .window()
        .with_winit_window(|window: &winit::window::Window| {
            if let Ok(handle) = window.window_handle() {
                if let RawWindowHandle::Win32(win32) = handle.as_raw() {
                    widget_owner.set(Some(win32.hwnd.get() as isize));
                }
            }
        });
}

fn install_tray_bridge(bridge: UiBridge) -> Result<TrayHandles, Box<dyn std::error::Error>> {
    crate::tray::install_tray(move |command| {
        let bridge = bridge.clone();
        let _ = slint::invoke_from_event_loop(move || {
            bridge.handle_tray_command(command);
        });
    })
}

fn wire_main_window(main_window: &MainWindow, bridge: UiBridge) {
    let pin_bridge = bridge.clone();
    main_window.on_toggle_always_on_top(move || {
        let next = {
            let state = pin_bridge.state.lock().expect("state poisoned");
            !state.always_on_top()
        };
        pin_bridge.runtime.request_set_always_on_top(next);
    });

    let edge_bridge = bridge.clone();
    main_window.on_toggle_edge_mode(move || {
        let next = {
            let state = edge_bridge.state.lock().expect("state poisoned");
            !state.snapshot().edge_mode
        };
        edge_bridge.runtime.request_set_edge_mode(next);
    });

    let width_bridge = bridge.clone();
    main_window.on_adjust_edge_width(move |delta| {
        let current = {
            let state = width_bridge.state.lock().expect("state poisoned");
            state.snapshot().edge_width_label
        };
        let parsed = current
            .split_whitespace()
            .next()
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(120.0);
        width_bridge
            .runtime
            .request_set_edge_width((parsed + delta as f64).clamp(160.0, 480.0));
    });

    let notifications_bridge = bridge.clone();
    main_window.on_toggle_notifications(move || {
        let next = {
            let state = notifications_bridge.state.lock().expect("state poisoned");
            !state.snapshot().notifications_enabled
        };
        notifications_bridge.runtime.request_set_notifications(next);
    });

    let sound_bridge = bridge.clone();
    main_window.on_toggle_sound(move || {
        let next = {
            let state = sound_bridge.state.lock().expect("state poisoned");
            !state.snapshot().sound_enabled
        };
        sound_bridge.runtime.request_set_sound(next);
    });

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

    let save_bridge = bridge.clone();
    main_window.on_save_config(move || {
        save_bridge.runtime.request_save_config();
    });

    let quit_bridge = bridge.clone();
    main_window.on_quit_app(move || {
        quit_bridge.runtime.request_quit();
        let _ = slint::quit_event_loop();
    });

    let toggle_signal_bridge = bridge.clone();
    main_window.on_toggle_signal_read(move |index| {
        toggle_signal_bridge.toggle_signal_read_at(index);
    });

    let close_bridge = bridge;
    main_window.window().on_close_requested(move || {
        close_bridge.mark_main_hidden();
        CloseRequestResponse::HideWindow
    });
}

fn wire_widget_window(
    widget_window: &WidgetWindow,
    bridge: UiBridge,
    widget_controller: Rc<RefCell<WidgetControllerState>>,
) {
    let open_bridge = bridge.clone();
    widget_window.on_open_main(move || {
        open_bridge.set_main_visibility(true);
    });

    let drag_window_handle = widget_window.as_weak();
    let drag_controller = widget_controller.clone();
    let drag_timer = Rc::new(Timer::default());
    let animate_timer = Rc::new(Timer::default());
    let animation_state = Rc::new(RefCell::new(None::<WidgetAnimationState>));
    let hover_hide_timer = Rc::new(Timer::default());
    let drag_animate_timer = animate_timer.clone();
    let drag_animation_state = animation_state.clone();
    widget_window.on_start_widget_drag(move || {
        {
            let mut controller = drag_controller.borrow_mut();
            controller.last_observed_position = None;
            controller.stable_polls = 0;
        }

        if let Some(widget_window) = drag_window_handle.upgrade() {
            let _ = widget_window
                .window()
                .with_winit_window(|window: &winit::window::Window| {
                    let _ = window.drag_window();
                });

            let timer_window = drag_window_handle.clone();
            let timer_controller = drag_controller.clone();
            let timer_handle = drag_timer.clone();
            let timer_animate = drag_animate_timer.clone();
            let timer_animation_state = drag_animation_state.clone();
            drag_timer.start(TimerMode::Repeated, Duration::from_millis(120), move || {
                let Some(widget_window) = timer_window.upgrade() else {
                    timer_handle.stop();
                    return;
                };

                let Some(geometry) = resolve_widget_geometry(&widget_window) else {
                    return;
                };

                let current_position = (
                    geometry.position.x.round() as i32,
                    geometry.position.y.round() as i32,
                );

                let mut should_snap = false;
                {
                    let mut controller = timer_controller.borrow_mut();
                    if controller.last_observed_position == Some(current_position) {
                        controller.stable_polls = controller.stable_polls.saturating_add(1);
                        should_snap = controller.stable_polls >= 2;
                    } else {
                        controller.last_observed_position = Some(current_position);
                        controller.stable_polls = 0;
                    }
                }

                if should_snap {
                    let placement = build_widget_placement(
                        geometry.position.x,
                        geometry.position.y,
                        geometry.size.width,
                        geometry.size.height,
                        geometry.work_area,
                    );
                    animate_widget_placement(
                        &widget_window,
                        &timer_controller,
                        &timer_animate,
                        &timer_animation_state,
                        placement,
                        true,
                    );
                    timer_handle.stop();
                }
            });
        }
    });

    let hover_reveal_window = widget_window.as_weak();
    let hover_reveal_controller = widget_controller.clone();
    let hover_reveal_timer = hover_hide_timer.clone();
    let hover_reveal_animate_timer = animate_timer.clone();
    let hover_reveal_animation_state = animation_state.clone();
    widget_window.on_hover_started(move || {
        hover_reveal_timer.stop();

        let Some(widget_window) = hover_reveal_window.upgrade() else {
            return;
        };
        let Some(geometry) = resolve_widget_geometry(&widget_window) else {
            return;
        };
        let current = hover_reveal_controller.borrow().current_placement;
        if let Some(current) = current {
            let revealed =
                reveal_widget_placement(current, geometry.size.width, geometry.work_area);
            if revealed != current {
                animate_widget_placement(
                    &widget_window,
                    &hover_reveal_controller,
                    &hover_reveal_animate_timer,
                    &hover_reveal_animation_state,
                    revealed,
                    true,
                );
            }
        }
    });

    let hover_hide_window = widget_window.as_weak();
    let hover_hide_controller = widget_controller.clone();
    let hover_hide_timer_handle = hover_hide_timer.clone();
    let hover_hide_animate_timer = animate_timer.clone();
    let hover_hide_animation_state = animation_state.clone();
    widget_window.on_hover_ended(move || {
        if hover_hide_window.upgrade().is_none() {
            return;
        }

        let timer_window = hover_hide_window.clone();
        let timer_controller = hover_hide_controller.clone();
        let timer_handle = hover_hide_timer_handle.clone();
        let timer_animate = hover_hide_animate_timer.clone();
        let timer_animation_state = hover_hide_animation_state.clone();
        hover_hide_timer_handle.start(
            TimerMode::SingleShot,
            Duration::from_millis(520),
            move || {
                let Some(widget_window) = timer_window.upgrade() else {
                    return;
                };
                let Some(geometry) = resolve_widget_geometry(&widget_window) else {
                    return;
                };
                let current = timer_controller.borrow().current_placement;
                if let Some(current) = current {
                    let hidden =
                        hide_widget_placement(current, geometry.size.width, geometry.work_area);
                    if hidden != current {
                        animate_widget_placement(
                            &widget_window,
                            &timer_controller,
                            &timer_animate,
                            &timer_animation_state,
                            hidden,
                            true,
                        );
                    }
                }
                timer_handle.stop();
            },
        );
    });

    let widget_menu_bridge = bridge.clone();
    let widget_menu_window = widget_window.as_weak();
    let widget_menu_controller = widget_controller.clone();
    let widget_menu_hide_timer = hover_hide_timer.clone();
    let widget_menu_animate_timer = animate_timer.clone();
    let widget_menu_animation_state = animation_state.clone();
    widget_window.on_show_widget_menu(move || {
        widget_menu_hide_timer.stop();
        if let Some(widget_window) = widget_menu_window.upgrade() {
            if let Some(geometry) = resolve_widget_geometry(&widget_window) {
                if let Some(current) = widget_menu_controller.borrow().current_placement {
                    let revealed =
                        reveal_widget_placement(current, geometry.size.width, geometry.work_area);
                    if revealed != current {
                        animate_widget_placement(
                            &widget_window,
                            &widget_menu_controller,
                            &widget_menu_animate_timer,
                            &widget_menu_animation_state,
                            revealed,
                            true,
                        );
                    }
                }
            }
        }
        widget_menu_bridge.show_widget_menu();
    });

    let refresh_bridge = bridge.clone();
    let refresh_hide_timer = hover_hide_timer.clone();
    widget_window.on_refresh_data(move || {
        refresh_hide_timer.stop();
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
    main_window.set_save_hint(SharedString::from(snapshot.save_hint.as_str()));
    main_window.set_connection_label(SharedString::from(snapshot.connection_label.as_str()));
    main_window.set_connection_tone(SharedString::from(snapshot.connection_tone.as_str()));
    main_window.set_stats_primary(SharedString::from(snapshot.stats_primary.as_str()));
    main_window.set_stats_secondary(SharedString::from(snapshot.stats_secondary.as_str()));
    main_window.set_stats_tertiary(SharedString::from(snapshot.stats_tertiary.as_str()));
    main_window.set_stats_quaternary(SharedString::from(snapshot.stats_quaternary.as_str()));
    main_window.set_edge_width_label(SharedString::from(snapshot.edge_width_label.as_str()));
    let signal_rows: Vec<SignalRowData> = snapshot
        .signal_rows
        .iter()
        .map(|row| SignalRowData {
            title: SharedString::from(row.title.as_str()),
            meta: SharedString::from(row.meta.as_str()),
            is_header: row.is_header,
            unread: row.unread,
            pending: row.pending,
            unread_count: row.unread_count,
            timeline_visible: row.timeline_visible,
            timeline_ratio: row.timeline_ratio,
            timeline_positive: row.timeline_positive,
        })
        .collect();
    main_window.set_signal_rows(VecModel::from_slice(&signal_rows));
    main_window.set_main_visible(snapshot.main_visible);
    main_window.set_widget_visible(snapshot.widget_visible);
    main_window.set_pin_enabled(snapshot.always_on_top);
    main_window.set_edge_mode(snapshot.edge_mode);
    main_window.set_notifications_enabled(snapshot.notifications_enabled);
    main_window.set_sound_enabled(snapshot.sound_enabled);
    let _ = main_window
        .window()
        .with_winit_window(|window: &winit::window::Window| {
            window.set_window_level(if snapshot.always_on_top {
                winit::window::WindowLevel::AlwaysOnTop
            } else {
                winit::window::WindowLevel::Normal
            });
        });
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

fn position_widget_window(
    widget_window: &WidgetWindow,
    widget_controller: &Rc<RefCell<WidgetControllerState>>,
) {
    let placement = resolve_widget_geometry(widget_window)
        .map(|geometry| {
            let state_path = widget_state_path();
            if let Some(saved) = load_widget_placement(&state_path) {
                restore_widget_placement(
                    saved,
                    geometry.size.width,
                    geometry.size.height,
                    geometry.work_area,
                )
            } else {
                let anchor = geometry
                    .work_area
                    .map(|work_area| {
                        let anchor = widget_anchor_position(
                            work_area,
                            geometry.size.width,
                            geometry.size.height,
                        );
                        WidgetPlacement {
                            x: anchor.x,
                            y: anchor.y,
                            dock: WidgetDockSide::Free,
                            auto_hidden: false,
                        }
                    })
                    .unwrap_or(WidgetPlacement {
                        x: geometry.position.x,
                        y: geometry.position.y,
                        dock: WidgetDockSide::Free,
                        auto_hidden: false,
                    });
                anchor
            }
        })
        .unwrap_or(WidgetPlacement {
            x: 0.0,
            y: 0.0,
            dock: WidgetDockSide::Free,
            auto_hidden: false,
        });

    apply_widget_placement(widget_window, widget_controller, placement, false);
}

struct WidgetGeometry {
    position: LogicalPoint,
    size: LogicalSize,
    work_area: Option<WorkArea>,
}

struct LogicalPoint {
    x: f64,
    y: f64,
}

struct LogicalSize {
    width: f64,
    height: f64,
}

fn resolve_widget_geometry(widget_window: &WidgetWindow) -> Option<WidgetGeometry> {
    widget_window
        .window()
        .with_winit_window(|window: &winit::window::Window| {
            let scale = window.scale_factor();
            let position = window.outer_position().ok()?;
            let size = window.outer_size();
            let work_area = window.current_monitor().map(|monitor| {
                let monitor_scale = monitor.scale_factor();
                let monitor_position = monitor.position();
                let monitor_size = monitor.size();
                WorkArea {
                    x: monitor_position.x as f64 / monitor_scale,
                    y: monitor_position.y as f64 / monitor_scale,
                    width: monitor_size.width as f64 / monitor_scale,
                    height: monitor_size.height as f64 / monitor_scale,
                }
            });

            Some(WidgetGeometry {
                position: LogicalPoint {
                    x: position.x as f64 / scale,
                    y: position.y as f64 / scale,
                },
                size: LogicalSize {
                    width: size.width as f64 / scale,
                    height: size.height as f64 / scale,
                },
                work_area,
            })
        })
        .flatten()
}

fn apply_widget_placement(
    widget_window: &WidgetWindow,
    widget_controller: &Rc<RefCell<WidgetControllerState>>,
    placement: WidgetPlacement,
    persist: bool,
) {
    widget_controller.borrow_mut().current_placement = Some(placement);
    let _ = widget_window
        .window()
        .set_position(LogicalPosition::new(placement.x as f32, placement.y as f32));

    if persist {
        let _ = save_widget_placement(&widget_state_path(), &placement);
    }
}

fn animate_widget_placement(
    widget_window: &WidgetWindow,
    widget_controller: &Rc<RefCell<WidgetControllerState>>,
    animate_timer: &Rc<Timer>,
    animation_state: &Rc<RefCell<Option<WidgetAnimationState>>>,
    target: WidgetPlacement,
    persist: bool,
) {
    let Some(geometry) = resolve_widget_geometry(widget_window) else {
        apply_widget_placement(widget_window, widget_controller, target, persist);
        return;
    };

    let current = widget_controller
        .borrow()
        .current_placement
        .unwrap_or(WidgetPlacement {
            x: geometry.position.x,
            y: geometry.position.y,
            dock: target.dock,
            auto_hidden: target.auto_hidden,
        });

    if (current.x - target.x).abs() < 0.5 && (current.y - target.y).abs() < 0.5 {
        apply_widget_placement(widget_window, widget_controller, target, persist);
        return;
    }

    widget_controller.borrow_mut().current_placement = Some(target);
    animate_timer.stop();
    *animation_state.borrow_mut() = Some(WidgetAnimationState {
        from: WidgetPlacement {
            x: geometry.position.x,
            y: geometry.position.y,
            ..current
        },
        to: target,
        started_at: Instant::now(),
        duration: Duration::from_millis(180),
        persist,
    });

    let timer_window = widget_window.as_weak();
    let timer_controller = widget_controller.clone();
    let timer_handle = animate_timer.clone();
    let timer_state = animation_state.clone();
    animate_timer.start(TimerMode::Repeated, Duration::from_millis(16), move || {
        let Some(widget_window) = timer_window.upgrade() else {
            timer_handle.stop();
            *timer_state.borrow_mut() = None;
            return;
        };

        let (from, to, started_at, duration, persist) = {
            let state_ref = timer_state.borrow();
            let Some(animation) = state_ref.as_ref() else {
                timer_handle.stop();
                return;
            };
            (
                animation.from,
                animation.to,
                animation.started_at,
                animation.duration,
                animation.persist,
            )
        };

        let elapsed = started_at.elapsed();
        let progress = (elapsed.as_secs_f64() / duration.as_secs_f64()).min(1.0);
        let next = interpolate_widget_placement(from, to, progress);
        let _ = widget_window
            .window()
            .set_position(LogicalPosition::new(next.x as f32, next.y as f32));

        if progress >= 1.0 {
            let final_target = to;
            timer_handle.stop();
            *timer_state.borrow_mut() = None;
            if persist {
                let _ = save_widget_placement(&widget_state_path(), &final_target);
            }
            timer_controller.borrow_mut().current_placement = Some(final_target);
        }
    });
}

pub fn interpolate_widget_placement(
    from: WidgetPlacement,
    to: WidgetPlacement,
    progress: f64,
) -> WidgetPlacement {
    let eased = ease_out_cubic(progress.clamp(0.0, 1.0));
    WidgetPlacement {
        x: from.x + (to.x - from.x) * eased,
        y: from.y + (to.y - from.y) * eased,
        dock: to.dock,
        auto_hidden: to.auto_hidden,
    }
}

fn ease_out_cubic(progress: f64) -> f64 {
    1.0 - (1.0 - progress).powi(3)
}
