use crate::db::DbState;
use crate::stock;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tauri::{AppHandle, State};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Line {
    pub id: Option<i64>,
    pub name: String,
    pub path: String,
    pub prefix: String,
    pub interval_check: i64,
    pub interval_alert: i64,
    pub archived_path: Option<String>,
    pub rejected_path: Option<String>,
    pub active: bool,
    pub site: Option<String>,
    pub unite: Option<String>,
    pub flag_dec: Option<String>,
    pub code_ligne: Option<String>,
    pub log_path: Option<String>,
    pub file_format: Option<String>,
    pub total_traites: Option<i64>,
    pub total_erreurs: Option<i64>,
    pub last_file_time: Option<String>,
    pub etat_actuel: Option<String>,
    pub created_at: Option<String>,
}

use chrono::Local;

#[tauri::command]
pub async fn get_lines(state: State<'_, DbState>) -> Result<Vec<Line>, String> {
    // 1. Fetch all lines
    let mut lines = sqlx::query_as::<_, Line>(
        "SELECT id, name, path, prefix, interval_check, interval_alert, archived_path, rejected_path, active, \
                site, unite, flag_dec, code_ligne, log_path, file_format,\
                0 as total_traites, 0 as total_erreurs, last_file_time, etat_actuel, created_at \
         FROM lines ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    // 2. Calculate today's stats for each line
    let today_prefix = Local::now().format("%Y-%m-%d").to_string() + "%";

    for line in &mut lines {
        if let Some(id) = line.id {
            // Count successes
            let traites: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM production_data 
                 WHERE line_id = ? AND status = 'SUCCESS' AND processed_at LIKE ?",
            )
            .bind(id)
            .bind(&today_prefix)
            .fetch_one(&state.pool)
            .await
            .unwrap_or(0);

            // Count errors
            let erreurs: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM production_data 
                 WHERE line_id = ? AND status != 'SUCCESS' AND processed_at LIKE ?",
            )
            .bind(id)
            .bind(&today_prefix)
            .fetch_one(&state.pool)
            .await
            .unwrap_or(0);

            line.total_traites = Some(traites);
            line.total_erreurs = Some(erreurs);
        }
    }

    Ok(lines)
}

#[tauri::command]
pub async fn save_line(state: State<'_, DbState>, line: Line) -> Result<i64, String> {
    if let Some(id) = line.id {
        sqlx::query(
            "UPDATE lines SET \
                name = ?, path = ?, prefix = ?, interval_check = ?, \
                interval_alert = ?, archived_path = ?, rejected_path = ?, active = ?,\
                site = ?, unite = ?, flag_dec = ?, code_ligne = ?, log_path = ?, file_format = ?\
            WHERE id = ?",
        )
        .bind(&line.name)
        .bind(&line.path)
        .bind(&line.prefix)
        .bind(line.interval_check)
        .bind(line.interval_alert)
        .bind(&line.archived_path)
        .bind(&line.rejected_path)
        .bind(line.active)
        .bind(&line.site)
        .bind(&line.unite)
        .bind(&line.flag_dec)
        .bind(&line.code_ligne)
        .bind(&line.log_path)
        .bind(&line.file_format)
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(id)
    } else {
        let id = sqlx::query(
            "INSERT INTO lines (name, path, prefix, interval_check, interval_alert, archived_path, rejected_path, active, \
                               site, unite, flag_dec, code_ligne, log_path, file_format) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&line.name)
        .bind(&line.path)
        .bind(&line.prefix)
        .bind(line.interval_check)
        .bind(line.interval_alert)
        .bind(&line.archived_path)
        .bind(&line.rejected_path)
        .bind(line.active)
        .bind(&line.site)
        .bind(&line.unite)
        .bind(&line.flag_dec)
        .bind(&line.code_ligne)
        .bind(&line.log_path)
        .bind(&line.file_format)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?
        .last_insert_rowid();

        Ok(id)
    }
}

#[tauri::command]
pub async fn delete_line(
    app_handle: AppHandle,
    state: State<'_, DbState>,
    id: i64,
) -> Result<(), String> {
    stock::stop_watcher(app_handle, id);

    sqlx::query("DELETE FROM lines WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn toggle_line_active(
    app_handle: AppHandle,
    state: State<'_, DbState>,
    id: i64,
    active: bool,
) -> Result<(), String> {
    sqlx::query("UPDATE lines SET active = ? WHERE id = ?")
        .bind(active)
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    if active {
        let row = sqlx::query("SELECT path, prefix, archived_path FROM lines WHERE id = ?")
            .bind(id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        use sqlx::Row;
        stock::start_watcher(
            app_handle,
            id,
            row.get::<String, _>("path"),
            row.get::<String, _>("prefix"),
            row.get::<Option<String>, _>("archived_path"),
        );
    } else {
        stock::stop_watcher(app_handle, id);
    }

    Ok(())
}

#[tauri::command]
pub async fn start_line_watcher(
    app_handle: AppHandle,
    id: i64,
    path: String,
    prefix: String,
    archived_path: Option<String>,
) -> Result<(), String> {
    stock::start_watcher(app_handle, id, path, prefix, archived_path);
    Ok(())
}

#[tauri::command]
pub async fn stop_line_watcher(app_handle: AppHandle, id: i64) -> Result<(), String> {
    stock::stop_watcher(app_handle, id);
    Ok(())
}
