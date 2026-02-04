use crate::db::DbState;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use tauri::State;

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

async fn save_model_mappings_to_db(
    pool: &Pool<Sqlite>,
    format_name: &str,
    mappings: Vec<MappingRow>,
) -> Result<(), String> {
    let fmt = format_name.to_uppercase();
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM model_mappings WHERE format_name = ?")
        .bind(&fmt)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    for (idx, m) in mappings.into_iter().enumerate() {
        let sort_order = if m.sort_order != 0 { m.sort_order } else { idx as i64 };

        sqlx::query(
            "INSERT INTO model_mappings (format_name, sort_order, sql_field, file_column, parameter, transformation, description) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&fmt)
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

pub(crate) fn get_ateis_default_mappings() -> Vec<MappingRow> {
    vec![
        MappingRow { id: None, line_id: 0, sort_order: 0, sql_field: "YSSCC_0".to_string(), file_column: Some("0".to_string()), parameter: None, transformation: None, description: Some("Code SCC".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 1, sql_field: "YDATE_0".to_string(), file_column: Some("1".to_string()), parameter: None, transformation: Some("date".to_string()), description: Some("Date déclaration".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 2, sql_field: "YHEURE_0".to_string(), file_column: Some("1".to_string()), parameter: None, transformation: Some("heure".to_string()), description: Some("Heure déclaration".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 3, sql_field: "ITMREF_0".to_string(), file_column: Some("5".to_string()), parameter: None, transformation: None, description: Some("Référence article".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 4, sql_field: "LOT_0".to_string(), file_column: Some("7".to_string()), parameter: None, transformation: None, description: Some("Numéro de lot".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 5, sql_field: "QTY_0".to_string(), file_column: Some("9".to_string()), parameter: None, transformation: Some("decimal".to_string()), description: Some("Quantité".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 6, sql_field: "YDATDL_0".to_string(), file_column: Some("8".to_string()), parameter: None, transformation: Some("date".to_string()), description: Some("Date livraison".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 7, sql_field: "YNLIGN_0".to_string(), file_column: Some("12".to_string()), parameter: None, transformation: None, description: Some("Numéro de ligne".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 8, sql_field: "MFGNUM_0".to_string(), file_column: Some("18".to_string()), parameter: None, transformation: None, description: Some("Numéro de fabrication".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 9, sql_field: "YCODEPOT_0".to_string(), file_column: Some("4".to_string()), parameter: None, transformation: None, description: Some("Code dépôt".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 10, sql_field: "YPALETTE_0".to_string(), file_column: Some("16".to_string()), parameter: None, transformation: None, description: Some("Palette".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 11, sql_field: "YINTERCAL_0".to_string(), file_column: Some("17".to_string()), parameter: None, transformation: None, description: Some("Intercalaire".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 12, sql_field: "FCY_0".to_string(), file_column: None, parameter: Some("site".to_string()), transformation: None, description: Some("Site de production".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 13, sql_field: "UOM_0".to_string(), file_column: None, parameter: Some("unite".to_string()), transformation: None, description: Some("Unité de mesure".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 14, sql_field: "YFLGDEC_0".to_string(), file_column: None, parameter: Some("flag_dec".to_string()), transformation: Some("tinyint".to_string()), description: Some("Flag déclaration".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 15, sql_field: "CREUSR_0".to_string(), file_column: None, parameter: Some("code_ligne".to_string()), transformation: None, description: Some("Utilisateur création".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 16, sql_field: "CREDATTIM_0".to_string(), file_column: Some("1".to_string()), parameter: None, transformation: Some("datetime".to_string()), description: Some("Date/heure création".to_string()) },
    ]
}

pub(crate) fn get_logitron_default_mappings() -> Vec<MappingRow> {
    vec![
        MappingRow { id: None, line_id: 0, sort_order: 0, sql_field: "YSSCC_0".to_string(), file_column: Some("0".to_string()), parameter: None, transformation: None, description: Some("Code SCC".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 1, sql_field: "YDATE_0".to_string(), file_column: Some("1".to_string()), parameter: None, transformation: Some("date".to_string()), description: Some("Date déclaration".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 2, sql_field: "YHEURE_0".to_string(), file_column: Some("2".to_string()), parameter: None, transformation: Some("heure".to_string()), description: Some("Heure déclaration".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 3, sql_field: "CREDATTIM_0".to_string(), file_column: Some("1-2".to_string()), parameter: None, transformation: Some("datetime_combine".to_string()), description: Some("Date/heure création combinée".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 4, sql_field: "ITMREF_0".to_string(), file_column: Some("3".to_string()), parameter: None, transformation: None, description: Some("Référence article".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 5, sql_field: "LOT_0".to_string(), file_column: Some("4".to_string()), parameter: None, transformation: None, description: Some("Numéro de lot".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 6, sql_field: "QTY_0".to_string(), file_column: Some("5".to_string()), parameter: None, transformation: Some("decimal".to_string()), description: Some("Quantité".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 7, sql_field: "YDATDL_0".to_string(), file_column: Some("7".to_string()), parameter: None, transformation: Some("date".to_string()), description: Some("Date livraison".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 8, sql_field: "YNLIGN_0".to_string(), file_column: Some("8".to_string()), parameter: None, transformation: None, description: Some("Numéro de ligne".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 9, sql_field: "MFGNUM_0".to_string(), file_column: Some("13".to_string()), parameter: None, transformation: None, description: Some("Numéro de fabrication".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 10, sql_field: "YCODEPOT_0".to_string(), file_column: Some("14".to_string()), parameter: None, transformation: None, description: Some("Code dépôt".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 11, sql_field: "YPALETTE_0".to_string(), file_column: Some("15".to_string()), parameter: None, transformation: Some("split_before_plus".to_string()), description: Some("Partie avant +".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 12, sql_field: "YINTERCAL_0".to_string(), file_column: Some("15".to_string()), parameter: None, transformation: Some("split_after_plus".to_string()), description: Some("Partie après +".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 13, sql_field: "FCY_0".to_string(), file_column: None, parameter: Some("site".to_string()), transformation: None, description: Some("Site de production".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 14, sql_field: "UOM_0".to_string(), file_column: None, parameter: Some("unite".to_string()), transformation: None, description: Some("Unité de mesure".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 15, sql_field: "YFLGDEC_0".to_string(), file_column: None, parameter: Some("flag_dec".to_string()), transformation: Some("tinyint".to_string()), description: Some("Flag déclaration".to_string()) },
        MappingRow { id: None, line_id: 0, sort_order: 16, sql_field: "CREUSR_0".to_string(), file_column: None, parameter: Some("code_ligne".to_string()), transformation: None, description: Some("Utilisateur création".to_string()) },
    ]
}

#[tauri::command]
pub async fn get_model_mappings(state: State<'_, DbState>, format_name: String) -> Result<Vec<MappingRow>, String> {
    let fmt = format_name.to_uppercase();

    let rows = sqlx::query_as::<_, MappingRow>(
        "SELECT id, 0 as line_id, sort_order, sql_field, file_column, parameter, transformation, description \
         FROM model_mappings WHERE format_name = ? ORDER BY sort_order ASC, id ASC",
    )
    .bind(&fmt)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    if !rows.is_empty() {
        return Ok(rows);
    }

    let defaults = match fmt.as_str() {
        "ATEIS" => get_ateis_default_mappings(),
        "LOGITRON" => get_logitron_default_mappings(),
        _ => get_ateis_default_mappings(),
    };

    save_model_mappings_to_db(&state.pool, &fmt, defaults).await?;

    let rows = sqlx::query_as::<_, MappingRow>(
        "SELECT id, 0 as line_id, sort_order, sql_field, file_column, parameter, transformation, description \
         FROM model_mappings WHERE format_name = ? ORDER BY sort_order ASC, id ASC",
    )
    .bind(&fmt)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[tauri::command]
pub async fn save_model_mappings(
    state: State<'_, DbState>,
    format_name: String,
    mappings: Vec<MappingRow>,
) -> Result<(), String> {
    save_model_mappings_to_db(&state.pool, &format_name, mappings).await
}

#[tauri::command]
pub async fn reset_model_mappings(state: State<'_, DbState>, format_name: String) -> Result<(), String> {
    let fmt = format_name.to_uppercase();
    let defaults = match fmt.as_str() {
        "ATEIS" => get_ateis_default_mappings(),
        "LOGITRON" => get_logitron_default_mappings(),
        _ => get_ateis_default_mappings(),
    };
    save_model_mappings_to_db(&state.pool, &fmt, defaults).await
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
pub async fn save_mappings(
    state: State<'_, DbState>,
    line_id: i64,
    mappings: Vec<MappingRow>,
) -> Result<(), String> {
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
