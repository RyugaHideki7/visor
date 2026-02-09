use crate::db::DbState;
use odbc_api::{ConnectionOptions, Cursor, Environment};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tauri::{AppHandle, Emitter, Manager, State};
use std::io::Write;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct HfsqlConfig {
    pub id: i64,
    pub dsn: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub log_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HfsqlConnectionTestResult {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Clone, Serialize)]
struct SyncProgress {
    current: usize,
    total: usize,
    status: String,
}

#[tauri::command]
pub async fn get_hfsql_config(state: State<'_, DbState>) -> Result<HfsqlConfig, String> {
    let row = sqlx::query_as::<_, HfsqlConfig>(
        "SELECT id, dsn, username, password, log_path FROM hfsql_config WHERE id = 1",
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(row)
}

#[tauri::command]
pub async fn save_hfsql_config(
    state: State<'_, DbState>,
    dsn: String,
    username: String,
    password: String,
    log_path: String,
) -> Result<(), String> {
    sqlx::query("UPDATE hfsql_config SET dsn = ?, username = ?, password = ?, log_path = ? WHERE id = 1")
        .bind(&dsn)
        .bind(&username)
        .bind(&password)
        .bind(&log_path)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn do_test_connection(dsn: String, user: String, pwd: String) -> HfsqlConnectionTestResult {
    let env = match Environment::new() {
        Ok(e) => e,
        Err(e) => {
            return HfsqlConnectionTestResult {
                success: false,
                error: Some(e.to_string()),
            };
        }
    };

    let conn_string = format!("DSN={};UID={};PWD={};", dsn, user, pwd);

    let conn = match env.connect_with_connection_string(&conn_string, ConnectionOptions::default())
    {
        Ok(c) => c,
        Err(e) => {
            return HfsqlConnectionTestResult {
                success: false,
                error: Some(e.to_string()),
            };
        }
    };
    let result = match conn.execute("SELECT Count(*) FROM Article WHERE 1=0", ()) {
        Ok(cursor) => {
            // Drop the cursor immediately to release borrows
            drop(cursor);
            HfsqlConnectionTestResult {
                success: true,
                error: None,
            }
        }
        Err(e) => HfsqlConnectionTestResult {
            success: false,
            error: Some(e.to_string()),
        },
    };

    result
}

#[tauri::command]
pub async fn test_hfsql_connection(
    state: State<'_, DbState>,
) -> Result<HfsqlConnectionTestResult, String> {
    let cfg = get_hfsql_config(state).await?;

    let dsn = cfg.dsn.unwrap_or_default();
    let user = cfg.username.unwrap_or_default();
    let pwd = cfg.password.unwrap_or_default();

    if dsn.trim().is_empty() {
        return Ok(HfsqlConnectionTestResult {
            success: false,
            error: Some("DSN manquant".to_string()),
        });
    }

    // ODBC calls are blocking - run in blocking thread
    let result = tokio::task::spawn_blocking(move || do_test_connection(dsn, user, pwd))
        .await
        .map_err(|e| e.to_string())?;

    Ok(result)
}

#[derive(Debug, Serialize)]
pub struct ArticleSyncResult {
    pub total_processed: i64,
    pub updated: i64,
    pub inserted: i64,
    pub errors: i64,
    #[serde(rename = "details")]
    pub error_details: Vec<String>,
}

#[tauri::command]
pub async fn sync_ateis_produit(app: AppHandle, state: State<'_, DbState>) -> Result<ArticleSyncResult, String> {
    use crate::commands::sql_queries::get_or_init_sql_query;
    use crate::commands::sql_server::get_sql_server_config;
    use futures_util::TryStreamExt;
    use tiberius::{AuthMethod, Client, Config as SqlConfig, QueryItem};
    use tokio_util::compat::TokioAsyncWriteCompatExt;

    // 1. Get Global SQL Server Config (Source) - IGNORE ENABLED FLAG
    let sql_cfg = get_sql_server_config(state.clone()).await?;
    let sql_host = sql_cfg.server.unwrap_or_default();
    let sql_db = sql_cfg.database.unwrap_or_default();
    let sql_user = sql_cfg.username.unwrap_or_default();
    let sql_pwd = sql_cfg.password.unwrap_or_default();

    if sql_host.trim().is_empty() {
        return Err("Configuration SQL Server manquante (voir Paramètres)".to_string());
    }

    // 2. Get Global HFSQL Config (Destination)
    let hfsql_cfg = get_hfsql_config(state.clone()).await?;
    let dsn = hfsql_cfg.dsn.unwrap_or_default();
    let user = hfsql_cfg.username.unwrap_or_default();
    let pwd = hfsql_cfg.password.unwrap_or_default();
    
    // Resolve Log Directory
    let log_dir_base = match hfsql_cfg.log_path {
        Some(path) if !path.is_empty() => path,
        _ => {
            // Default to Desktop/T/BLOG if not configured
            app.path()
                .desktop_dir()
                .ok()
                .map(|p| p.join("T").join("BLOG").to_string_lossy().to_string())
                .unwrap_or_else(|| r"C:\T\BLOG".to_string())
        }
    };

    if dsn.trim().is_empty() {
        return Err("Configuration HFSQL globale manquante (DSN)".to_string());
    }

    // 3. Connect to SQL Server (Independent)
    let mut tiberius_config = SqlConfig::new();
    tiberius_config.host(sql_host.as_str());
    tiberius_config.port(1433);
    tiberius_config.authentication(AuthMethod::sql_server(sql_user, sql_pwd));
    tiberius_config.trust_cert();
    if !sql_db.trim().is_empty() {
        tiberius_config.database(sql_db);
    }

    let tcp = tokio::net::TcpStream::connect(tiberius_config.get_addr())
        .await
        .map_err(|e| format!("Erreur connexion SQL Server: {}", e))?;
    tcp.set_nodelay(true).map_err(|e| e.to_string())?;

    let mut sql_client = Client::connect(tiberius_config, tcp.compat_write())
        .await
        .map_err(|e| format!("Erreur auth SQL Server: {}", e))?;

    // 3. Fetch articles from SQL Server using ATEIS_PRODUIT query
    let query = get_or_init_sql_query(
        &state.pool,
        "ATEIS_PRODUIT",
        crate::commands::sql_queries::DEFAULT_ATEIS_PRODUIT_QUERY,
    )
    .await?;

    let mut stream = sql_client
        .query(query.as_str(), &[])
        .await
        .map_err(|e| e.to_string())?;

    // Collect articles from SQL Server
    let mut articles = Vec::new();
    while let Some(item) = stream.try_next().await.map_err(|e| e.to_string())? {
        let row = match item {
            QueryItem::Row(r) => r,
            _ => continue,
        };

        let get_str = |idx: usize| -> String {
            row.try_get::<&str, _>(idx)
                .ok()
                .flatten()
                .map(|s| s.to_string())
                .unwrap_or_default()
        };
        let get_dec = |idx: usize| -> f64 {
            row.try_get::<rust_decimal::Decimal, _>(idx)
                .ok()
                .flatten()
                .and_then(|d| d.to_string().parse().ok())
                .or_else(|| row.try_get::<f64, _>(idx).ok().flatten())
                .or_else(|| row.try_get::<i64, _>(idx).ok().flatten().map(|i| i as f64))
                .unwrap_or(0.0)
        };
        let get_int = |idx: usize| -> i32 {
            row.try_get::<i32, _>(idx)
                .ok()
                .flatten()
                .or_else(|| row.try_get::<i64, _>(idx).ok().flatten().map(|i| i as i32))
                .unwrap_or(0)
        };

        articles.push((
            get_str(0), // EAN13_Fardeau
            get_str(1), // Designation
            get_str(2), // CodeArt
            get_dec(3), // PoidsFardeau
            get_int(4), // NbFardeauxPal
            get_str(5), // DecalDLUO
            get_str(6), // EAN_Palette
            get_str(7), // DesignArabe
            get_str(8), // EAN_Palette_Export
        ));
    }

    let total_articles = articles.len() as i64;
    let app_handle = app.clone();

    // 4. Process articles - upsert into HFSQL
    let dsn_clone = dsn.clone();
    let user_clone = user.clone();
    let pwd_clone = pwd.clone();

    // Logging setup
    let log_dir = log_dir_base.clone();
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Failed to create log directory: {}", e);
    }
    let now = chrono::Local::now();
    let log_filename = format!("transfer_articles_{}.log", now.format("%Y%m%d_%H%M%S"));
    let log_path = std::path::Path::new(&log_dir).join(log_filename);

    let log_path_console = log_path.clone();
    let log_msg = move |msg: &str| {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let line = format!("{} - INFO - {}\n", timestamp, msg);
        println!("{}", msg); // Print to console
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path_console)
        {
            let _ = file.write_all(line.as_bytes());
        }
    };
    
    // Clone for the closure
    let log_path_clone = log_path.clone();

    // Log Start
    {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let header = format!(
            "{}\nTRANSFERT ARTICLES SQL Server -> HFSQL\nMode: UPSERT (UPDATE/INSERT)\nDébut: {}\n{}\n\n",
            "=".repeat(70),
            timestamp,
            "=".repeat(70)
        );
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
             let _ = file.write_all(header.as_bytes());
        }
    }
    log_msg("DEBUT DU TRANSFERT DES ARTICLES");
    log_msg("Mode: UPSERT (UPDATE si existe, INSERT sinon)");
    log_msg("Cle: CodeArt");
    log_msg(&format!("1. Recuperation des articles depuis SQL Server...\n[OK] {} articles recuperes", articles.len()));
    log_msg(&format!("\n2. Traitement UPSERT de {} articles...", articles.len()));

    let result = tokio::task::spawn_blocking(move || {
        let mut updated = 0i64;
        let mut inserted = 0i64;
        let mut errors = 0i64;
        let mut error_details = Vec::new();
        let mut last_progress_emit = std::time::Instant::now();
        
        // Helper for logging inside the thread
        let log_msg_inner = |msg:String| {
             let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
             let line = format!("{} - INFO - {}\n", timestamp, msg);
             println!("{}", msg);
             if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path_clone)
            {
                let _ = file.write_all(line.as_bytes());
            }
        };

        // Initialize ODBC Environment once
        let env = match Environment::new() {
            Ok(e) => e,
            Err(e) => {
                let err_msg = format!("Env init error: {}", e);
                log_msg_inner(format!("[ERROR] {}", err_msg));
                return ArticleSyncResult {
                    total_processed: total_articles,
                    updated: 0,
                    inserted: 0,
                    errors: total_articles,
                    error_details: vec![err_msg],
                };
            }
        };

        // Create Connection ONCE (Persistent connection)
        let conn_string = format!("DSN={};UID={};PWD={};", dsn_clone, user_clone, pwd_clone);
        
        let conn = match env.connect_with_connection_string(&conn_string, ConnectionOptions::default()) {
            Ok(c) => c,
            Err(e) => {
                 let err_msg = format!("Connection init error: {}", e);
                 log_msg_inner(format!("[ERROR] {}", err_msg));
                 return ArticleSyncResult {
                    total_processed: total_articles,
                    updated: 0,
                    inserted: 0,
                    errors: total_articles,
                    error_details: vec![err_msg],
                };
            }
        };

        for (i, article) in articles.iter().enumerate() {
            let code_art = &article.2;
            if code_art.trim().is_empty() {
                continue;
            }

            // Check if article exists
            // Use SELECT CodeArt instead of COUNT(*) to avoid type mapping issues
            let check_query = format!("SELECT CodeArt FROM Article WHERE CodeArt = '{}'", code_art);
            
            let exists = match conn.execute(&check_query, ()) {
                Ok(Some(mut cursor)) => {
                    // If we can get a row, it exists
                    cursor.next_row().ok().flatten().is_some()
                },
                Ok(None) => false, // No cursor returned (should shouldn't happen for SELECT)
                Err(e) => {
                    log_msg_inner(format!("Check query failed for {}: {}", code_art, e));
                    false
                }
            };

            // Prepare values (Sanitize strings)
            let v_ean13 = article.0.replace("'", "''");
            let v_designation = article.1.replace("'", "''");
            let v_code_art = article.2.replace("'", "''");
            let v_poids = article.3; // f64
            let v_nb = article.4; // i32
            let v_decal = article.5.replace("'", "''");
            let v_ean_pal = article.6.replace("'", "''");
            let v_design_arabe = article.7.replace("'", "''");
            let v_ean_exp = article.8.replace("'", "''");

            if exists {
                let sql = format!(
                    "UPDATE Article SET EAN13_Fardeau='{}', Designation='{}', PoidsFardeau={}, \
                     NbFardeauxPal={}, DecalDLUO='{}', EAN_Palette='{}', DesignArabe='{}', \
                     EAN_Palette_Export='{}' WHERE CodeArt='{}'",
                    v_ean13, v_designation, v_poids, v_nb, v_decal, v_ean_pal, v_design_arabe, v_ean_exp, v_code_art
                );

                match conn.execute(&sql, ()) {
                    Ok(_) => {
                        updated += 1;
                        // if updated <= 3 { log_msg_inner(format!("    UPDATE: {}", code_art)); }
                    }
                    Err(e) => {
                        errors += 1;
                        if error_details.len() < 10 { error_details.push(format!("{}: {}", code_art, e)); }
                        log_msg_inner(format!("[ERROR] UPDATE {}: {}", code_art, e));
                    }
                }
            } else {
                let sql = format!(
                    "INSERT INTO Article (EAN13_Fardeau, Designation, CodeArt, PoidsFardeau, \
                     NbFardeauxPal, DecalDLUO, EAN_Palette, DesignArabe, EAN_Palette_Export) \
                     VALUES ('{}', '{}', '{}', {}, {}, '{}', '{}', '{}', '{}')",
                    v_ean13, v_designation, v_code_art, v_poids, v_nb, v_decal, v_ean_pal, v_design_arabe, v_ean_exp
                );

                match conn.execute(&sql, ()) {
                    Ok(_) => {
                        inserted += 1;
                        if inserted <= 3 { log_msg_inner(format!("    INSERT: {}", code_art)); }
                    }
                    Err(e) => {
                         errors += 1;
                         if error_details.len() < 10 { error_details.push(format!("{}: {}", code_art, e)); }
                         log_msg_inner(format!("[ERROR] INSERT {}: {}", code_art, e));
                    }
                }
            };
            
            // Emit progress event every 50ms or every item if slow
            if last_progress_emit.elapsed().as_millis() > 50 {
                let _ = app_handle.emit("ateis-produit-sync-progress", SyncProgress {
                    current: i + 1,
                    total: articles.len(),
                    status: format!("Processing: {}", code_art),
                });
                last_progress_emit = std::time::Instant::now();
            }

            if (i + 1) % 50 == 0 {
                log_msg_inner(format!("  Progression: {}/{}", i + 1, total_articles));
            }
        }
        
        // Final progress emit
        let _ = app_handle.emit("ateis-produit-sync-progress", SyncProgress {
            current: articles.len(),
            total: articles.len(),
            status: "Completed".to_string(),
        });

        // Final Report Log
         let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
         let footer = format!(
            "\n{}\n[RAPPORT FINAL] TRANSFERT ARTICLES (UPSERT)\n{}\nTotal articles traites: {}\nArticles mis a jour: {}\nNouveaux articles inseres: {}\nErreurs de traitement: {}\n\n{}\nSTATUT: {}\nFIN: {}\n{}\n",
            "=".repeat(70),
            "=".repeat(70),
            total_articles,
            updated,
            inserted,
            errors,
            "=".repeat(70),
            if errors < total_articles { "SUCCES" } else { "ECHEC" },
            timestamp,
            "=".repeat(70)
        );
        log_msg_inner(footer);

        let final_result = ArticleSyncResult {
            total_processed: total_articles,
            updated,
            inserted,
            errors,
            error_details,
        };

        // Emit the final result
        let _ = app_handle.emit("ateis-produit-sync-result", &final_result);

        final_result
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(result)
}

