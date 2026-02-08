use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::fs;
use tauri::AppHandle;
use tauri::Manager;

async fn seed_model(
    pool: &Pool<Sqlite>,
    format_name: &str,
    rows: Vec<(
        i64,
        &str,
        Option<&str>,
        Option<&str>,
        Option<&str>,
        Option<&str>,
    )>,
) -> Result<(), sqlx::Error> {
    let existing: i64 =
        sqlx::query_scalar("SELECT COUNT(1) FROM model_mappings WHERE format_name = ?")
            .bind(format_name)
            .fetch_one(pool)
            .await
            .unwrap_or(0);

    if existing > 0 {
        return Ok(());
    }

    for (sort_order, sql_field, file_column, parameter, transformation, description) in rows {
        let _ = sqlx::query(
            "INSERT OR IGNORE INTO model_mappings (format_name, sort_order, sql_field, file_column, parameter, transformation, description) \
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(format_name)
        .bind(sort_order)
        .bind(sql_field)
        .bind(file_column)
        .bind(parameter)
        .bind(transformation)
        .bind(description)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub struct DbState {
    pub pool: Pool<Sqlite>,
}

pub async fn init_db(app_handle: &AppHandle) -> Result<Pool<Sqlite>, Box<dyn std::error::Error>> {
    let app_dir = app_handle.path().app_data_dir()?;

    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }

    let db_path = app_dir.join("visor.db");
    let db_url = format!("sqlite://{}", db_path.to_str().unwrap());

    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        Sqlite::create_database(&db_url).await?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS lines (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            prefix TEXT NOT NULL,
            interval_check INTEGER DEFAULT 60,
            interval_alert INTEGER DEFAULT 120,
            archived_path TEXT,
            rejected_path TEXT,
            active BOOLEAN DEFAULT 1,
            site TEXT,
            unite TEXT,
            flag_dec TEXT,
            code_ligne TEXT,
            log_path TEXT,
            file_format TEXT DEFAULT 'ATEIS',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await?;

    // Migration: add new columns if they don't exist (for existing DBs)
    let _ = sqlx::query("ALTER TABLE lines ADD COLUMN site TEXT")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE lines ADD COLUMN unite TEXT")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE lines ADD COLUMN flag_dec TEXT")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE lines ADD COLUMN code_ligne TEXT")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE lines ADD COLUMN log_path TEXT")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE lines ADD COLUMN file_format TEXT DEFAULT 'ATEIS'")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE lines ADD COLUMN total_traites INTEGER DEFAULT 0")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE lines ADD COLUMN total_erreurs INTEGER DEFAULT 0")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE lines ADD COLUMN last_file_time TEXT")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE lines ADD COLUMN etat_actuel TEXT DEFAULT 'ARRET'")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE lines ADD COLUMN rejected_path TEXT")
        .execute(&pool)
        .await;

    // Migration: Add log_path to hfsql_config
    let _ = sqlx::query("ALTER TABLE hfsql_config ADD COLUMN log_path TEXT")
        .execute(&pool)
        .await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS mappings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            line_id INTEGER NOT NULL,
            sort_order INTEGER DEFAULT 0,
            sql_field TEXT NOT NULL,
            file_column TEXT,
            parameter TEXT,
            transformation TEXT,
            description TEXT,
            FOREIGN KEY(line_id) REFERENCES lines(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS model_mappings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            format_name TEXT NOT NULL,
            sort_order INTEGER DEFAULT 0,
            sql_field TEXT NOT NULL,
            file_column TEXT,
            parameter TEXT,
            transformation TEXT,
            description TEXT,
            UNIQUE(format_name, sort_order)
        )",
    )
    .execute(&pool)
    .await?;

    seed_model(
        &pool,
        "ATEIS",
        vec![
            (0, "YSSCC_0", Some("0"), None, None, Some("Code SCC")),
            (
                1,
                "YDATE_0",
                Some("1"),
                None,
                Some("date"),
                Some("Date déclaration"),
            ),
            (
                2,
                "YHEURE_0",
                Some("1"),
                None,
                Some("heure"),
                Some("Heure déclaration"),
            ),
            (
                3,
                "ITMREF_0",
                Some("5"),
                None,
                None,
                Some("Référence article"),
            ),
            (4, "LOT_0", Some("7"), None, None, Some("Numéro de lot")),
            (
                5,
                "QTY_0",
                Some("9"),
                None,
                Some("decimal"),
                Some("Quantité"),
            ),
            (
                6,
                "YDATDL_0",
                Some("8"),
                None,
                Some("date"),
                Some("Date livraison"),
            ),
            (
                7,
                "YNLIGN_0",
                Some("12"),
                None,
                None,
                Some("Numéro de ligne"),
            ),
            (
                8,
                "MFGNUM_0",
                Some("18"),
                None,
                None,
                Some("Numéro de fabrication"),
            ),
            (9, "YCODEPOT_0", Some("4"), None, None, Some("Code dépôt")),
            (
                10,
                "YPALETTE_0",
                Some("16"),
                None,
                Some("split_before_plus"),
                Some("Partie avant +"),
            ),
            (
                11,
                "YINTERCAL_0",
                Some("17"),
                None,
                Some("split_after_plus"),
                Some("Partie après +"),
            ),
            (
                12,
                "FCY_0",
                None,
                Some("site"),
                None,
                Some("Site de production"),
            ),
            (
                13,
                "UOM_0",
                None,
                Some("unite"),
                None,
                Some("Unité de mesure"),
            ),
            (
                14,
                "YFLGDEC_0",
                None,
                Some("flag_dec"),
                Some("tinyint"),
                Some("Flag déclaration"),
            ),
            (
                15,
                "CREUSR_0",
                None,
                Some("code_ligne"),
                None,
                Some("Utilisateur création"),
            ),
            (
                16,
                "CREDATTIM_0",
                Some("1"),
                None,
                Some("datetime"),
                Some("Date/heure création"),
            ),
        ],
    )
    .await?;

    seed_model(
        &pool,
        "LOGITRON",
        vec![
            (0, "YSSCC_0", Some("0"), None, None, Some("Code SCC")),
            (
                1,
                "YDATE_0",
                Some("1"),
                None,
                Some("date"),
                Some("Date déclaration"),
            ),
            (
                2,
                "YHEURE_0",
                Some("2"),
                None,
                Some("heure"),
                Some("Heure déclaration"),
            ),
            (
                3,
                "CREDATTIM_0",
                Some("1-2"),
                None,
                Some("datetime_combine"),
                Some("Date/heure création combinée"),
            ),
            (
                4,
                "ITMREF_0",
                Some("3"),
                None,
                None,
                Some("Référence article"),
            ),
            (5, "LOT_0", Some("4"), None, None, Some("Numéro de lot")),
            (
                6,
                "QTY_0",
                Some("5"),
                None,
                Some("decimal"),
                Some("Quantité"),
            ),
            (
                7,
                "YDATDL_0",
                Some("7"),
                None,
                Some("date"),
                Some("Date livraison"),
            ),
            (
                8,
                "YNLIGN_0",
                Some("8"),
                None,
                None,
                Some("Numéro de ligne"),
            ),
            (
                9,
                "MFGNUM_0",
                Some("13"),
                None,
                None,
                Some("Numéro de fabrication"),
            ),
            (10, "YCODEPOT_0", Some("14"), None, None, Some("Code dépôt")),
            (
                11,
                "YPALETTE_0",
                Some("15"),
                None,
                Some("split_before_plus"),
                Some("Partie avant +"),
            ),
            (
                12,
                "YINTERCAL_0",
                Some("15"),
                None,
                Some("split_after_plus"),
                Some("Partie après +"),
            ),
            (
                13,
                "FCY_0",
                None,
                Some("site"),
                None,
                Some("Site de production"),
            ),
            (
                14,
                "UOM_0",
                None,
                Some("unite"),
                None,
                Some("Unité de mesure"),
            ),
            (
                15,
                "YFLGDEC_0",
                None,
                Some("flag_dec"),
                Some("tinyint"),
                Some("Flag déclaration"),
            ),
            (
                16,
                "CREUSR_0",
                None,
                Some("code_ligne"),
                None,
                Some("Utilisateur création"),
            ),
        ],
    )
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS production_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            line_id INTEGER,
            filename TEXT,
            processed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            status TEXT NOT NULL,
            message TEXT,
            FOREIGN KEY(line_id) REFERENCES lines(id)
        )",
    )
    .execute(&pool)
    .await?;

    // Create a generic key-value store for app configuration
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS config (
            key TEXT PRIMARY KEY,
            value TEXT
        )",
    )
    .execute(&pool)
    .await?;

    // Logs table for Journaux page
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            line_id INTEGER,
            level TEXT NOT NULL,
            source TEXT,
            message TEXT NOT NULL,
            details TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(line_id) REFERENCES lines(id) ON DELETE SET NULL
        )",
    )
    .execute(&pool)
    .await?;

    // SQL Server connection settings table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sql_server_config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            server TEXT,
            database TEXT,
            username TEXT,
            password TEXT,
            enabled BOOLEAN DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;

    // Insert default SQL Server config row if not exists
    sqlx::query(
        "INSERT OR IGNORE INTO sql_server_config (id, server, database, username, password, enabled) 
         VALUES (1, '', '', '', '', 0)"
    )
    .execute(&pool)
    .await?;

    // SQL query templates table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sql_queries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            format_name TEXT UNIQUE NOT NULL,
            format_name TEXT UNIQUE NOT NULL,
            query_template TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    // HFSQL connection settings table (ODBC)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS hfsql_config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            dsn TEXT,
            username TEXT,
            password TEXT,
            log_path TEXT
        )",
    )
    .execute(&pool)
    .await?;

    // Insert default HFSQL config row if not exists
    sqlx::query(
        "INSERT OR IGNORE INTO hfsql_config (id, dsn, username, password) 
         VALUES (1, 'HFSQL', 'Admin', '1234')",
    )
    .execute(&pool)
    .await?;

    // Insert default SQL queries for ATEIS and LOGITRON formats
    // Use centralized defaults from commands module
    let default_ateis_query = crate::commands::sql_queries::DEFAULT_ATEIS_QUERY;

    let default_logitron_query = crate::commands::sql_queries::DEFAULT_LOGITRON_QUERY;

    let _ = sqlx::query(
        "DELETE FROM sql_queries 
         WHERE id NOT IN (
            SELECT MIN(id) 
            FROM sql_queries 
            GROUP BY format_name
         )",
    )
    .execute(&pool)
    .await;

    let _ = sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_sql_queries_format_name ON sql_queries(format_name)",
    )
    .execute(&pool)
    .await;

    // 3. Now safe to insert defaults using INSERT OR IGNORE
    sqlx::query(
        "INSERT OR IGNORE INTO sql_queries (format_name, query_template) VALUES ('ATEIS', ?)",
    )
    .bind(default_ateis_query)
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT OR IGNORE INTO sql_queries (format_name, query_template) VALUES ('LOGITRON', ?)",
    )
    .bind(default_logitron_query)
    .execute(&pool)
    .await?;

    // Version check: Disable SQL Server if version changed
    let current_version = app_handle.package_info().version.to_string();
    let stored_version: Option<String> =
        sqlx::query_scalar("SELECT value FROM config WHERE key = 'last_version'")
            .fetch_optional(&pool)
            .await
            .unwrap_or(None);

    if stored_version.as_deref() != Some(&current_version) {
        // Version mismatch (update or first run) -> Disable SQL Server
        let _ = sqlx::query("UPDATE sql_server_config SET enabled = 0 WHERE id = 1")
            .execute(&pool)
            .await;

        // Update stored version
        let _ =
            sqlx::query("INSERT OR REPLACE INTO config (key, value) VALUES ('last_version', ?)")
                .bind(&current_version)
                .execute(&pool)
                .await;
    }

    Ok(pool)
}
