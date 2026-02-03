use notify::{Config, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;
use std::time::{Duration, Instant};
use sqlx::{Pool, Sqlite, Row};
use tauri::AppHandle;
use tauri::Manager;
use chrono::Local;
use std::collections::HashMap;
use std::sync::{mpsc, Mutex};
use serde::Deserialize;
use serde_json::json;
use sqlx::FromRow;
use encoding_rs::{UTF_8, WINDOWS_1252};
use tiberius::{Client, AuthMethod, Config as SqlConfig, ToSql};
use tokio_util::compat::TokioAsyncWriteCompatExt;
use crate::commands::SqlServerConfig;
use log::info;

pub struct WatcherState {
    watchers: Mutex<HashMap<i64, WatcherHandle>>,
}

struct WatcherHandle {
    stop_tx: mpsc::Sender<()>,
}

impl WatcherState {
    pub fn new() -> Self {
        Self {
            watchers: Mutex::new(HashMap::new()),
        }
    }
}

fn parse_insert_columns(query: &str) -> Vec<String> {
    let lower = query.to_lowercase();
    let insert_pos = lower.find("insert");
    if insert_pos.is_none() {
        return Vec::new();
    }

    // Find the first '(' after INSERT ... and the ')' before VALUES
    let open_paren = query[insert_pos.unwrap()..].find('(').map(|i| i + insert_pos.unwrap());
    let values_pos = lower.find(") values");
    if open_paren.is_none() || values_pos.is_none() {
        return Vec::new();
    }

    let open = open_paren.unwrap();
    let close = values_pos.unwrap();
    if close <= open {
        return Vec::new();
    }

    query[open + 1..close]
        .split(',')
        .map(|s| s.trim().trim_matches('[').trim_matches(']').trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn build_param_values_from_query(query: &str, mapped: &HashMap<String, String>) -> Vec<String> {
    let cols = parse_insert_columns(query);
    if cols.is_empty() {
        return Vec::new();
    }
    cols.iter()
        .map(|c| mapped.get(c).cloned().unwrap_or_default())
        .collect()
}

impl StockProcessor {
    async fn execute_sql_server_inserts(
        &self,
        line_config: Option<&LineConfig>,
        mappings: &[MappingRow],
        rows: &[HashMap<String, String>],
    ) -> Result<(), String> {
        let cfg = self
            .load_sql_server_config()
            .await
            .ok_or("SQL Server non configuré")?;

        if !cfg.enabled {
            return Err("SQL Server désactivé (activez la connexion dans Paramètres)".to_string());
        }

        if cfg.server.as_deref().unwrap_or("").trim().is_empty() {
            return Err("SQL Server: serveur manquant".to_string());
        }

        if cfg.username.as_deref().unwrap_or("").trim().is_empty() {
            return Err("SQL Server: utilisateur manquant".to_string());
        }

        if cfg.password.as_deref().unwrap_or("").trim().is_empty() {
            return Err("SQL Server: mot de passe manquant".to_string());
        }

        let format_name = line_config
            .and_then(|l| l.file_format.clone())
            .unwrap_or_else(|| "ATEIS".to_string());

        let query = self
            .load_query_template(&format_name)
            .await
            .ok_or("Template SQL manquant")?;

        let mut tiberius_config = SqlConfig::new();
        tiberius_config.host(cfg.server.as_deref().unwrap_or(""));
        tiberius_config.port(1433);
        tiberius_config.authentication(AuthMethod::sql_server(
            cfg.username.unwrap_or_default(),
            cfg.password.unwrap_or_default(),
        ));
        tiberius_config.trust_cert();
        if let Some(db) = cfg.database {
            tiberius_config.database(db);
        }

        let tcp = tokio::net::TcpStream::connect(tiberius_config.get_addr())
            .await
            .map_err(|e| e.to_string())?;
        tcp.set_nodelay(true).map_err(|e| e.to_string())?;
        let mut client = Client::connect(tiberius_config, tcp.compat_write())
            .await
            .map_err(|e| e.to_string())?;

        let mut logged_debug = false;
        for mapped in rows {
            let mut params = build_param_values_from_query(&query, mapped);
            if params.is_empty() {
                params = mappings
                    .iter()
                    .map(|m| mapped.get(&m.sql_field).cloned().unwrap_or_default())
                    .collect();
            }

            if !logged_debug {
                logged_debug = true;
                let cols = parse_insert_columns(&query);
                let fcy = mapped.get("FCY_0").cloned().unwrap_or_default();
                info!("SQL params order: {:?}", cols);
                info!("SQL mapped FCY_0: {}", fcy);
            }

            // Keep owned strings alive while passing &dyn ToSql references
            let owned: Vec<String> = params;
            let params_refs: Vec<&dyn ToSql> = owned.iter().map(|s| s as &dyn ToSql).collect();
            client
                .execute(query.as_str(), &params_refs[..])
                .await
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    async fn load_sql_server_config(&self) -> Option<SqlServerConfig> {
        sqlx::query_as::<_, SqlServerConfig>(
            "SELECT id, server, database, username, password, enabled FROM sql_server_config WHERE id = 1",
        )
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten()
    }

    async fn load_query_template(&self, format_name: &str) -> Option<String> {
        sqlx::query_scalar::<_, String>(
            "SELECT query_template FROM sql_queries WHERE format_name = ?",
        )
        .bind(format_name)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten()
    }
}

pub struct StockProcessor {
    pool: Pool<Sqlite>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow, Deserialize)]
struct MappingRow {
    pub id: Option<i64>,
    pub line_id: i64,
    pub sort_order: i64,
    pub sql_field: String,
    pub file_column: Option<String>,
    pub parameter: Option<String>,
    pub transformation: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
struct LineConfig {
    pub name: String,
    pub site: Option<String>,
    pub unite: Option<String>,
    pub flag_dec: Option<String>,
    pub code_ligne: Option<String>,
    pub log_path: Option<String>,
    pub file_format: Option<String>,
    pub rejected_path: Option<String>,
}

/// Disk logger for per-line log files (matches Python Logger class)
struct DiskLogger;

impl DiskLogger {
    fn log_ligne(line_name: &str, log_path: &Option<String>, message: &str, log_type: &str) {
        let log_dir = match log_path {
            Some(p) if !p.is_empty() => p,
            _ => return,
        };

        if let Err(e) = fs::create_dir_all(log_dir) {
            eprintln!("Failed to create log dir: {}", e);
            return;
        }

        let log_file = format!(
            "{}/{}_{}.log",
            log_dir,
            line_name,
            Local::now().format("%Y%m")
        );

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let entry = format!("[{}] [{}] {}\n", timestamp, log_type, message);

        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
        {
            let _ = file.write_all(entry.as_bytes());
        }
    }

    fn log_sql(
        line_name: &str,
        log_path: &Option<String>,
        query: &str,
        values: &str,
        success: bool,
        error_msg: &str,
    ) {
        let log_dir = match log_path {
            Some(p) if !p.is_empty() => p,
            _ => return,
        };

        if let Err(e) = fs::create_dir_all(log_dir) {
            eprintln!("Failed to create log dir: {}", e);
            return;
        }

        let sql_log_file = format!(
            "{}/{}_sql_{}.log",
            log_dir,
            line_name,
            Local::now().format("%Y%m")
        );

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let status = if success { "SUCCESS" } else { "ERROR" };

        let mut entry = format!("\n[{}] [{}]\n", timestamp, status);
        entry.push_str(&format!(
            "Requête: {}\n",
            if query.len() > 500 {
                &query[..500]
            } else {
                query
            }
        ));
        entry.push_str(&format!("Valeurs: {}\n", values));
        if !success && !error_msg.is_empty() {
            entry.push_str(&format!("Erreur: {}\n", error_msg));
        }
        entry.push_str(&"-".repeat(80));
        entry.push('\n');

        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&sql_log_file)
        {
            let _ = file.write_all(entry.as_bytes());
        }
    }
}

/// Read file with multiple encoding attempts (like Python's encoding fallback)
fn read_file_with_encoding_fallback(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;

    // Try UTF-8 first
    if let Ok(s) = std::str::from_utf8(&bytes) {
        return Ok(s.to_string());
    }

    // Try UTF-8 with BOM handling
    let (cow, _, had_errors) = UTF_8.decode(&bytes);
    if !had_errors {
        return Ok(cow.into_owned());
    }

    // Fallback to Windows-1252 (cp1252) which is a superset of ISO-8859-1
    let (cow, _, _) = WINDOWS_1252.decode(&bytes);
    Ok(cow.into_owned())
}

impl StockProcessor {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Load line configuration for parameter resolution
    async fn load_line_config(&self, line_id: i64) -> Option<LineConfig> {
        let row = sqlx::query(
            "SELECT name, site, unite, flag_dec, code_ligne, log_path, file_format, rejected_path 
             FROM lines WHERE id = ?"
        )
        .bind(line_id)
        .fetch_optional(&self.pool)
        .await
        .ok()??;

        Some(LineConfig {
            name: row.get("name"),
            site: row.get("site"),
            unite: row.get("unite"),
            flag_dec: row.get("flag_dec"),
            code_ligne: row.get("code_ligne"),
            log_path: row.get("log_path"),
            file_format: row.get("file_format"),
            rejected_path: row.get("rejected_path"),
        })
    }

    /// Update line statistics after processing
    async fn update_line_stats(&self, line_id: i64, success: bool) {
        let now_dt = Local::now();
        let today = now_dt.date_naive();
        let now_str = now_dt.format("%Y-%m-%d %H:%M:%S").to_string();

        // Fetch current counters and last_file_time to detect day change
        let mut total_traites: i64 = 0;
        let mut total_erreurs: i64 = 0;
        let mut last_date: Option<chrono::NaiveDate> = None;

        if let Ok(row) = sqlx::query("SELECT total_traites, total_erreurs, last_file_time FROM lines WHERE id = ?")
            .bind(line_id)
            .fetch_one(&self.pool)
            .await
        {
            total_traites = row.get::<i64, _>("total_traites");
            total_erreurs = row.get::<i64, _>("total_erreurs");
            if let Ok(ts) = row.try_get::<String, _>("last_file_time") {
                if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M:%S") {
                    last_date = Some(parsed.date());
                }
            }
        }

        // Reset daily counters if date changed (or missing last date)
        if last_date.map(|d| d != today).unwrap_or(true) {
            total_traites = 0;
            total_erreurs = 0;
        }

        if success {
            total_traites += 1;
        } else {
            total_erreurs += 1;
        }

        let status = if success { "MARCHE" } else { "ERREUR" };

        let _ = sqlx::query(
            "UPDATE lines SET total_traites = ?, total_erreurs = ?, last_file_time = ?, etat_actuel = ? WHERE id = ?"
        )
        .bind(total_traites)
        .bind(total_erreurs)
        .bind(now_str)
        .bind(status)
        .bind(line_id)
        .execute(&self.pool)
        .await;
    }

    /// Add log entry to database
    async fn add_db_log(&self, line_id: i64, level: &str, source: &str, message: &str, details: Option<&str>) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let _ = sqlx::query(
            "INSERT INTO logs (line_id, level, source, message, details, created_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(line_id)
        .bind(level)
        .bind(source)
        .bind(message)
        .bind(details)
        .bind(now)
        .execute(&self.pool)
        .await;
    }

    pub async fn process_file(&self, line_id: i64, path: PathBuf, prefix: String, archived_path: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let filename = path.file_name().unwrap().to_str().unwrap().to_string();
        
        // Basic check for prefix and allowed extensions (TMP, CSV, TXT)
        let upper = filename.to_uppercase();
        let allowed_ext = upper.ends_with(".TMP") || upper.ends_with(".CSV") || upper.ends_with(".TXT");
        if !upper.contains(&prefix.to_uppercase()) || !allowed_ext {
            return Ok(());
        }

        // Load line config for parameters and logging
        let line_config = self.load_line_config(line_id).await;
        let line_name = line_config.as_ref().map(|c| c.name.clone()).unwrap_or_else(|| format!("line_{}", line_id));
        let log_path = line_config.as_ref().and_then(|c| c.log_path.clone());

        // Wait a bit to ensure file is completely written (like the Python time.sleep(1))
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Check if file is locked
        if is_file_locked(&path) {
            DiskLogger::log_ligne(&line_name, &log_path, &format!("Fichier {} en cours d'utilisation", filename), "WARNING");
            return Ok(());
        }

        // Read file with encoding fallback
        let content = match read_file_with_encoding_fallback(&path) {
            Ok(c) => c,
            Err(e) => {
                let msg = format!("Erreur lecture fichier {}: {}", filename, e);
                DiskLogger::log_ligne(&line_name, &log_path, &msg, "ERROR");
                self.add_db_log(line_id, "ERROR", "FileProcessor", &msg, None).await;
                self.update_line_stats(line_id, false).await;
                return Err(e.into());
            }
        };

        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b';')
            .has_headers(false)
            .flexible(true)
            .from_reader(content.as_bytes());

        // Load mappings for this line (global model mappings by file_format)
        let format_name = line_config
            .as_ref()
            .and_then(|l| l.file_format.clone())
            .unwrap_or_else(|| "ATEIS".to_string());

        let mappings = sqlx::query_as::<_, MappingRow>(
            "SELECT id, 0 as line_id, sort_order, sql_field, file_column, parameter, transformation, description \
             FROM model_mappings WHERE format_name = ? ORDER BY sort_order ASC, id ASC",
        )
        .bind(format_name.to_uppercase())
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let mut row_count = 0;
        let mut first_mapped: Option<serde_json::Value> = None;
        let mut had_error = false;
        let mut error_msg: Option<String> = None;
        let mut all_mapped_values: Vec<HashMap<String, String>> = Vec::new();

        if mappings.is_empty() {
            had_error = true;
            error_msg = Some(format!(
                "Aucun mapping configuré pour le modèle {}",
                format_name.to_uppercase()
            ));
        }

        for result in rdr.records() {
            let record = match result {
                Ok(r) => r,
                Err(e) => {
                    had_error = true;
                    error_msg = Some(e.to_string());
                    break;
                }
            };

            row_count += 1;

            // Map record with line config for parameter resolution
            let mapped = map_record_with_mappings_and_params(&record, &mappings, &line_config);
            if first_mapped.is_none() {
                first_mapped = Some(json!(mapped));
            }
            all_mapped_values.push(mapped);
        }

        // If no rows or error, mark as error
        if row_count == 0 {
            had_error = true;
            error_msg = Some("Fichier vide ou format invalide".to_string());
        }

        // Execute SQL Server inserts if enabled and we have mapped rows
        if !had_error && !all_mapped_values.is_empty() {
            if let Err(e) = self.execute_sql_server_inserts(line_config.as_ref(), &mappings, &all_mapped_values).await {
                let msg = format!("Erreur SQL Server pour {}: {}", filename, e);
                DiskLogger::log_sql(&line_name, &log_path, "(voir détails)", &serde_json::to_string(&all_mapped_values).unwrap_or_default(), false, &msg);
                self.add_db_log(line_id, "ERROR", "SQLServer", &msg, None).await;
                self.update_line_stats(line_id, false).await;
                had_error = true;
                error_msg = Some(msg);
            }
        }

        let status = if had_error { "ERROR" } else { "SUCCESS" };
        let message = json!({
            "rows": row_count,
            "sample": first_mapped,
            "error": error_msg,
        })
        .to_string();

        // Insert production data record reflecting final status
        let processed_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        sqlx::query(
            "INSERT INTO production_data (line_id, filename, status, message, processed_at) 
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(line_id)
        .bind(&filename)
        .bind(status)
        .bind(&message)
        .bind(&processed_at)
        .execute(&self.pool)
        .await?;

        // Update statistics
        self.update_line_stats(line_id, !had_error).await;

        // Log result
        if had_error {
            let msg = format!("Échec traitement {}: {}", filename, error_msg.as_deref().unwrap_or("unknown"));
            DiskLogger::log_ligne(&line_name, &log_path, &msg, "ERROR");
            self.add_db_log(line_id, "ERROR", "FileProcessor", &msg, None).await;

            // Move to rejected folder if configured
            if let Some(reject_dir) = line_config.as_ref().and_then(|c| c.rejected_path.clone()) {
                let reject_path = Path::new(&reject_dir);
                if !reject_path.exists() {
                    let _ = fs::create_dir_all(reject_path);
                }

                let new_filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("rejected.tmp")
                    .to_string();
                let dest_path = reject_path.join(new_filename);
                let _ = fs::rename(&path, dest_path);
            }
        } else {
            let msg = format!("Fichier {} traité avec succès - {} enregistrements", filename, row_count);
            DiskLogger::log_ligne(&line_name, &log_path, &msg, "INFO");
            self.add_db_log(line_id, "SUCCESS", "FileProcessor", &msg, None).await;

            // Archive only if processing succeeded
            if let Some(archive_dir) = archived_path {
                let archive_dir_path = Path::new(&archive_dir);
                if !archive_dir_path.exists() {
                    fs::create_dir_all(archive_dir_path)?;
                }
                
                let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
                let new_filename = format!("{}_{}{}", 
                    path.file_stem().unwrap().to_str().unwrap(),
                    timestamp,
                    path.extension().unwrap_or_default().to_str().map(|s| format!(".{}", s)).unwrap_or_default()
                );
                
                let dest_path = archive_dir_path.join(new_filename);
                fs::rename(&path, dest_path)?;
            }
        }

        Ok(())
    }
}

/// Check if file is locked (like Python's is_file_locked)
fn is_file_locked(path: &Path) -> bool {
    match fs::OpenOptions::new().read(true).write(true).open(path) {
        Ok(_) => false,
        Err(_) => true,
    }
}

fn get_file_value(record: &csv::StringRecord, file_column: &str) -> String {
    // Supports numeric columns ("0"), and combined columns ("1-2") which becomes "val1;val2".
    if let Some((a, b)) = file_column.split_once('-') {
        let a_idx = a.trim().parse::<usize>().ok();
        let b_idx = b.trim().parse::<usize>().ok();
        let v1 = a_idx.and_then(|i| record.get(i)).unwrap_or("").to_string();
        let v2 = b_idx.and_then(|i| record.get(i)).unwrap_or("").to_string();
        return format!("{};{}", v1, v2);
    }

    if let Ok(idx) = file_column.trim().parse::<usize>() {
        return record.get(idx).unwrap_or("").to_string();
    }

    "".to_string()
}

fn apply_transformation(value: String, transformation: &str) -> String {
    match transformation {
        "date" => {
            let v = value.trim();
            if v.is_empty() {
                return Local::now().format("%d/%m/%Y").to_string();
            }

            // Handle long digit strings like YYYYMMDDHHMMSSmmm by taking the date part first
            if v.chars().all(|c| c.is_ascii_digit()) && v.len() >= 8 {
                if let Ok(dt) = chrono::NaiveDate::parse_from_str(&v[0..8], "%Y%m%d") {
                    return dt.format("%d/%m/%Y").to_string();
                }
            }

            // Try multiple formats (same spirit as python)
            let formats = [
                "%d/%m/%Y",
                "%Y-%m-%d",
                "%d-%m-%Y",
                "%d.%m.%Y",
                "%d/%m/%y",
                "%d-%m-%y",
                "%d.%m.%y",
                "%Y%m%d",
            ];

            for fmt in formats {
                if let Ok(dt) = chrono::NaiveDate::parse_from_str(v, fmt) {
                    return dt.format("%d/%m/%Y").to_string();
                }
            }
            Local::now().format("%d/%m/%Y").to_string()
        }
        "heure" => {
            let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
            if digits.len() >= 14 {
                // Assume YYYYMMDDHHMMSS... take HHMMSS
                digits[8..14].to_string()
            } else if digits.len() >= 12 {
                // YYYYMMDDHHMM.. take HHMM and add 00
                format!("{}00", &digits[8..12])
            } else if digits.len() >= 6 {
                digits[..6].to_string()
            } else if digits.len() >= 4 {
                format!("{}00", &digits[..4])
            } else {
                "000000".to_string()
            }
        }
        "datetime" => {
            let v = value.trim();
            if v.is_empty() {
                return Local::now().format("%d/%m/%Y %H:%M:%S").to_string();
            }

            // Handle pure digit long format like YYYYMMDDHHMMSSmmm
            if v.chars().all(|c| c.is_ascii_digit()) {
                if v.len() >= 14 {
                    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&v[..14], "%Y%m%d%H%M%S") {
                        return dt.format("%d/%m/%Y %H:%M:%S").to_string();
                    }
                }
                if v.len() >= 8 {
                    if let Ok(d) = chrono::NaiveDate::parse_from_str(&v[..8], "%Y%m%d") {
                        let t_str = if v.len() >= 14 { &v[8..14] } else { "000000" };
                        if let Ok(t) = chrono::NaiveTime::parse_from_str(t_str, "%H%M%S") {
                            return chrono::NaiveDateTime::new(d, t).format("%d/%m/%Y %H:%M:%S").to_string();
                        }
                        return d.and_hms_opt(0, 0, 0).unwrap().format("%d/%m/%Y %H:%M:%S").to_string();
                    }
                }
            }

            let formats = [
                "%d/%m/%Y %H:%M:%S",
                "%d/%m/%Y %H:%M",
                "%Y-%m-%d %H:%M:%S",
                "%Y-%m-%d %H:%M",
                "%Y%m%d %H%M%S",
                "%Y%m%d%H%M%S",
                "%d/%m/%Y",
                "%Y-%m-%d",
                "%Y%m%d",
            ];

            for fmt in formats {
                if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(v, fmt) {
                    return dt.format("%d/%m/%Y %H:%M:%S").to_string();
                }
                if let Ok(d) = chrono::NaiveDate::parse_from_str(v, fmt) {
                    let dt = d.and_hms_opt(0, 0, 0)
                        .unwrap_or_else(|| chrono::NaiveDateTime::new(d, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()));
                    return dt.format("%d/%m/%Y %H:%M:%S").to_string();
                }
            }

            Local::now().format("%d/%m/%Y %H:%M:%S").to_string()
        }
        "decimal" => {
            let cleaned = value.replace(',', ".");
            let cleaned = cleaned
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                .collect::<String>();
            cleaned
        }
        "tinyint" => {
            let n = value.trim().parse::<i64>().unwrap_or(1);
            if n == 2 { "2".to_string() } else { "1".to_string() }
        }
        "current_datetime" => Local::now().format("%d/%m/%Y %H:%M:%S").to_string(),
        "datetime_combine" => {
            // Expects "date;time". Best-effort conversion to "DD/MM/YYYY HH:MM:SS".
            let parts: Vec<&str> = value.split(';').collect();
            if parts.len() < 2 {
                return Local::now().format("%d/%m/%Y %H:%M:%S").to_string();
            }
            let date_part = parts[0].trim();
            let time_part = parts[1].trim();

            let date_formats = ["%d/%m/%Y", "%Y-%m-%d", "%d-%m-%Y", "%d.%m.%Y", "%Y%m%d"];
            let time_formats = ["%H:%M:%S", "%H.%M.%S", "%H%M%S", "%H:%M", "%H.%M"]; 

            let mut date_obj: Option<chrono::NaiveDate> = None;
            let mut time_obj: Option<chrono::NaiveTime> = None;
            for fmt in date_formats {
                if let Ok(d) = chrono::NaiveDate::parse_from_str(date_part, fmt) {
                    date_obj = Some(d);
                    break;
                }
            }
            for fmt in time_formats {
                if let Ok(t) = chrono::NaiveTime::parse_from_str(time_part, fmt) {
                    time_obj = Some(t);
                    break;
                }
            }

            if let (Some(d), Some(t)) = (date_obj, time_obj) {
                chrono::NaiveDateTime::new(d, t).format("%d/%m/%Y %H:%M:%S").to_string()
            } else {
                Local::now().format("%d/%m/%Y %H:%M:%S").to_string()
            }
        }
        _ => value,
    }
}

fn apply_split(value: &str, part: &str) -> String {
    if let Some((before, after)) = value.split_once('+') {
        if part == "before" {
            return before.trim().chars().take(10).collect();
        }
        return after.trim().chars().take(10).collect();
    }
    if part == "before" {
        value.trim().chars().take(10).collect()
    } else {
        "".to_string()
    }
}

/// Get parameter value from line config
fn get_parameter_value(param: &str, line_config: &Option<LineConfig>) -> String {
    let config = match line_config {
        Some(c) => c,
        None => return String::new(),
    };

    match param.to_lowercase().as_str() {
        "site" | "fcy_0" => config.site.clone().unwrap_or_default(),
        "unite" | "uom_0" => config.unite.clone().unwrap_or_else(|| "unité".to_string()),
        "flag_dec" | "yflgdec_0" => config.flag_dec.clone().unwrap_or_else(|| "1".to_string()),
        "code_ligne" | "ynlign_0" => config.code_ligne.clone().unwrap_or_default(),
        "creusr_0" | "user" => "VISOR".to_string(),
        _ => String::new(),
    }
}

fn map_record_with_mappings_and_params(
    record: &csv::StringRecord,
    mappings: &[MappingRow],
    line_config: &Option<LineConfig>,
) -> HashMap<String, String> {
    let mut out = HashMap::new();

    for m in mappings {
        // Determine source: file column or parameter
        let mut value = if let Some(param) = &m.parameter {
            if !param.is_empty() {
                get_parameter_value(param, line_config)
            } else if let Some(file_col) = &m.file_column {
                if !file_col.is_empty() {
                    get_file_value(record, file_col)
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else if let Some(file_col) = &m.file_column {
            if !file_col.is_empty() {
                get_file_value(record, file_col)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Apply transformation
        if let Some(t) = &m.transformation {
            if !t.is_empty() {
                if t == "split_before_plus" {
                    value = apply_split(&value, "before");
                } else if t == "split_after_plus" {
                    value = apply_split(&value, "after");
                } else {
                    value = apply_transformation(value, t);
                }
            }
        }

        out.insert(m.sql_field.clone(), value);
    }

    out
}

fn scan_existing_files(path: &Path, prefix: &str) -> Vec<PathBuf> {
    let mut matches = Vec::new();

    if let Ok(read_dir) = fs::read_dir(path) {
        for entry in read_dir.flatten() {
            let p = entry.path();
            if !p.is_file() {
                continue;
            }

            let filename = match p.file_name().and_then(|s| s.to_str()) {
                Some(v) => v,
                None => continue,
            };

            let upper = filename.to_uppercase();
            let allowed_ext = upper.ends_with(".TMP") || upper.ends_with(".CSV") || upper.ends_with(".TXT");
            if allowed_ext && upper.contains(&prefix.to_uppercase()) {
                matches.push(p);
            }
        }
    }

    matches
}

pub fn start_watcher(app_handle: AppHandle, line_id: i64, path: String, prefix: String, archived_path: Option<String>) {
    let state = app_handle.state::<WatcherState>();

    {
        let watchers = state.watchers.lock().expect("watchers mutex poisoned");
        if watchers.contains_key(&line_id) {
            // Already running
            return;
        }
    }

    let pool = app_handle.state::<crate::db::DbState>().pool.clone();
    let processor = StockProcessor::new(pool);

    let (stop_tx, stop_rx) = mpsc::channel::<()>();

    {
        let mut watchers = state.watchers.lock().expect("watchers mutex poisoned");
        watchers.insert(line_id, WatcherHandle { stop_tx });
    }
    
    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher = notify::RecommendedWatcher::new(tx, Config::default()).expect("failed to create watcher");
        
        let watch_path = Path::new(&path);
        if !watch_path.exists() {
            eprintln!("Watch path does not exist: {}", path);
            return;
        }

        // Process already-existing files at startup (parity with Python).
        for p in scan_existing_files(watch_path, &prefix) {
            let pr = processor.pool.clone();
            let proc = StockProcessor::new(pr);
            let pref = prefix.clone();
            let arch = archived_path.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = proc.process_file(line_id, p, pref, arch).await {
                    eprintln!("Error processing existing file: {}", e);
                }
            });
        }

        watcher.watch(watch_path, RecursiveMode::NonRecursive).expect("failed to watch path");

        let mut last_scan = Instant::now();

        loop {
            if stop_rx.try_recv().is_ok() {
                break;
            }

            // Fallback poll to catch files when FS events are missed (e.g., network shares).
            if last_scan.elapsed() >= Duration::from_secs(5) {
                for p in scan_existing_files(watch_path, &prefix) {
                    let pr = processor.pool.clone();
                    let proc = StockProcessor::new(pr);
                    let pref = prefix.clone();
                    let arch = archived_path.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = proc.process_file(line_id, p, pref, arch).await {
                            eprintln!("Error processing polled file: {}", e);
                        }
                    });
                }
                last_scan = Instant::now();
            }

            let recv = match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(v) => v,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(e) => {
                    eprintln!("watch channel error: {:?}", e);
                    break;
                }
            };

            match recv {
                Ok(event) => {
                    if matches!(event.kind, notify::EventKind::Create(_) | notify::EventKind::Modify(_)) {
                        for path_buf in event.paths {
                            let p = path_buf.clone();
                            let pr = processor.pool.clone();
                            let proc = StockProcessor::new(pr);
                            let pref = prefix.clone();
                            let arch = archived_path.clone();

                            tauri::async_runtime::spawn(async move {
                                if let Err(e) = proc.process_file(line_id, p, pref, arch).await {
                                    eprintln!("Error processing file: {}", e);
                                }
                            });
                        }
                    }
                }
                Err(e) => {
                    eprintln!("watch error: {:?}", e);
                }
            }
        }
    });
}

pub fn stop_watcher(app_handle: AppHandle, line_id: i64) {
    let state = app_handle.state::<WatcherState>();
    let handle = {
        let mut watchers = state.watchers.lock().expect("watchers mutex poisoned");
        watchers.remove(&line_id)
    };

    if let Some(h) = handle {
        let _ = h.stop_tx.send(());
    }
}