#[tauri::command]
pub async fn sync_ateis_of(app: AppHandle, state: State<'_, DbState>) -> Result<ArticleSyncResult, String> {
    use crate::commands::sql_queries::get_or_init_sql_query;
    use crate::commands::sql_server::get_sql_server_config;
    use tiberius::{AuthMethod, Client, Config as SqlConfig};
    use tokio_util::compat::TokioAsyncWriteCompatExt;
    use chrono::NaiveDateTime;
    use rust_decimal::Decimal;
    use rust_decimal::prelude::ToPrimitive;

    // 1. Get Global SQL Server Config (Source)
    let sql_cfg = get_sql_server_config(state.clone()).await?;
    let sql_host = sql_cfg.server.unwrap_or_default();
    let sql_db = sql_cfg.database.unwrap_or_default();
    let sql_user = sql_cfg.username.unwrap_or_default();
    let sql_pwd = sql_cfg.password.unwrap_or_default();

    if sql_host.trim().is_empty() {
        return Err("Configuration SQL Server manquante (voir Paramètres)".to_string());
    }

    // 2. Get Global HFSQL Config (Destination)
    let hfsql_cfg = get_hfsql_config(state.clone()).await?;
    let dsn = hfsql_cfg.dsn.unwrap_or_default();
    let user = hfsql_cfg.username.unwrap_or_default();
    let pwd = hfsql_cfg.password.unwrap_or_default();
    
    // Resolve Log Directory
    let log_dir_base = match hfsql_cfg.log_path {
        Some(path) if !path.is_empty() => path,
        _ => {
            app.path()
                .desktop_dir()
                .ok()
                .map(|p| p.join("T").join("BLOG").to_string_lossy().to_string())
                .unwrap_or_else(|| r"C:\T\BLOG".to_string())
        }
    };

    if dsn.trim().is_empty() {
        return Err("Configuration HFSQL globale manquante (DSN)".to_string());
    }

    // 3. Connect to SQL Server
    let mut tiberius_config = SqlConfig::new();
    tiberius_config.host(sql_host.as_str());
    tiberius_config.port(1433);
    tiberius_config.authentication(AuthMethod::sql_server(sql_user, sql_pwd));
    tiberius_config.trust_cert();
    if !sql_db.trim().is_empty() {
        tiberius_config.database(sql_db);
    }

    let tcp = tokio::net::TcpStream::connect(tiberius_config.get_addr())
        .await
        .map_err(|e| format!("Erreur connexion SQL Server: {}", e))?;
    tcp.set_nodelay(true).map_err(|e| e.to_string())?;

    let mut sql_client = Client::connect(tiberius_config, tcp.compat_write())
        .await
        .map_err(|e| format!("Erreur auth SQL Server: {}", e))?;

    // 4. Fetch OFs
    let query = get_or_init_sql_query(
        &state.pool,
        "ATEIS_OF",
        crate::commands::sql_queries::DEFAULT_ATEIS_OF_QUERY,
    )
    .await?;

    let stream = sql_client.query(query, &[]).await.map_err(|e| e.to_string())?;
    let rows = stream.into_first_result().await.map_err(|e| e.to_string())?;

    let mut of_list = Vec::new();
    for row in rows {
        let mfg_num: &str = row.get(0).unwrap_or("");
        let ligne_of: &str = row.get(1).unwrap_or("");
        let itm_ref: &str = row.get(2).unwrap_or("");
        // Robust handling for ext_qty: try f64, then fallback to Decimal
        let ext_qty: f64 = match row.try_get::<f64, _>(3) {
            Ok(Some(val)) => val,
            Ok(None) => 0.0,
            Err(_) => match row.try_get::<Decimal, _>(3) {
                Ok(Some(d)) => d.to_f64().unwrap_or(0.0),
                Ok(None) => 0.0,
                Err(_) => 0.0,
            },
        };
        let desc: &str = row.get(4).unwrap_or("");
        let str_dat: Option<NaiveDateTime> = row.get(5);
        let end_dat: Option<NaiveDateTime> = row.get(6);

        // Date format YYYYMMDD as integers
        let date_debut = str_dat.map(|d| d.format("%Y%m%d").to_string().parse::<i32>().unwrap_or(0)).unwrap_or(0);
        let date_fin = end_dat.map(|d| d.format("%Y%m%d").to_string().parse::<i32>().unwrap_or(0)).unwrap_or(0);

        of_list.push((
            mfg_num.to_string(),
            ligne_of.to_string(),
            itm_ref.to_string(),
            ext_qty,
            desc.to_string(),
            date_debut,
            date_fin
        ));
    }

    // 5. Connect to HFSQL via ODBC (Blocking)
    let total_of = of_list.len();
    let dsn_clone = dsn.clone();
    let user_clone = user.clone();
    let pwd_clone = pwd.clone();
    let log_dir = log_dir_base.clone();
    let app_handle = app.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut updated = 0;
        let mut inserted = 0;
        let mut errors = 0;
        let mut details = Vec::new();

        let log_msg = |msg: &str| {
            let date = chrono::Local::now().format("%Y%m%d").to_string();
            let time = chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]").to_string();
            let filename = format!("Ateis_OF_{}.log", date);
            let path = std::path::Path::new(&log_dir).join(filename);
            
            if let Some(parent) = path.parent() { let _ = std::fs::create_dir_all(parent); }

            if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
                 let _ = writeln!(file, "{}{}", time, msg); 
            }
        };

        let log_msg_inner = |msg: String| { log_msg(&msg); };
        
        // Log Header
        let date = chrono::Local::now().format("%Y%m%d").to_string();
        let path = std::path::Path::new(&log_dir).join(format!("Ateis_OF_{}.log", date));
        if !path.exists() {
             let header = format!(
                "{}\n   TRANSFERT ATEIS -> HFSQL (OF)\n   Date: {}\n{}\n\n",
                "=".repeat(50),
                chrono::Local::now().format("%d/%m/%Y %H:%M:%S"),
                "=".repeat(50)
            );
             if let Some(parent) = path.parent() { let _ = std::fs::create_dir_all(parent); }
             if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
                 let _ = file.write_all(header.as_bytes());
            }
        }
        log_msg("DEBUT DU TRANSFERT DES OF");
        log_msg("Mode: UPSERT (UPDATE si existe, INSERT sinon)");
        log_msg("Cle: Numero");
        log_msg(&format!("1. Recuperation des OF depuis SQL Server...\n[OK] {} OF recuperes", of_list.len()));
        log_msg(&format!("\n2. Traitement UPSERT de {} OF...", of_list.len()));

        let env = match Environment::new() {
            Ok(e) => e,
            Err(e) => {
                let err_msg = format!("Env init error: {}", e);
                log_msg_inner(format!("[ERROR] {}", err_msg));
                return ArticleSyncResult {
                    total_processed: total_of as i64,
                    updated: 0,
                    inserted: 0,
                    errors: total_of as i64,
                    error_details: vec![err_msg],
                };
            }
        };

        let conn_string = format!("DSN={};UID={};PWD={};", dsn_clone, user_clone, pwd_clone);
        let conn = match env.connect_with_connection_string(&conn_string, ConnectionOptions::default()) {
            Ok(c) => c,
            Err(e) => {
                 let err_msg = format!("Connection init error: {}", e);
                 log_msg_inner(format!("[ERROR] {}", err_msg));
                 return ArticleSyncResult {
                    total_processed: total_of as i64,
                    updated: 0,
                    inserted: 0,
                    errors: total_of as i64,
                    error_details: vec![err_msg],
                };
            }
        };

        let mut last_progress_emit = std::time::Instant::now();

        for (i, item) in of_list.iter().enumerate() {
            let numero = &item.0;
            if numero.trim().is_empty() {
                continue;
            }

            let check_query = format!("SELECT Numero FROM OrdreFabrication WHERE Numero = '{}'", numero);
            
            let exists = match conn.execute(&check_query, ()) {
                Ok(Some(mut cursor)) => cursor.next_row().ok().flatten().is_some(),
                Ok(None) => false,
                Err(e) => {
                    log_msg_inner(format!("Check query failed for {}: {}", numero, e));
                    false
                }
            };

            // Prepare values
            let v_numero = item.0.replace("'", "''");
            let v_ligne = item.1.replace("'", "''");
            let v_code_art = item.2.replace("'", "''");
            let v_qty = item.3;
            let v_desc = item.4.replace("'", "''");
            let v_date_debut = item.5;
            let v_date_fin = item.6;

            if exists {
                let sql = format!(
                    "UPDATE OrdreFabrication SET NumeroLigne='{}', CodeArt='{}', Quantite={}, \
                     Description='{}', DateDebut={}, DateFin={} WHERE Numero='{}'",
                    v_ligne, v_code_art, v_qty, v_desc, v_date_debut, v_date_fin, v_numero
                );

                match conn.execute(&sql, ()) {
                    Ok(_) => { updated += 1; },
                    Err(e) => {
                        errors += 1;
                        if details.len() < 10 { details.push(format!("{}: {}", numero, e)); }
                        log_msg_inner(format!("[ERROR] UPDATE {}: {}", numero, e));
                    }
                }
            } else {
                let sql = format!(
                    "INSERT INTO OrdreFabrication (Numero, NumeroLigne, CodeArt, Quantite, Description, DateDebut, DateFin) \
                     VALUES ('{}', '{}', '{}', {}, '{}', {}, {})",
                    v_numero, v_ligne, v_code_art, v_qty, v_desc, v_date_debut, v_date_fin
                );

                match conn.execute(&sql, ()) {
                    Ok(_) => { inserted += 1; },
                    Err(e) => {
                         errors += 1;
                         if details.len() < 10 { details.push(format!("{}: {}", numero, e)); }
                         log_msg_inner(format!("[ERROR] INSERT {}: {}", numero, e));
                    }
                }
            };
            
            if last_progress_emit.elapsed().as_millis() > 50 {
                let _ = app_handle.emit("ateis-of-sync-progress", SyncProgress {
                    current: i + 1,
                    total: total_of,
                    status: format!("Processing: {}", numero),
                });
                last_progress_emit = std::time::Instant::now();
            }

            if (i + 1) % 50 == 0 {
                log_msg_inner(format!("  Progression: {}/{}", i + 1, total_of));
            }
        }
        
        let _ = app_handle.emit("ateis-of-sync-progress", SyncProgress {
            current: total_of,
            total: total_of,
            status: "Completed".to_string(),
        });

        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let footer = format!(
            "\n{}\n[RAPPORT FINAL] TRANSFERT OF (UPSERT)\n{}\nTotal OF traites: {}\nOF mis a jour: {}\nNouveaux OF inseres: {}\nErreurs de traitement: {}\n\n{}\nSTATUT: {}\nFIN: {}\n{}\n",
            "=".repeat(70),
            "=".repeat(70),
            total_of,
            updated,
            inserted,
            errors,
            "=".repeat(70),
            if errors < total_of as i64 { "SUCCES" } else { "ECHEC" },
            timestamp,
            "=".repeat(70)
        );
        log_msg(&footer);

        let final_result = ArticleSyncResult {
            total_processed: total_of as i64,
            updated,
            inserted,
            errors,
            error_details: details,
        };

        let _ = app_handle.emit("ateis-of-sync-result", &final_result);

        final_result
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(result)
}

