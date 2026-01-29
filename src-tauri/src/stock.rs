use notify::{Config, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::fs;
use std::time::Duration;
use sqlx::{Pool, Sqlite};
use tauri::AppHandle;
use tauri::Manager;
use chrono::Local;
use std::collections::HashMap;
use std::sync::{mpsc, Mutex};
use serde::Deserialize;
use serde_json::json;
use sqlx::FromRow;

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

impl StockProcessor {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn process_file(&self, line_id: i64, path: PathBuf, prefix: String, archived_path: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let filename = path.file_name().unwrap().to_str().unwrap();
        
        // Basic check for prefix and extension as in Python code
        if !filename.to_uppercase().contains(&prefix.to_uppercase()) || !filename.to_uppercase().ends_with(".TMP") {
            return Ok(());
        }

        // Wait a bit to ensure file is completely written (like the Python time.sleep(1))
        tokio::time::sleep(Duration::from_millis(500)).await;

        let content = fs::read_to_string(&path)?;
        let mut rdr = csv::ReaderBuilder::new().delimiter(b';').from_reader(content.as_bytes());

        // Load mappings for this line (if none, still count rows).
        let mappings = sqlx::query_as::<_, MappingRow>(
            "SELECT id, line_id, sort_order, sql_field, file_column, parameter, transformation, description \
             FROM mappings WHERE line_id = ? ORDER BY sort_order ASC, id ASC",
        )
        .bind(line_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let mut row_count = 0;
        let mut first_mapped: Option<serde_json::Value> = None;
        let mut had_error = false;
        let mut error_msg: Option<String> = None;

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

            if mappings.is_empty() {
                continue;
            }

            let mapped = map_record_with_mappings(&record, &mappings);
            if first_mapped.is_none() {
                first_mapped = Some(json!(mapped));
            }
        }

        let status = if had_error { "ERROR" } else { "SUCCESS" };
        let message = json!({
            "rows": row_count,
            "sample": first_mapped,
            "error": error_msg,
        })
        .to_string();

        sqlx::query(
            "INSERT INTO production_data (line_id, filename, status, message) 
             VALUES (?, ?, ?, ?)"
        )
        .bind(line_id)
        .bind(filename)
        .bind(status)
        .bind(message)
        .execute(&self.pool)
        .await?;

        // Archive only if processing succeeded
        if status == "SUCCESS" {
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
                return Local::now().format("%Y-%m-%d").to_string();
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
                    return dt.format("%Y-%m-%d").to_string();
                }
            }
            Local::now().format("%Y-%m-%d").to_string()
        }
        "heure" => {
            let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
            if digits.len() >= 6 {
                digits[..6].to_string()
            } else if digits.len() >= 4 {
                format!("{}00", &digits[..4])
            } else {
                "000000".to_string()
            }
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
        "current_datetime" => Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "datetime_combine" => {
            // Expects "date;time". Best-effort conversion to "YYYY-MM-DD HH:MM:SS".
            let parts: Vec<&str> = value.split(';').collect();
            if parts.len() < 2 {
                return Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
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
                chrono::NaiveDateTime::new(d, t).format("%Y-%m-%d %H:%M:%S").to_string()
            } else {
                Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
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

fn map_record_with_mappings(record: &csv::StringRecord, mappings: &[MappingRow]) -> HashMap<String, String> {
    let mut out = HashMap::new();

    for m in mappings {
        // Parameter mappings will be wired once line parameters exist in DB.
        let mut value = if let Some(file_col) = &m.file_column {
            get_file_value(record, file_col)
        } else {
            "".to_string()
        };

        if let Some(t) = &m.transformation {
            if t == "split_before_plus" {
                value = apply_split(&value, "before");
            } else if t == "split_after_plus" {
                value = apply_split(&value, "after");
            } else {
                value = apply_transformation(value, t);
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
            if upper.ends_with(".TMP") && upper.contains(&prefix.to_uppercase()) {
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

        loop {
            if stop_rx.try_recv().is_ok() {
                break;
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
