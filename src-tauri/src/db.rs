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
                Some("2"),
                None,
                Some("heure"),
                Some("Heure déclaration"),
            ),
            (
                3,
                "ITMREF_0",
                Some("3"),
                None,
                None,
                Some("Référence article"),
            ),
            (4, "LOT_0", Some("4"), None, None, Some("Numéro de lot")),
            (
                5,
                "QTY_0",
                Some("5"),
                None,
                Some("decimal"),
                Some("Quantité"),
            ),
            (
                6,
                "YDATDL_0",
                Some("7"),
                None,
                Some("date"),
                Some("Date livraison"),
            ),
            (
                7,
                "YNLIGN_0",
                Some("8"),
                None,
                None,
                Some("Numéro de ligne"),
            ),
            (
                8,
                "MFGNUM_0",
                Some("13"),
                None,
                None,
                Some("Numéro de fabrication"),
            ),
            (9, "YCODEPOT_0", Some("14"), None, None, Some("Code dépôt")),
            (
                10,
                "YPALETTE_0",
                Some("15"),
                None,
                Some("split_before_plus"),
                Some("Partie avant +"),
            ),
            (
                11,
                "YINTERCAL_0",
                Some("15"),
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
                Some("1-2"),
                None,
                Some("datetime_combine"),
                Some("Date/heure création (YDATE + YHEURE)"),
            ),
        ],
    )
    .await?;

    // Backfill existing ATEIS rows to use date+time (YDATE/YHEURE) instead of current datetime
    sqlx::query(
        "UPDATE model_mappings
         SET file_column = '1-2', transformation = 'datetime_combine'
         WHERE format_name = 'ATEIS' AND sql_field = 'CREDATTIM_0'",
    )
    .execute(&pool)
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
            query_template TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    // Insert default SQL queries for ATEIS and LOGITRON formats
    let default_ateis_query = r#"INSERT INTO ITHRITEST.YINTDECL (
    YSSCC_0, YDATE_0, YHEURE_0, ITMREF_0, LOT_0, 
    QTY_0, YDATDL_0, YNLIGN_0, MFGNUM_0, YCODEPOT_0, 
    YPALETTE_0, YINTERCAL_0, FCY_0, UOM_0, YFLGDEC_0, 
    CREUSR_0, CREDATTIM_0
) VALUES (
    @P1, @P2, @P3, @P4, @P5, 
    @P6, @P7, @P8, @P9, @P10, 
    @P11, @P12, @P13, @P14, @P15, 
    @P16, @P17
)"#;

    let default_logitron_query = r#"INSERT INTO ITHRITEST.YINTDECL (
    MFGNUM_0, FCY_0, ITMREF_0, QTY_0, UOM_0, 
    YSSCC_0, YFLGDEC_0, LOT_0, CREDATTIM_0, 
    CREUSR_0, YDATE_0, YHEURE_0, YNLIGN_0, 
    YDATDL_0, YCODEPOT_0, YPALETTE_0, YINTERCAL_0
) VALUES (
    @P1, @P2, @P3, @P4, @P5, 
    @P6, @P7, @P8, @P9, 
    @P10, @P11, @P12, @P13, 
    @P14, @P15, @P16, @P17
)"#;

    sqlx::query(
        "INSERT OR IGNORE INTO sql_queries (format_name, query_template) VALUES ('ATEIS', ?)",
    )
    .bind(default_ateis_query)
    .execute(&pool)
    .await?;

    // Always overwrite to ensure template stays in sync with code
    sqlx::query("UPDATE sql_queries SET query_template = ? WHERE format_name = 'ATEIS'")
        .bind(default_ateis_query)
        .execute(&pool)
        .await?;

    sqlx::query(
        "INSERT OR IGNORE INTO sql_queries (format_name, query_template) VALUES ('LOGITRON', ?)",
    )
    .bind(default_logitron_query)
    .execute(&pool)
    .await?;

    sqlx::query("UPDATE sql_queries SET query_template = ? WHERE format_name = 'LOGITRON'")
        .bind(default_logitron_query)
        .execute(&pool)
        .await?;

    Ok(pool)
}
