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

pub(crate) const DEFAULT_ATEIS_QUERY: &str = r#"INSERT INTO ITHRI.YINTDECL (
    YSSCC_0, 
    YDATE_0, 
    YHEURE_0,     
    ITMREF_0, 
    LOT_0, 
    QTY_0,
    YDATDL_0, 
    YNLIGN_0,
    MFGNUM_0, 
    YCODEPOT_0,
    YPALETTE_0,
    YINTERCAL_0,
    FCY_0, 
    UOM_0,
    YFLGDEC_0, 
    CREUSR_0,
    CREDATTIM_0,
    UPDDATTIM_0 
  
) VALUES (
    @P1, 
    @P2,
    @P3,  
    @P4,
    @P5, 
    @P6, 
    @P7,
    @P8,
    @P9, 
    @P10, 
    @P11, 
    @P12, 
    @P13, 
    @P14, 
    @P15,
    @P16,
    @P17, 
    getdate()
)"#;

pub(crate) const DEFAULT_LOGITRON_QUERY: &str = r#"INSERT INTO ITHRI.YINTDECL (
    YSSCC_0, 
    YDATE_0, 
    YHEURE_0, 
    CREDATTIM_0,      
    ITMREF_0, 
    LOT_0, 
    QTY_0,
    YDATDL_0, 
    YNLIGN_0,
    MFGNUM_0, 
    YCODEPOT_0,
    YPALETTE_0,
    YINTERCAL_0,
    FCY_0, 
    UOM_0,
    YFLGDEC_0, 
    CREUSR_0,
    UPDDATTIM_0 
  
) VALUES (
    @P1, 
    @P2,
    @P3,  
    @P4,
    @P5, 
    @P6, 
    @P7,
    @P8,
    @P9, 
    @P10, 
    @P11, 
    @P12, 
    @P13, 
    @P14, 
    @P15,
    @P16,
    @P17, getdate()
)"#;

pub(crate) const DEFAULT_ORDRE_FABRICATION_QUERY: &str = r#"SELECT 
    MFG.MFGNUM_0, 
    MFI.ITMREF_0,           
    MFG.EXTQTY_0, 
    MFG.STRDAT_0,
    '', 
    MFG.YLIGNEOF_0 
FROM ITHRI.MFGHEAD MFG
INNER JOIN ITHRI.MFGITM MFI ON MFG.MFGNUM_0 = MFI.MFGNUM_0
WHERE MFG.MFGFCY_0 LIKE 'IFR%' 
    AND MFI.ITMREF_0 LIKE 'PF%'
    AND DATEDIFF(day, MFG.STRDAT_0, getdate()) < 40
ORDER BY MFG.STRDAT_0 DESC
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
pub async fn get_sql_query(
    state: State<'_, DbState>,
    format_name: String,
) -> Result<String, String> {
    let fname = format_name.to_uppercase();
    match fname.as_str() {
        "LOGITRON_PRODUIT" => {
            get_or_init_sql_query(&state.pool, &fname, DEFAULT_LOGITRON_PRODUIT_QUERY).await
        }
        "LOGITRON_ORDRE_FABRICATION" => {
            get_or_init_sql_query(&state.pool, &fname, DEFAULT_ORDRE_FABRICATION_QUERY).await
        }
        _ => get_or_init_sql_query(&state.pool, &fname, "").await,
    }
}

#[tauri::command]
pub async fn reset_sql_query(state: State<'_, DbState>, format_name: String) -> Result<(), String> {
    let fname = format_name.to_uppercase();
    let default = match fname.as_str() {
        "LOGITRON_PRODUIT" => DEFAULT_LOGITRON_PRODUIT_QUERY,
        "LOGITRON_ORDRE_FABRICATION" => DEFAULT_ORDRE_FABRICATION_QUERY,
        "ATEIS" => DEFAULT_ATEIS_QUERY,
        "LOGITRON" => DEFAULT_LOGITRON_QUERY,
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
