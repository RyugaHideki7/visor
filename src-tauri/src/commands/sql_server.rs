use crate::db::DbState;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tauri::State;
use tiberius::{AuthMethod, Client, Config as SqlConfig};
use tokio_util::compat::TokioAsyncWriteCompatExt;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SqlServerConfig {
    pub id: i64,
    pub server: Option<String>,
    pub database: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub error: Option<String>,
}

pub(crate) async fn connect_sql_server(
    cfg: SqlServerConfig,
) -> Result<Client<tokio_util::compat::Compat<tokio::net::TcpStream>>, String> {
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
pub async fn get_sql_server_config(state: State<'_, DbState>) -> Result<SqlServerConfig, String> {
    let config = sqlx::query_as::<_, SqlServerConfig>(
        "SELECT id, server, database, username, password, enabled FROM sql_server_config WHERE id = 1",
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
        "UPDATE sql_server_config SET server = ?, database = ?, username = ?, password = ?, enabled = ? WHERE id = 1",
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

#[tauri::command]
pub async fn test_sql_server_connection(
    state: State<'_, DbState>,
) -> Result<ConnectionTestResult, String> {
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
            return Ok(ConnectionTestResult {
                success: false,
                error: Some(e.to_string()),
            });
        }
        Err(_) => {
            return Ok(ConnectionTestResult {
                success: false,
                error: Some("Timeout de connexion (10s)".to_string()),
            });
        }
    };

    if let Err(e) = tcp.set_nodelay(true) {
        return Ok(ConnectionTestResult {
            success: false,
            error: Some(e.to_string()),
        });
    }

    let client_res = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        Client::connect(tiberius_config, tcp.compat_write()),
    )
    .await;

    match client_res {
        Ok(Ok(mut client)) => {
            let query_res = client.query("SELECT 1", &[]).await;
            match query_res {
                Ok(_) => Ok(ConnectionTestResult {
                    success: true,
                    error: None,
                }),
                Err(e) => Ok(ConnectionTestResult {
                    success: false,
                    error: Some(e.to_string()),
                }),
            }
        }
        Ok(Err(e)) => Ok(ConnectionTestResult {
            success: false,
            error: Some(e.to_string()),
        }),
        Err(_) => Ok(ConnectionTestResult {
            success: false,
            error: Some("Timeout de connexion (10s)".to_string()),
        }),
    }
}
