use crate::commands::sql_server::SqlServerConfig;
use crate::stock::encoding::read_file_with_encoding_fallback;
use crate::stock::fs_utils::is_file_locked;
use crate::stock::transforms::{apply_split, apply_transformation};
use chrono::Local;
use log::info;
use serde::Deserialize;
use serde_json::json;
use sqlx::{FromRow, Pool, Row, Sqlite};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tiberius::{AuthMethod, Client, Config as SqlConfig, ToSql};
use tokio_util::compat::TokioAsyncWriteCompatExt;

fn parse_insert_columns(query: &str) -> Vec<String> {
    let lower = query.to_lowercase();
    let insert_pos = lower.find("insert");
    if insert_pos.is_none() {
        return Vec::new();
    }

    let open_paren = query[insert_pos.unwrap()..]
        .find('(')
        .map(|i| i + insert_pos.unwrap());
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
        .map(|s| {
            s.trim()
                .trim_matches('[')
                .trim_matches(']')
                .trim()
                .to_string()
        })
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
            let _ = std::io::Write::write_all(&mut file, entry.as_bytes());
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
            let _ = std::io::Write::write_all(&mut file, entry.as_bytes());
        }
    }
}

impl StockProcessor {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub(crate) fn pool_clone(&self) -> Pool<Sqlite> {
        self.pool.clone()
    }

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

