use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::fs;
use tauri::AppHandle;
use tauri::Manager;

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
    let _ = sqlx::query("ALTER TABLE lines ADD COLUMN site TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE lines ADD COLUMN unite TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE lines ADD COLUMN flag_dec TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE lines ADD COLUMN code_ligne TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE lines ADD COLUMN log_path TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE lines ADD COLUMN file_format TEXT DEFAULT 'ATEIS'").execute(&pool).await;

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

    Ok(pool)
}
