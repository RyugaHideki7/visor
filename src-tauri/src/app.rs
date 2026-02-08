use tauri::Manager;
use tauri_plugin_opener;
use tauri_plugin_process;
use tauri_plugin_single_instance::init as single_instance_init;
use tauri_plugin_updater;

pub fn run_app() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(single_instance_init(|app, _args, _cwd| {
            // On second launch, focus existing window instead of spawning another instance.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            crate::logging::setup(app)?;

            // Watcher registry (prevents duplicates and enables stop/start).
            app.manage(crate::stock::WatcherState::new());

            let handle = app.handle();
            let handle_clone = handle.clone();
            tauri::async_runtime::block_on(async move {
                let pool = crate::db::init_db(&handle_clone)
                    .await
                    .expect("failed to init db");
                handle_clone.manage(crate::db::DbState { pool: pool.clone() });

                // Start watchers for active lines
                let lines = sqlx::query(
                    "SELECT id, path, prefix, archived_path FROM lines WHERE active = 1",
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
            crate::commands::lines::get_lines,
            crate::commands::lines::save_line,
            crate::commands::lines::delete_line,
            crate::commands::sql_server::test_sql_server_connection,
            crate::commands::hfsql::get_hfsql_config,
            crate::commands::hfsql::save_hfsql_config,
            crate::commands::hfsql::test_hfsql_connection,
            crate::commands::hfsql::sync_ateis_produit,
            crate::commands::hfsql::sync_ateis_of,
            crate::commands::exports::export_logitron_produit_dat,
            crate::commands::exports::export_ordre_fabrication_dat,
            crate::commands::exports::export_ateis_produit_dat,
            crate::commands::exports::export_ateis_of_dat,
            crate::commands::sql_queries::get_sql_query,
            crate::commands::sql_queries::reset_sql_query,
            crate::commands::lines::toggle_line_active,
            crate::commands::lines::start_line_watcher,
            crate::commands::lines::stop_line_watcher,
            crate::commands::dashboard::get_dashboard_snapshot,
            crate::commands::mappings::get_mappings,
            crate::commands::mappings::save_mappings,
            crate::commands::mappings::get_model_mappings,
            crate::commands::mappings::save_model_mappings,
            crate::commands::mappings::reset_model_mappings,
            crate::commands::production::get_production_data,
            crate::commands::sql_server::get_sql_server_config,
            crate::commands::sql_server::save_sql_server_config,
            crate::commands::logs::get_logs,
            crate::commands::logs::add_log,
            crate::commands::logs::clear_logs,
            crate::commands::sql_queries::get_sql_queries,
            crate::commands::sql_queries::save_sql_query,
            crate::commands::defaults::get_default_mappings,
            crate::commands::logs::reset_line_stats
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
