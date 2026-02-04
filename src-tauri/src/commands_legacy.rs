use crate::db::DbState;
use crate::stock;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;
use tauri::{State, AppHandle};
use tiberius::{Client, AuthMethod, Config as SqlConfig, QueryItem};
use tokio_util::compat::TokioAsyncWriteCompatExt;

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

async fn get_or_init_sql_query(pool: &Pool<Sqlite>, format_name: &str, default_query: &str) -> Result<String, String> {
    // Try get
    if let Some(existing) = sqlx::query_scalar::<_, String>(
        "SELECT query_template FROM sql_queries WHERE format_name = ?",
    )
    .bind(format_name)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())? {
        return Ok(existing);
    }

    // Insert default if missing
    sqlx::query(
        "INSERT INTO sql_queries (format_name, query_template) VALUES (?, ?)",
    )
    .bind(format_name)
    .bind(default_query)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(default_query.to_string())
}

fn format_left(value: Option<String>, width: usize) -> String {
    let s = value.unwrap_or_default();
    let mut out: String = s.chars().take(width).collect();
    if out.chars().count() < width {
        out.push_str(&" ".repeat(width - out.chars().count()));
    }
    out
}

fn format_left_any<T: ToString>(value: Option<T>, width: usize) -> String {
    format_left(value.map(|v| v.to_string()), width)
}

