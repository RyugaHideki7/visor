use crate::db::DbState;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SqlQuery {
    pub id: i64,
    pub format_name: String,
    pub query_template: String,
}

pub(crate) const DEFAULT_LOGITRON_PRODUIT_QUERY: &str = r#"SELECT 
        ITMREF_0 as CODE_PRODUIT,
        ITMDES1_0 as LIBELLE,
        CASE WHEN ITMWEI_0 IS NULL THEN 0 ELSE ITMWEI_0 END AS POIDS_CASIER,
        CASE WHEN EANCOD_0 IS NULL THEN '0' ELSE EANCOD_0 END AS EAN_CARTON,
        CASE WHEN YBP_0 IS NULL THEN 0 ELSE YBP_0 END AS NB_BOUTEILLE_PAR_CASIER,
        CASE WHEN YBPL_0 IS NULL THEN 0 ELSE YBPL_0 END AS NB_BOUTEILLE_PAR_PALETTE,
        CASE WHEN YPPL_0 IS NULL THEN 0 ELSE YPPL_0 END AS NB_CASIER_PAR_PALETTE,
        ISNULL(YCDLUO_0, '') as METHODE_CALCUL_DLUO,
        CASE WHEN YTEMPEAN_0 IS NULL THEN '0' ELSE YTEMPEAN_0 END AS EAN_PALETTE
    FROM ITHRI.ITMMASTER
    WHERE ITMREF_0 LIKE 'PF%'
    ORDER BY ITMREF_0
"#;

pub(crate) async fn get_or_init_sql_query(
    pool: &Pool<Sqlite>,
    format_name: &str,
    default_query: &str,
) -> Result<String, String> {
    if let Some(existing) = sqlx::query_scalar::<_, String>(
        "SELECT query_template FROM sql_queries WHERE format_name = ?",
    )
    .bind(format_name)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    {
        return Ok(existing);
    }

    sqlx::query("INSERT INTO sql_queries (format_name, query_template) VALUES (?, ?)")
        .bind(format_name)
        .bind(default_query)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(default_query.to_string())
}

#[tauri::command]
pub async fn get_sql_queries(state: State<'_, DbState>) -> Result<Vec<SqlQuery>, String> {
    let queries = sqlx::query_as::<_, SqlQuery>(
        "SELECT id, format_name, query_template FROM sql_queries ORDER BY format_name",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(queries)
}

#[tauri::command]
pub async fn save_sql_query(
    state: State<'_, DbState>,
    format_name: String,
    query_template: String,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO sql_queries (format_name, query_template) VALUES (?, ?)\n         ON CONFLICT(format_name) DO UPDATE SET query_template = excluded.query_template",
    )
    .bind(&format_name)
    .bind(&query_template)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_sql_query(state: State<'_, DbState>, format_name: String) -> Result<String, String> {
    let fname = format_name.to_uppercase();
    match fname.as_str() {
        "LOGITRON_PRODUIT" => {
            get_or_init_sql_query(&state.pool, &fname, DEFAULT_LOGITRON_PRODUIT_QUERY).await
        }
        _ => get_or_init_sql_query(&state.pool, &fname, "").await,
    }
}

#[tauri::command]
pub async fn reset_sql_query(state: State<'_, DbState>, format_name: String) -> Result<(), String> {
    let fname = format_name.to_uppercase();
    let default = match fname.as_str() {
        "LOGITRON_PRODUIT" => DEFAULT_LOGITRON_PRODUIT_QUERY,
        _ => "",
    };

    if default.is_empty() {
        return Err("Aucun défaut défini pour ce format".to_string());
    }

    sqlx::query(
        "INSERT INTO sql_queries (format_name, query_template) VALUES (?, ?) \
         ON CONFLICT(format_name) DO UPDATE SET query_template = excluded.query_template",
    )
    .bind(&fname)
    .bind(default)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}
