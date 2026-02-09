use std::collections::HashMap;
use std::sync::{mpsc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};

pub struct SchedulerState {
    // Map of task_type -> stop_channel_sender
    pub tasks: Mutex<HashMap<String, mpsc::Sender<()>>>,
}

impl SchedulerState {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Clone, serde::Serialize)]
pub struct SchedulerStatus {
    pub task_type: String,
    pub running: bool,
}

#[tauri::command]
pub async fn start_scheduler(
    app_handle: AppHandle,
    state: State<'_, SchedulerState>,
    task_type: String,
    interval_minutes: u64,
    param: Option<String>,
) -> Result<(), String> {
    let mut tasks = state.tasks.lock().map_err(|e| e.to_string())?;

    if tasks.contains_key(&task_type) {
        return Ok(()); // Already running
    }

    let (tx, rx) = mpsc::channel();
    tasks.insert(task_type.clone(), tx);

    let interval_sec = interval_minutes * 60;
    let task_type_clone = task_type.clone();
    let param_clone = param.clone();

    thread::spawn(move || {
        let mut first_run = true;
        loop {
            // Check for stop signal
            if rx.try_recv().is_ok() {
                break;
            }

            // Sleep logic
            if !first_run {
                match rx.recv_timeout(Duration::from_secs(interval_sec)) {
                    Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Continue to execution
                    }
                }
            }
            first_run = false;

            // Execute Task based on type
            let app_handle_clone = app_handle.clone();
            let tt = task_type_clone.clone();
            let p_val = param_clone.clone();

            tauri::async_runtime::block_on(async move {
                match tt.as_str() {
                    "ATEIS_PRODUIT_SYNC" => {
                        let _ = crate::commands::hfsql::sync_ateis_produit(
                            app_handle_clone.clone(),
                            app_handle_clone.state::<crate::db::DbState>(),
                        )
                        .await;
                    }
                    "ATEIS_OF_SYNC" => {
                        let _ = crate::commands::hfsql::sync_ateis_of(
                            app_handle_clone.clone(),
                            app_handle_clone.state::<crate::db::DbState>(),
                        )
                        .await;
                    }
                    "LOGITRON_PRODUIT" => {
                        // Use param as path if provided, else resolve default
                        let state = app_handle_clone.state::<crate::db::DbState>();
                        let output_path = if let Some(p) = p_val.as_ref().filter(|s| !s.is_empty())
                        {
                            p.clone()
                        } else {
                            // Fallback logic
                            let base_path =
                                match crate::commands::hfsql::get_hfsql_config(state.clone()).await
                                {
                                    Ok(cfg) => cfg.log_path.filter(|p| !p.is_empty()),
                                    Err(_) => None,
                                };
                            let output_dir = base_path.unwrap_or_else(|| {
                                app_handle_clone
                                    .path()
                                    .desktop_dir()
                                    .ok()
                                    .map(|p| p.join("T").join("BLOG").to_string_lossy().to_string())
                                    .unwrap_or_else(|| r"C:\T\BLOG".to_string())
                            });
                            std::path::Path::new(&output_dir)
                                .join("LOGITRON_PRODUIT.DAT")
                                .to_string_lossy()
                                .to_string()
                        };

                        let _ = crate::commands::exports::export_logitron_produit_dat(
                            app_handle_clone.clone(),
                            state.clone(),
                            output_path,
                            Some(true),
                        )
                        .await;
                    }
                    "LOGITRON_OF" => {
                        // Use param as path if provided, else resolve default
                        let state = app_handle_clone.state::<crate::db::DbState>();
                        let output_path = if let Some(p) = p_val.as_ref().filter(|s| !s.is_empty())
                        {
                            p.clone()
                        } else {
                            // Fallback logic
                            let base_path =
                                match crate::commands::hfsql::get_hfsql_config(state.clone()).await
                                {
                                    Ok(cfg) => cfg.log_path.filter(|p| !p.is_empty()),
                                    Err(_) => None,
                                };
                            let output_dir = base_path.unwrap_or_else(|| {
                                app_handle_clone
                                    .path()
                                    .desktop_dir()
                                    .ok()
                                    .map(|p| p.join("T").join("BLOG").to_string_lossy().to_string())
                                    .unwrap_or_else(|| r"C:\T\BLOG".to_string())
                            });
                            std::path::Path::new(&output_dir)
                                .join("LOGITRON_ORDRE_FABRICATION.DAT")
                                .to_string_lossy()
                                .to_string()
                        };

                        let _ = crate::commands::exports::export_ordre_fabrication_dat(
                            app_handle_clone.clone(),
                            state.clone(),
                            output_path,
                        )
                        .await;
                    }
                    "ATEIS_EXPORT" => {
                        // Keep ATEIS_EXPORT as combined for now if not requested otherwise, or split if needed.
                        // User only mentioned Logitron issues. Leaving ATEIS_EXPORT logic as is but fixing path resolution if needed.
                        let state = app_handle_clone.state::<crate::db::DbState>();
                        let base_path =
                            match crate::commands::hfsql::get_hfsql_config(state.clone()).await {
                                Ok(cfg) => cfg.log_path.filter(|p| !p.is_empty()),
                                Err(_) => None,
                            };

                        let output_dir = base_path.unwrap_or_else(|| {
                            app_handle_clone
                                .path()
                                .desktop_dir()
                                .ok()
                                .map(|p| p.join("T").join("BLOG").to_string_lossy().to_string())
                                .unwrap_or_else(|| r"C:\T\BLOG".to_string())
                        });

                        let path_prod = std::path::Path::new(&output_dir).join("ATEIS_PRODUIT.DAT");
                        let path_of = std::path::Path::new(&output_dir).join("ATEIS_OF.DAT");

                        let _ = crate::commands::exports::export_ateis_produit_dat(
                            state.clone(),
                            path_prod.to_string_lossy().to_string(),
                        )
                        .await;

                        let _ = crate::commands::exports::export_ateis_of_dat(
                            state.clone(),
                            path_of.to_string_lossy().to_string(),
                        )
                        .await;
                    }
                    _ => {
                        eprintln!("Unknown task type: {}", tt);
                    }
                }
            });
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_scheduler(
    state: State<'_, SchedulerState>,
    task_type: String,
) -> Result<(), String> {
    let mut tasks = state.tasks.lock().map_err(|e| e.to_string())?;

    if let Some(tx) = tasks.remove(&task_type) {
        let _ = tx.send(());
    }

    Ok(())
}

#[tauri::command]
pub async fn get_scheduler_status(
    state: State<'_, SchedulerState>,
    task_type: String,
) -> Result<bool, String> {
    let tasks = state.tasks.lock().map_err(|e| e.to_string())?;
    Ok(tasks.contains_key(&task_type))
}
