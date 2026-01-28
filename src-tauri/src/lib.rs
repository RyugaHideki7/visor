#[cfg_attr(mobile, tauri::mobile_entry_point)]
mod app;
mod logging;

pub fn run() {
  app::run_app();
}
