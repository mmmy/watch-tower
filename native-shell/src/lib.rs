pub mod api_client;
pub mod app_state;
pub mod config;
pub mod hooks;
pub mod main_window_state;
pub mod notifications;
pub mod runtime;
pub mod shell;
pub mod signal_time;
pub mod tray;
pub mod widget_state;

pub use shell::run;
