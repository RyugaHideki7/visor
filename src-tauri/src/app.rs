pub fn run_app() {
  tauri::Builder::default()
    .setup(|app| {
      crate::logging::setup(app)?;
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
