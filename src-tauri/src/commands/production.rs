use crate::db::DbState;
use tauri::State;

#[tauri::command]
pub async fn get_production_data(
    state: State<'_, DbState>,
    line_id: i64,
) -> Result<Vec<serde_json::Value>, String> {
    let rows = sqlx::query(
        "SELECT id, filename, processed_at, status, message FROM production_data WHERE line_id = ? ORDER BY processed_at DESC LIMIT 100",
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
