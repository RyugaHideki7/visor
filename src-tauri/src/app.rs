use tauri::Manager;

pub fn run_app() {
  tauri::Builder::default()
    .setup(|app| {
      crate::logging::setup(app)?;

      // Watcher registry (prevents duplicates and enables stop/start).
      app.manage(crate::stock::WatcherState::new());
      
      let handle = app.handle();
      let handle_clone = handle.clone();
      tauri::async_runtime::block_on(async move {
          let pool = crate::db::init_db(&handle_clone).await.expect("failed to init db");
          handle_clone.manage(crate::db::DbState { pool: pool.clone() });

          // Start watchers for active lines
          let lines = sqlx::query(
              "SELECT id, path, prefix, archived_path FROM lines WHERE active = 1"
          )
          .fetch_all(&pool)
          .await
          .expect("failed to fetch lines");

          for line in lines {
              use sqlx::Row;
              crate::stock::start_watcher(
                  handle_clone.clone(),
                  line.get("id"),
                  line.get("path"),
                  line.get("prefix"),
                  line.get("archived_path"),
              );
          }
      });

      // Tray Setup
      let quit_i = tauri::menu::MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
      let show_i = tauri::menu::MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
      let menu = tauri::menu::Menu::with_items(app, &[&show_i, &quit_i])?;

      let _tray = tauri::tray::TrayIconBuilder::new()
          .icon(app.default_window_icon().unwrap().clone())
          .menu(&menu)
          .show_menu_on_left_click(true)
          .on_menu_event(|app, event| match event.id.as_ref() {
              "quit" => {
                  app.exit(0);
              }
              "show" => {
                  let window = app.get_webview_window("main").unwrap();
                  window.show().unwrap();
                  window.unminimize().unwrap();
                  window.set_focus().unwrap();
              }
              _ => {}
          })
          .build(app)?;

      Ok(())
    })
    .on_window_event(|window, event| {
      if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        window.hide().unwrap();
        api.prevent_close();
      }
    })
    .invoke_handler(tauri::generate_handler![
        crate::commands::get_lines,
        crate::commands::save_line,
        crate::commands::delete_line,
        crate::commands::toggle_line_active,
        crate::commands::start_line_watcher,
        crate::commands::stop_line_watcher,
        crate::commands::get_dashboard_snapshot,
        crate::commands::get_mappings,
        crate::commands::save_mappings,
        crate::commands::get_production_data,
        crate::commands::get_sql_server_config,
        crate::commands::save_sql_server_config,
        crate::commands::get_logs,
        crate::commands::add_log,
        crate::commands::clear_logs
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