    async fn load_line_config(&self, line_id: i64) -> Option<LineConfig> {
        let row = sqlx::query(
            "SELECT name, site, unite, flag_dec, code_ligne, log_path, file_format, rejected_path \n             FROM lines WHERE id = ?",
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

    async fn update_line_stats(&self, line_id: i64, success: bool) {
        let now_dt = Local::now();
        let now_str = now_dt.format("%Y-%m-%d %H:%M:%S").to_string();
        let status = if success { "MARCHE" } else { "ERREUR" };

        // We no longer update total_traites/total_erreurs columns in the lines table
        // as they are calculated dynamically from production_data history
        let _ = sqlx::query("UPDATE lines SET last_file_time = ?, etat_actuel = ? WHERE id = ?")
            .bind(now_str)
            .bind(status)
            .bind(line_id)
            .execute(&self.pool)
            .await;
    }

    async fn add_db_log(
        &self,
        line_id: i64,
        level: &str,
        source: &str,
        message: &str,
        details: Option<&str>,
    ) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let _ = sqlx::query(
            "INSERT INTO logs (line_id, level, source, message, details, created_at) VALUES (?, ?, ?, ?, ?, ?)",
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

    fn is_connection_error(msg: &str) -> bool {
        let lower = msg.to_lowercase();
        lower.contains("login failed")
            || lower.contains("échec de l'ouverture de session")
            || lower.contains("impossible d'ouvrir la base de données")
            || lower.contains("la connexion a échoué")
            || lower.contains("connection")
            || lower.contains("network")
            || lower.contains("refused")
            || lower.contains("timeout")
            || lower.contains("tcp provider")
            || lower.contains("code: 4060") // Cannot open database
            || lower.contains("code: 18456") // Login failed
            || lower.contains("target machine actively refused")
    }

    pub async fn process_file(
        &self,
        line_id: i64,
        path: PathBuf,
        prefix: String,
        archived_path: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !path.exists() {
            return Ok(());
        }

        let filename = path.file_name().unwrap().to_str().unwrap().to_string();

        let upper = filename.to_uppercase();
        let allowed_ext =
            upper.ends_with(".TMP") || upper.ends_with(".CSV") || upper.ends_with(".TXT");
        if !upper.contains(&prefix.to_uppercase()) || !allowed_ext {
            return Ok(());
        }

        let line_config = self.load_line_config(line_id).await;
        let line_name = line_config
            .as_ref()
            .map(|c| c.name.clone())
            .unwrap_or_else(|| format!("line_{}", line_id));
        let log_path = line_config.as_ref().and_then(|c| c.log_path.clone());

        tokio::time::sleep(Duration::from_millis(500)).await;

        if is_file_locked(&path) {
            DiskLogger::log_ligne(
                &line_name,
                &log_path,
                &format!("Fichier {} en cours d'utilisation", filename),
                "WARNING",
            );
            return Ok(());
        }

        let content = match read_file_with_encoding_fallback(&path) {
            Ok(c) => c,
            Err(e) => {
                let msg = format!("Erreur lecture fichier {}: {}", filename, e);
                DiskLogger::log_ligne(&line_name, &log_path, &msg, "ERROR");
                self.add_db_log(line_id, "ERROR", "FileProcessor", &msg, None)
                    .await;
                self.update_line_stats(line_id, false).await;
                return Err(e.into());
            }
        };

        let source_parent = path.parent().unwrap_or_else(|| Path::new("."));
        let temp_subdir = source_parent.join("visor_temp");

        if let Err(e) = tokio::fs::create_dir_all(&temp_subdir).await {
            let msg = format!(
                "Impossible de créer dossier temp {}: {}",
                temp_subdir.display(),
                e
            );
            DiskLogger::log_ligne(&line_name, &log_path, &msg, "ERROR");
            self.add_db_log(line_id, "ERROR", "FileProcessor", &msg, None)
                .await;
            return Err(e.into());
        }

        let temp_filename = format!("visor_processing_{}_{}", line_id, filename);
        let temp_path = temp_subdir.join(&temp_filename);

        if let Err(e) = fs::rename(&path, &temp_path) {
            let msg = format!(
                "Impossible de déplacer le fichier {} vers {}: {}",
                filename,
                temp_path.display(),
                e
            );
            DiskLogger::log_ligne(&line_name, &log_path, &msg, "ERROR");
            self.add_db_log(line_id, "ERROR", "FileProcessor", &msg, None)
                .await;
            let _ = fs::remove_dir_all(&temp_subdir);
            return Err(e.into());
        }

        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b';')
            .has_headers(false)
            .flexible(true)
            .from_reader(content.as_bytes());

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

            let mapped = map_record_with_mappings_and_params(&record, &mappings, &line_config);
            if first_mapped.is_none() {
                first_mapped = Some(json!(mapped));
            }
            all_mapped_values.push(mapped);
        }

        if row_count == 0 {
            had_error = true;
            error_msg = Some("Fichier vide ou format invalide".to_string());
        }

        if !had_error && !all_mapped_values.is_empty() {
            if let Err(e) = self
                .execute_sql_server_inserts(line_config.as_ref(), &mappings, &all_mapped_values)
                .await
            {
                if Self::is_connection_error(&e) {
                    let msg = format!("Erreur connexion SQL (fichier reporté) : {}", e);
                    DiskLogger::log_ligne(&line_name, &log_path, &msg, "WARNING");
                    self.add_db_log(line_id, "WARNING", "SQLServer", &msg, None)
                        .await;

                    // Update line stats to ERROR for visual feedback
                    self.update_line_stats(line_id, false).await;

                    // Attempt to restore file
                    if let Err(restore_err) = fs::rename(&temp_path, &path) {
                        // Critical failure: cannot restore file. Must fallback to error folder to save data.
                        let crit_msg = format!(
                            "Echec restauration fichier après erreur connexion: {}",
                            restore_err
                        );
                        DiskLogger::log_ligne(&line_name, &log_path, &crit_msg, "CRITICAL");
                        // Let it fall through to standard error handling (move to rejected)
                        had_error = true;
                        error_msg = Some(format!("{} | {}", msg, crit_msg));
                    } else {
                        // File restored successfully.
                        // Clean up temp dir if empty
                        if temp_subdir.read_dir().unwrap().count() == 0 {
                            let _ = fs::remove_dir(&temp_subdir);
                        }
                        return Ok(());
                    }
                } else {
                    let msg = format!("Erreur SQL Server pour {}: {}", filename, e);
                    DiskLogger::log_sql(
                        &line_name,
                        &log_path,
                        "(voir détails)",
                        &serde_json::to_string(&all_mapped_values).unwrap_or_default(),
                        false,
                        &msg,
                    );
                    self.add_db_log(line_id, "ERROR", "SQLServer", &msg, None)
                        .await;
                    self.update_line_stats(line_id, false).await;
                    had_error = true;
                    error_msg = Some(msg);
                }
            }
        }

        // If we had a connection error that successfully restored, we returned early.
        // If we are here, it's either success or a different error/restore failed.

        let status = if had_error { "ERROR" } else { "SUCCESS" };
        let message = json!({
            "rows": row_count,
            "sample": first_mapped,
            "error": error_msg,
        })
        .to_string();

        let processed_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        sqlx::query(
            "INSERT INTO production_data (line_id, filename, status, message, processed_at) \n         VALUES (?, ?, ?, ?, ?)",
        )
        .bind(line_id)
        .bind(&filename)
        .bind(status)
        .bind(&message)
        .bind(&processed_at)
        .execute(&self.pool)
        .await?;

        self.update_line_stats(line_id, !had_error).await;

        if had_error {
            let msg = format!(
                "Échec traitement {}: {}",
                filename,
                error_msg.as_deref().unwrap_or("unknown")
            );
            DiskLogger::log_ligne(&line_name, &log_path, &msg, "ERROR");
            self.add_db_log(line_id, "ERROR", "FileProcessor", &msg, None)
                .await;

            if let Some(reject_dir) = line_config.as_ref().and_then(|c| c.rejected_path.clone()) {
                let reject_path = Path::new(&reject_dir);
                if !reject_path.exists() {
                    let _ = fs::create_dir_all(reject_path);
                }

                let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
                let new_filename = format!(
                    "{}_{}{}",
                    Path::new(&filename).file_stem().unwrap().to_str().unwrap(),
                    timestamp,
                    Path::new(&filename)
                        .extension()
                        .unwrap_or_default()
                        .to_str()
                        .map(|s| format!(".{}", s))
                        .unwrap_or_default()
                );
                let dest_path = reject_path.join(new_filename);

                if let Err(e) = fs::rename(&temp_path, &dest_path) {
                    eprintln!("Failed to move to rejected folder: {}", e);
                    let _ = fs::remove_file(&temp_path);
                }
            } else {
                let _ = fs::remove_file(&temp_path);
            }
        } else {
            let msg = format!(
                "Fichier {} traité avec succès - {} enregistrements",
                filename, row_count
            );
            DiskLogger::log_ligne(&line_name, &log_path, &msg, "INFO");
            self.add_db_log(line_id, "SUCCESS", "FileProcessor", &msg, None)
                .await;

            if let Some(archive_dir) = archived_path {
                let archive_dir_path = Path::new(&archive_dir);
                if !archive_dir_path.exists() {
                    if let Err(e) = fs::create_dir_all(archive_dir_path) {
                        eprintln!("Failed to create archive directory: {}", e);
                        let _ = fs::remove_file(&temp_path);
                        return Ok(());
                    }
                }

                let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
                let new_filename = format!(
                    "{}_{}{}",
                    Path::new(&filename).file_stem().unwrap().to_str().unwrap(),
                    timestamp,
                    Path::new(&filename)
                        .extension()
                        .unwrap_or_default()
                        .to_str()
                        .map(|s| format!(".{}", s))
                        .unwrap_or_default()
                );

                let dest_path = archive_dir_path.join(new_filename);
                if let Err(e) = fs::rename(&temp_path, &dest_path) {
                    eprintln!("Failed to archive file: {}", e);
                    let _ = fs::remove_file(&temp_path);
                }
            } else {
                let _ = fs::remove_file(&temp_path);
            }
        }

        if temp_subdir.read_dir().unwrap().count() == 0 {
            let _ = fs::remove_dir(&temp_subdir);
        }

        Ok(())
    }
}

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

fn get_file_value(record: &csv::StringRecord, file_column: &str) -> String {
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

fn map_record_with_mappings_and_params(
    record: &csv::StringRecord,
    mappings: &[MappingRow],
    line_config: &Option<LineConfig>,
) -> HashMap<String, String> {
    let mut out = HashMap::new();

    for m in mappings {
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
