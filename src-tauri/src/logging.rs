pub fn setup(app: &tauri::App) -> tauri::Result<()> {
  if cfg!(debug_assertions) {
    app.handle().plugin(
      tauri_plugin_log::Builder::default()
        .level(log::LevelFilter::Info)
        .build(),
    )?;
  }

  Ok(())
}
