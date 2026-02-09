use crate::commands::lines::Line;
use crate::db::DbState;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct DashboardLine {
    pub id: i64,
    pub name: String,
    pub active: bool,
    pub pending_files: i64,
    pub error_files: i64,
    pub last_processed: Option<String>,
    pub total_processed: i64,
    pub status: String,
    pub site: Option<String>,
}

#[tauri::command]
pub async fn get_dashboard_snapshot(
    state: State<'_, DbState>,
) -> Result<Vec<DashboardLine>, String> {
    let lines = sqlx::query_as::<_, Line>(
        "SELECT id, name, path, prefix, interval_check, interval_alert, archived_path, rejected_path, active, \
                site, unite, code_ligne, log_path, file_format,\
                total_traites, total_erreurs, last_file_time, etat_actuel, created_at, flag_dec \
         FROM lines ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut result = Vec::new();

    for line in lines {
        let id = line.id.unwrap_or_default();

        let last_processed: Option<String> = sqlx::query_scalar(
            "SELECT processed_at FROM production_data WHERE line_id = ? ORDER BY processed_at DESC LIMIT 1",
        )
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        let today_prefix = Local::now().format("%Y-%m-%d").to_string() + "%";
        let total_processed: i64 = sqlx::query_scalar(
            "SELECT COUNT(1) FROM production_data WHERE line_id = ? AND status = 'SUCCESS' AND processed_at LIKE ?",
        )
        .bind(id)
        .bind(today_prefix)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        let pending_files = match std::fs::read_dir(&line.path) {
            Ok(rd) => rd
                .flatten()
                .filter(|e| {
                    let p = e.path();
                    if !p.is_file() {
                        return false;
                    }
                    let name = p
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_uppercase();
                    name.ends_with(".TMP") && name.contains(&line.prefix.to_uppercase())
                })
                .count() as i64,
            Err(_) => 0,
        };

        let error_files = if let Some(path) = &line.rejected_path {
            match std::fs::read_dir(path) {
                Ok(rd) => rd.flatten().filter(|e| e.path().is_file()).count() as i64,
                Err(_) => 0,
            }
        } else {
            0
        };

        let status = if !line.active {
            "ARRET".to_string()
        } else if let Some(lp) = &last_processed {
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
        };

        result.push(DashboardLine {
            id,
            name: line.name,
            active: line.active,
            pending_files,
            error_files,
            last_processed,
            total_processed,
            status,
            site: line.site,
        });
    }

    Ok(result)
}
