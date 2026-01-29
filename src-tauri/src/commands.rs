use crate::db::DbState;
use crate::stock;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tauri::{State, AppHandle};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Line {
    pub id: Option<i64>,
    pub name: String,
    pub path: String,
    pub prefix: String,
    pub interval_check: i64,
    pub interval_alert: i64,
    pub archived_path: Option<String>,
    pub active: bool,
    pub site: Option<String>,
    pub unite: Option<String>,
    pub flag_dec: Option<String>,
    pub code_ligne: Option<String>,
    pub log_path: Option<String>,
    pub file_format: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SqlServerConfig {
    pub id: i64,
    pub server: Option<String>,
    pub database: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct LogEntry {
    pub id: i64,
    pub line_id: Option<i64>,
    pub level: String,
    pub source: Option<String>,
    pub message: String,
    pub details: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct DashboardLine {
    pub id: i64,
    pub name: String,
    pub active: bool,
    pub pending_files: i64,
    pub last_processed: Option<String>,
    pub total_processed: i64,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MappingRow {
    pub id: Option<i64>,
    pub line_id: i64,
    pub sort_order: i64,
    pub sql_field: String,
    pub file_column: Option<String>,
    pub parameter: Option<String>,
    pub transformation: Option<String>,
    pub description: Option<String>,
}

#[tauri::command]
pub async fn get_lines(state: State<'_, DbState>) -> Result<Vec<Line>, String> {
    let lines = sqlx::query_as::<_, Line>(
        "SELECT id, name, path, prefix, interval_check, interval_alert, archived_path, active, 
                site, unite, flag_dec, code_ligne, log_path, file_format, created_at 
         FROM lines ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(lines)
}

#[tauri::command]
pub async fn save_line(state: State<'_, DbState>, line: Line) -> Result<i64, String> {
    if let Some(id) = line.id {
        // Update
        sqlx::query(
            "UPDATE lines SET 
                name = ?, path = ?, prefix = ?, interval_check = ?, 
                interval_alert = ?, archived_path = ?, active = ?,
                site = ?, unite = ?, flag_dec = ?, code_ligne = ?, log_path = ?, file_format = ?
            WHERE id = ?"
        )
        .bind(&line.name)
        .bind(&line.path)
        .bind(&line.prefix)
        .bind(line.interval_check)
        .bind(line.interval_alert)
        .bind(&line.archived_path)
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
        // Insert
        let id = sqlx::query(
            "INSERT INTO lines (name, path, prefix, interval_check, interval_alert, archived_path, active, 
                               site, unite, flag_dec, code_ligne, log_path, file_format) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&line.name)
        .bind(&line.path)
        .bind(&line.prefix)
        .bind(line.interval_check)
        .bind(line.interval_alert)
        .bind(&line.archived_path)
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
pub async fn delete_line(app_handle: AppHandle, state: State<'_, DbState>, id: i64) -> Result<(), String> {
    // Stop watcher first (if any) to avoid orphan threads.
    stock::stop_watcher(app_handle, id);

    sqlx::query("DELETE FROM lines WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn toggle_line_active(app_handle: AppHandle, state: State<'_, DbState>, id: i64, active: bool) -> Result<(), String> {
    sqlx::query("UPDATE lines SET active = ? WHERE id = ?")
        .bind(active)
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    // Start/stop watcher
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
pub async fn start_line_watcher(app_handle: AppHandle, id: i64, path: String, prefix: String, archived_path: Option<String>) -> Result<(), String> {
    stock::start_watcher(app_handle, id, path, prefix, archived_path);
    Ok(())
}

#[tauri::command]
pub async fn stop_line_watcher(app_handle: AppHandle, id: i64) -> Result<(), String> {
    stock::stop_watcher(app_handle, id);
    Ok(())
}

#[tauri::command]
pub async fn get_dashboard_snapshot(state: State<'_, DbState>) -> Result<Vec<DashboardLine>, String> {
    let lines = sqlx::query_as::<_, Line>(
        "SELECT id, name, path, prefix, interval_check, interval_alert, archived_path, active, 
                site, unite, flag_dec, code_ligne, log_path, file_format, created_at 
         FROM lines ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut result = Vec::new();

    for line in lines {
        let id = line.id.unwrap_or_default();

        // last_processed = last production_data entry timestamp
        let last_processed: Option<String> = sqlx::query_scalar(
            "SELECT processed_at FROM production_data WHERE line_id = ? ORDER BY processed_at DESC LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        let total_processed: i64 = sqlx::query_scalar(
            "SELECT COUNT(1) FROM production_data WHERE line_id = ? AND status = 'SUCCESS'"
        )
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        // pending files count (similar to python "PREFIX*.TMP")
        let pending_files = match std::fs::read_dir(&line.path) {
            Ok(rd) => rd
                .flatten()
                .filter(|e| {
                    let p = e.path();
                    if !p.is_file() {
                        return false;
                    }
                    let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("").to_uppercase();
                    name.ends_with(".TMP") && name.contains(&line.prefix.to_uppercase())
                })
                .count() as i64,
            Err(_) => 0,
        };

        let status = if !line.active {
            "ARRET".to_string()
        } else {
            // Time-based logic: if last_processed within interval_check => MARCHE
            // within interval_alert => ALERTE, else ARRET.
            if let Some(lp) = &last_processed {
                // sqlx returns "YYYY-MM-DD HH:MM:SS" for sqlite CURRENT_TIMESTAMP
                let parsed = DateTime::parse_from_rfc3339(lp)
                    .ok()
                    .map(|dt| dt.with_timezone(&Local))
                    .or_else(|| {
                        NaiveDateTime::parse_from_str(lp, "%Y-%m-%d %H:%M:%S")
                            .ok()
                            .and_then(|ndt| Local.from_local_datetime(&ndt).single())
                    });

                if let Some(dt) = parsed {
                    let minutes = (Local::now() - dt).num_minutes();
                    if minutes <= line.interval_check {
                        "MARCHE".to_string()
                    } else if minutes <= line.interval_alert {
                        "ALERTE".to_string()
                    } else {
                        "ARRET".to_string()
                    }
                } else {
                    "ALERTE".to_string()
                }
            } else {
                "ALERTE".to_string()
            }
        };

        result.push(DashboardLine {
            id,
            name: line.name,
            active: line.active,
            pending_files,
            last_processed,
            total_processed,
            status,
        });
    }

    Ok(result)
}

#[tauri::command]
pub async fn get_mappings(state: State<'_, DbState>, line_id: i64) -> Result<Vec<MappingRow>, String> {
    let rows = sqlx::query_as::<_, MappingRow>(
        "SELECT id, line_id, sort_order, sql_field, file_column, parameter, transformation, description \
         FROM mappings WHERE line_id = ? ORDER BY sort_order ASC, id ASC",
    )
    .bind(line_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[tauri::command]
pub async fn save_mappings(state: State<'_, DbState>, line_id: i64, mappings: Vec<MappingRow>) -> Result<(), String> {
    let mut tx = state.pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM mappings WHERE line_id = ?")
        .bind(line_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    for (idx, m) in mappings.into_iter().enumerate() {
        let sort_order = if m.sort_order != 0 { m.sort_order } else { idx as i64 };

        sqlx::query(
            "INSERT INTO mappings (line_id, sort_order, sql_field, file_column, parameter, transformation, description) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(line_id)
        .bind(sort_order)
        .bind(m.sql_field)
        .bind(m.file_column)
        .bind(m.parameter)
        .bind(m.transformation)
        .bind(m.description)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_production_data(state: State<'_, DbState>, line_id: i64) -> Result<Vec<serde_json::Value>, String> {
    let rows = sqlx::query(
        "SELECT id, filename, processed_at, status, message FROM production_data WHERE line_id = ? ORDER BY processed_at DESC LIMIT 100"
    )
    .bind(line_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for row in rows {
        use sqlx::Row;
        result.push(serde_json::json!({
            "id": row.get::<i64, _>("id"),
            "filename": row.get::<String, _>("filename"),
            "processed_at": row.get::<String, _>("processed_at"),
            "status": row.get::<String, _>("status"),
            "message": row.get::<String, _>("message"),
        }));
    }
    Ok(result)
}

// ============ SQL Server Config Commands ============

#[tauri::command]
pub async fn get_sql_server_config(state: State<'_, DbState>) -> Result<SqlServerConfig, String> {
    let config = sqlx::query_as::<_, SqlServerConfig>(
        "SELECT id, server, database, username, password, enabled FROM sql_server_config WHERE id = 1"
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(config)
}

#[tauri::command]
pub async fn save_sql_server_config(
    state: State<'_, DbState>,
    server: String,
    database: String,
    username: String,
    password: String,
    enabled: bool,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE sql_server_config SET server = ?, database = ?, username = ?, password = ?, enabled = ? WHERE id = 1"
    )
    .bind(&server)
    .bind(&database)
    .bind(&username)
    .bind(&password)
    .bind(enabled)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

// ============ Logs Commands ============

#[tauri::command]
pub async fn get_logs(
    state: State<'_, DbState>,
    line_id: Option<i64>,
    level: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<LogEntry>, String> {
    let limit_val = limit.unwrap_or(200);
    
    let logs = if let Some(lid) = line_id {
        if let Some(lvl) = level {
            sqlx::query_as::<_, LogEntry>(
                "SELECT id, line_id, level, source, message, details, created_at 
                 FROM logs WHERE line_id = ? AND level = ? ORDER BY created_at DESC LIMIT ?"
            )
            .bind(lid)
            .bind(&lvl)
            .bind(limit_val)
            .fetch_all(&state.pool)
            .await
        } else {
            sqlx::query_as::<_, LogEntry>(
                "SELECT id, line_id, level, source, message, details, created_at 
                 FROM logs WHERE line_id = ? ORDER BY created_at DESC LIMIT ?"
            )
            .bind(lid)
            .bind(limit_val)
            .fetch_all(&state.pool)
            .await
        }
    } else if let Some(lvl) = level {
        sqlx::query_as::<_, LogEntry>(
            "SELECT id, line_id, level, source, message, details, created_at 
             FROM logs WHERE level = ? ORDER BY created_at DESC LIMIT ?"
        )
        .bind(&lvl)
        .bind(limit_val)
        .fetch_all(&state.pool)
        .await
    } else {
        sqlx::query_as::<_, LogEntry>(
            "SELECT id, line_id, level, source, message, details, created_at 
             FROM logs ORDER BY created_at DESC LIMIT ?"
        )
        .bind(limit_val)
        .fetch_all(&state.pool)
        .await
    };

    logs.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_log(
    state: State<'_, DbState>,
    line_id: Option<i64>,
    level: String,
    source: Option<String>,
    message: String,
    details: Option<String>,
) -> Result<i64, String> {
    let id = sqlx::query(
        "INSERT INTO logs (line_id, level, source, message, details) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(line_id)
    .bind(&level)
    .bind(&source)
    .bind(&message)
    .bind(&details)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?
    .last_insert_rowid();

    Ok(id)
}

#[tauri::command]
pub async fn clear_logs(state: State<'_, DbState>, line_id: Option<i64>) -> Result<(), String> {
    if let Some(lid) = line_id {
        sqlx::query("DELETE FROM logs WHERE line_id = ?")
            .bind(lid)
            .execute(&state.pool)
            .await
            .map_err(|e| e.to_string())?;
    } else {
        sqlx::query("DELETE FROM logs")
            .execute(&state.pool)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