async fn connect_sql_server(cfg: SqlServerConfig) -> Result<Client<tokio_util::compat::Compat<tokio::net::TcpStream>>, String> {
    if !cfg.enabled {
        return Err("SQL Server désactivé (activez la connexion dans Paramètres)".to_string());
    }

    let server = cfg.server.unwrap_or_default();
    let database = cfg.database.unwrap_or_default();
    let username = cfg.username.unwrap_or_default();
    let password = cfg.password.unwrap_or_default();

    if server.trim().is_empty() {
        return Err("SQL Server: serveur manquant".to_string());
    }
    if username.trim().is_empty() {
        return Err("SQL Server: utilisateur manquant".to_string());
    }
    if password.trim().is_empty() {
        return Err("SQL Server: mot de passe manquant".to_string());
    }

    let mut tiberius_config = SqlConfig::new();
    tiberius_config.host(server.as_str());
    tiberius_config.port(1433);
    tiberius_config.authentication(AuthMethod::sql_server(username, password));
    tiberius_config.trust_cert();
    if !database.trim().is_empty() {
        tiberius_config.database(database);
    }

    let tcp = tokio::net::TcpStream::connect(tiberius_config.get_addr())
        .await
        .map_err(|e| e.to_string())?;
    tcp.set_nodelay(true).map_err(|e| e.to_string())?;

    Client::connect(tiberius_config, tcp.compat_write())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_logitron_produit_dat(state: State<'_, DbState>, output_path: String) -> Result<ExportDatResult, String> {
    if output_path.trim().is_empty() {
        return Err("Chemin de sortie manquant".to_string());
    }

    let cfg = get_sql_server_config(state.clone()).await?;
    let mut client = connect_sql_server(cfg).await?;

    let query = get_or_init_sql_query(&state.pool, "LOGITRON_PRODUIT", DEFAULT_LOGITRON_PRODUIT_QUERY).await?;

    let mut stream = client.query(query.as_str(), &[]).await.map_err(|e| e.to_string())?;

    let out_path = Path::new(&output_path);
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    let tmp_path = out_path.with_extension("tmp");
    let tmp_file = fs::File::create(&tmp_path).map_err(|e| e.to_string())?;
    let mut writer = BufWriter::new(tmp_file);

    let mut row_count: i64 = 0;
    while let Some(item) = stream.try_next().await.map_err(|e| e.to_string())? {
        let row = match item {
            QueryItem::Row(r) => r,
            _ => continue,
        };

        let code_produit: Option<&str> = row.get("CODE_PRODUIT");
        let libelle: Option<&str> = row.get("LIBELLE");
        let poids_casier: Option<f64> = row.get("POIDS_CASIER");
        let ean_carton: Option<&str> = row.get("EAN_CARTON");
        let nb_bouteille_casier: Option<i64> = row.get("NB_BOUTEILLE_PAR_CASIER");
        let nb_bouteille_palette: Option<i64> = row.get("NB_BOUTEILLE_PAR_PALETTE");
        let nb_casier_palette: Option<i64> = row.get("NB_CASIER_PAR_PALETTE");
        let methode_dluo: Option<&str> = row.get("METHODE_CALCUL_DLUO");
        let ean_palette: Option<&str> = row.get("EAN_PALETTE");

        let line = format!(
            "{}{}{}{}{}{}{}{}{}\n",
            format_left(code_produit.map(|s| s.to_string()), 14),
            format_left(libelle.map(|s| s.to_string()), 30),
            format_left_any(poids_casier, 22),
            format_left(ean_carton.map(|s| s.to_string()), 14),
            format_left_any(nb_bouteille_casier, 22),
            format_left_any(nb_bouteille_palette, 22),
            format_left_any(nb_casier_palette, 22),
            format_left(methode_dluo.map(|s| s.to_string()), 8),
            format_left(ean_palette.map(|s| s.to_string()), 14),
        );

        writer.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
        row_count += 1;
    }

    writer.flush().map_err(|e| e.to_string())?;
    drop(writer);

    if out_path.exists() {
        fs::remove_file(out_path).map_err(|e| e.to_string())?;
    }
    fs::rename(&tmp_path, out_path).map_err(|e| e.to_string())?;

    Ok(ExportDatResult { output_path, rows: row_count })
}

#[tauri::command]
pub async fn get_sql_query(state: State<'_, DbState>, format_name: String) -> Result<String, String> {
    let fname = format_name.to_uppercase();
    match fname.as_str() {
        "LOGITRON_PRODUIT" => get_or_init_sql_query(&state.pool, &fname, DEFAULT_LOGITRON_PRODUIT_QUERY).await,
        _ => get_or_init_sql_query(&state.pool, &fname, "" ).await,
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SqlQuery {
    pub id: i64,
    pub format_name: String,
    pub query_template: String,
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
pub struct ConnectionTestResult {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExportDatResult {
    pub output_path: String,
    pub rows: i64,
}

const DEFAULT_LOGITRON_PRODUIT_QUERY: &str = r#"
    SELECT 
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

#[derive(Debug, Serialize)]
pub struct DashboardLine {
    pub id: i64,
    pub name: String,
    pub active: bool,
    pub pending_files: i64,
    pub last_processed: Option<String>,
    pub total_processed: i64,
    pub status: String,
    pub site: Option<String>,
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

async fn save_model_mappings_to_db(pool: &Pool<Sqlite>, format_name: &str, mappings: Vec<MappingRow>) -> Result<(), String> {
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
pub async fn save_model_mappings(state: State<'_, DbState>, format_name: String, mappings: Vec<MappingRow>) -> Result<(), String> {
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
pub async fn get_lines(state: State<'_, DbState>) -> Result<Vec<Line>, String> {
    let lines = sqlx::query_as::<_, Line>(
        "SELECT id, name, path, prefix, interval_check, interval_alert, archived_path, rejected_path, active, 
                site, unite, flag_dec, code_ligne, log_path, file_format,
                total_traites, total_erreurs, last_file_time, etat_actuel, created_at 
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
                interval_alert = ?, archived_path = ?, rejected_path = ?, active = ?,
                site = ?, unite = ?, flag_dec = ?, code_ligne = ?, log_path = ?, file_format = ?
            WHERE id = ?"
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
        // Insert
        let id = sqlx::query(
            "INSERT INTO lines (name, path, prefix, interval_check, interval_alert, archived_path, rejected_path, active, 
                               site, unite, flag_dec, code_ligne, log_path, file_format) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
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
pub async fn test_sql_server_connection(state: State<'_, DbState>) -> Result<ConnectionTestResult, String> {
    let cfg = get_sql_server_config(state).await?;

    let server = cfg.server.unwrap_or_default();
    let database = cfg.database.unwrap_or_default();
    let username = cfg.username.unwrap_or_default();
    let password = cfg.password.unwrap_or_default();

    if server.trim().is_empty() {
        return Ok(ConnectionTestResult {
            success: false,
            error: Some("Serveur SQL Server manquant".to_string()),
        });
    }

    let mut tiberius_config = SqlConfig::new();
    tiberius_config.host(server.as_str());
    tiberius_config.port(1433);
    tiberius_config.authentication(AuthMethod::sql_server(username, password));
    tiberius_config.trust_cert();
    if !database.trim().is_empty() {
        tiberius_config.database(database);
    }

    let connect_res = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        tokio::net::TcpStream::connect(tiberius_config.get_addr()),
    )
    .await;

    let tcp = match connect_res {
        Ok(Ok(tcp)) => tcp,
        Ok(Err(e)) => {
            return Ok(ConnectionTestResult { success: false, error: Some(e.to_string()) });
        }
        Err(_) => {
            return Ok(ConnectionTestResult { success: false, error: Some("Timeout de connexion (10s)".to_string()) });
        }
    };

    if let Err(e) = tcp.set_nodelay(true) {
        return Ok(ConnectionTestResult { success: false, error: Some(e.to_string()) });
    }

    let client_res = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        Client::connect(tiberius_config, tcp.compat_write()),
    )
    .await;

    match client_res {
        Ok(Ok(mut client)) => {
            // Lightweight roundtrip: SELECT 1
            let query_res = client.query("SELECT 1", &[]).await;
            match query_res {
                Ok(_) => Ok(ConnectionTestResult { success: true, error: None }),
                Err(e) => Ok(ConnectionTestResult { success: false, error: Some(e.to_string()) }),
            }
        }
        Ok(Err(e)) => Ok(ConnectionTestResult { success: false, error: Some(e.to_string()) }),
        Err(_) => Ok(ConnectionTestResult { success: false, error: Some("Timeout de connexion (10s)".to_string()) }),
    }
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
        "SELECT id, name, path, prefix, interval_check, interval_alert, archived_path, rejected_path, active, 
                site, unite, flag_dec, code_ligne, log_path, file_format,
                total_traites, total_erreurs, last_file_time, etat_actuel, created_at 
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
            site: line.site,
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

// ============ SQL Query Templates Commands ============

#[tauri::command]
pub async fn get_sql_queries(state: State<'_, DbState>) -> Result<Vec<SqlQuery>, String> {
    let queries = sqlx::query_as::<_, SqlQuery>(
        "SELECT id, format_name, query_template FROM sql_queries ORDER BY format_name"
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
        "INSERT INTO sql_queries (format_name, query_template) VALUES (?, ?)
         ON CONFLICT(format_name) DO UPDATE SET query_template = excluded.query_template"
    )
    .bind(&format_name)
    .bind(&query_template)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

// ============ Default Mappings Generation ============

/// Get default mappings for a file format (ATEIS or LOGITRON)
/// Matches the Python Config.get_default_mappings() function
#[tauri::command]
pub async fn get_default_mappings(format_name: String) -> Result<Vec<MappingRow>, String> {
    let mappings = match format_name.to_uppercase().as_str() {
        "ATEIS" => get_ateis_default_mappings(),
        "LOGITRON" => get_logitron_default_mappings(),
        _ => get_ateis_default_mappings(),
    };
    Ok(mappings)
}

fn get_ateis_default_mappings() -> Vec<MappingRow> {
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

fn get_logitron_default_mappings() -> Vec<MappingRow> {
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

/// Reset line statistics
#[tauri::command]
pub async fn reset_line_stats(state: State<'_, DbState>, line_id: i64) -> Result<(), String> {
    sqlx::query(
        "UPDATE lines SET total_traites = 0, total_erreurs = 0, last_file_time = NULL, etat_actuel = 'ARRET' WHERE id = ?"
    )
    .bind(line_id)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}
