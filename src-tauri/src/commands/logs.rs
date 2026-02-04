use crate::db::DbState;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tauri::State;

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
                "SELECT id, line_id, level, source, message, details, created_at \
                 FROM logs WHERE line_id = ? AND level = ? ORDER BY created_at DESC LIMIT ?",
            )
            .bind(lid)
            .bind(&lvl)
            .bind(limit_val)
            .fetch_all(&state.pool)
            .await
        } else {
            sqlx::query_as::<_, LogEntry>(
                "SELECT id, line_id, level, source, message, details, created_at \
                 FROM logs WHERE line_id = ? ORDER BY created_at DESC LIMIT ?",
            )
            .bind(lid)
            .bind(limit_val)
            .fetch_all(&state.pool)
            .await
        }
    } else if let Some(lvl) = level {
        sqlx::query_as::<_, LogEntry>(
            "SELECT id, line_id, level, source, message, details, created_at \
             FROM logs WHERE level = ? ORDER BY created_at DESC LIMIT ?",
        )
        .bind(&lvl)
        .bind(limit_val)
        .fetch_all(&state.pool)
        .await
    } else {
        sqlx::query_as::<_, LogEntry>(
            "SELECT id, line_id, level, source, message, details, created_at \
             FROM logs ORDER BY created_at DESC LIMIT ?",
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
    let id = sqlx::query("INSERT INTO logs (line_id, level, source, message, details) VALUES (?, ?, ?, ?, ?)")
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

#[tauri::command]
pub async fn reset_line_stats(state: State<'_, DbState>, line_id: i64) -> Result<(), String> {
    sqlx::query(
        "UPDATE lines SET total_traites = 0, total_erreurs = 0, last_file_time = NULL, etat_actuel = 'ARRET' WHERE id = ?",
    )
    .bind(line_id)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}
