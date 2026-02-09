use crate::commands::sql_queries::{
    get_or_init_sql_query, DEFAULT_LOGITRON_PRODUIT_QUERY, DEFAULT_ORDRE_FABRICATION_QUERY,
};
use crate::commands::sql_server::{connect_sql_server, get_sql_server_config};
use crate::db::DbState;
use futures_util::TryStreamExt;
use serde::Serialize;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;
use tauri::{Emitter, State};
use tiberius::numeric::Decimal;
use tiberius::QueryItem;

#[derive(Debug, Serialize)]
pub struct ExportDatResult {
    pub output_path: String,
    pub rows: i64,
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

#[tauri::command]
pub async fn export_logitron_produit_dat(
    app: tauri::AppHandle,
    state: State<'_, DbState>,
    output_path: String,
    is_auto: Option<bool>,
) -> Result<ExportDatResult, String> {
    let result = async {
        if output_path.trim().is_empty() {
            return Err("Chemin de sortie manquant".to_string());
        }

        let cfg = get_sql_server_config(state.clone()).await?;
        let mut client = connect_sql_server(cfg).await?;

        let query = get_or_init_sql_query(
            &state.pool,
            "LOGITRON_PRODUIT",
            DEFAULT_LOGITRON_PRODUIT_QUERY,
        )
        .await?;

        let mut stream = client
            .query(query.as_str(), &[])
            .await
            .map_err(|e| e.to_string())?;

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
            let poids_casier: Option<String> = row
                .try_get::<Decimal, _>("POIDS_CASIER")
                .ok()
                .flatten()
                .map(|v| v.to_string())
                .or_else(|| {
                    row.try_get::<f64, _>("POIDS_CASIER")
                        .ok()
                        .flatten()
                        .map(|v| v.to_string())
                })
                .or_else(|| {
                    row.try_get::<i64, _>("POIDS_CASIER")
                        .ok()
                        .flatten()
                        .map(|v| v.to_string())
                })
                .or_else(|| {
                    row.try_get::<i32, _>("POIDS_CASIER")
                        .ok()
                        .flatten()
                        .map(|v| v.to_string())
                })
                .or_else(|| {
                    row.try_get::<&str, _>("POIDS_CASIER")
                        .ok()
                        .flatten()
                        .map(|s| s.to_string())
                });
            let ean_carton: Option<&str> = row.get("EAN_CARTON");
            let nb_bouteille_casier: Option<i64> = row
                .try_get::<i64, _>("NB_BOUTEILLE_PAR_CASIER")
                .ok()
                .flatten()
                .or_else(|| {
                    row.try_get::<i32, _>("NB_BOUTEILLE_PAR_CASIER")
                        .ok()
                        .flatten()
                        .map(|v| v as i64)
                });
            let nb_bouteille_palette: Option<i64> = row
                .try_get::<i64, _>("NB_BOUTEILLE_PAR_PALETTE")
                .ok()
                .flatten()
                .or_else(|| {
                    row.try_get::<i32, _>("NB_BOUTEILLE_PAR_PALETTE")
                        .ok()
                        .flatten()
                        .map(|v| v as i64)
                });
            let nb_casier_palette: Option<i64> = row
                .try_get::<i64, _>("NB_CASIER_PAR_PALETTE")
                .ok()
                .flatten()
                .or_else(|| {
                    row.try_get::<i32, _>("NB_CASIER_PAR_PALETTE")
                        .ok()
                        .flatten()
                        .map(|v| v as i64)
                });
            let methode_dluo: Option<&str> = row.get("METHODE_CALCUL_DLUO");
            let ean_palette: Option<&str> = row.get("EAN_PALETTE");

            let line = format!(
                "{}{}{}{}{}{}{}{}{}\r\n",
                format_left(code_produit.map(|s| s.to_string()), 14),
                format_left(libelle.map(|s| s.to_string()), 30),
                format_left(poids_casier, 22),
                format_left(ean_carton.map(|s| s.to_string()), 14),
                format_left_any(nb_bouteille_casier, 22),
                format_left_any(nb_bouteille_palette, 22),
                format_left_any(nb_casier_palette, 22),
                format_left(methode_dluo.map(|s| s.to_string()), 8),
                format_left(ean_palette.map(|s| s.to_string()), 14),
            );

            writer
                .write_all(line.as_bytes())
                .map_err(|e| e.to_string())?;
            row_count += 1;
        }

        writer.flush().map_err(|e| e.to_string())?;
        drop(writer);

        if out_path.exists() {
            fs::remove_file(out_path).map_err(|e| e.to_string())?;
        }
        fs::rename(&tmp_path, out_path).map_err(|e| e.to_string())?;

        Ok(ExportDatResult {
            output_path,
            rows: row_count,
        })
    }
    .await;

    match &result {
        Ok(res) => {
            app.emit("logitron-produit-export-result", res)
                .unwrap_or_else(|e| eprintln!("Failed to emit result: {}", e));
        }
        Err(e) => {
            if is_auto != Some(true) {
                app.emit("logitron-produit-export-error", e)
                    .unwrap_or_else(|e| eprintln!("Failed to emit error: {}", e));
            } else {
                eprintln!("Suppressed auto-export error: {}", e);
            }
        }
    }

    result
}

fn format_date_yyyymmdd(date: Option<chrono::NaiveDate>) -> String {
    date.map(|d| d.format("%Y%m%d").to_string())
        .unwrap_or_default()
}

#[tauri::command]
pub async fn export_ordre_fabrication_dat(
    app: tauri::AppHandle,
    state: State<'_, DbState>,
    output_path: String,
) -> Result<ExportDatResult, String> {
    use chrono::NaiveDateTime;

    let result = async {
        if output_path.trim().is_empty() {
            return Err("Chemin de sortie manquant".to_string());
        }

        let cfg = get_sql_server_config(state.clone()).await?;
        let mut client = connect_sql_server(cfg).await?;

        let query = get_or_init_sql_query(
            &state.pool,
            "LOGITRON_ORDRE_FABRICATION",
            DEFAULT_ORDRE_FABRICATION_QUERY,
        )
        .await?;

        let mut stream = client
            .query(query.as_str(), &[])
            .await
            .map_err(|e| e.to_string())?;

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

            // Column 0: MFGNUM_0
            let mfgnum: String = row
                .try_get::<&str, _>(0)
                .ok()
                .flatten()
                .map(|s| s.to_string())
                .unwrap_or_default();

            // Column 1: ITMREF_0
            let itmref: String = row
                .try_get::<&str, _>(1)
                .ok()
                .flatten()
                .map(|s| s.to_string())
                .unwrap_or_default();

            // Column 2: EXTQTY_0
            let extqty: Option<String> = row
                .try_get::<Decimal, _>(2)
                .ok()
                .flatten()
                .map(|v| v.to_string())
                .or_else(|| {
                    row.try_get::<f64, _>(2)
                        .ok()
                        .flatten()
                        .map(|v| v.to_string())
                })
                .or_else(|| {
                    row.try_get::<i64, _>(2)
                        .ok()
                        .flatten()
                        .map(|v| v.to_string())
                })
                .or_else(|| {
                    row.try_get::<i32, _>(2)
                        .ok()
                        .flatten()
                        .map(|v| v.to_string())
                });
            let extqty_str = extqty.unwrap_or_else(|| "0".to_string());

            // Column 3: STRDAT_0 (date)
            let strdat: Option<chrono::NaiveDate> = row
                .try_get::<NaiveDateTime, _>(3)
                .ok()
                .flatten()
                .map(|dt| dt.date())
                .or_else(|| row.try_get::<chrono::NaiveDate, _>(3).ok().flatten());
            let strdat_str = format_date_yyyymmdd(strdat);

            // Column 4: empty string (as per original query)
            let empty = "";

            // Column 5: YLIGNEOF_0
            let yligneof: String = row
                .try_get::<&str, _>(5)
                .ok()
                .flatten()
                .map(|s| s.to_string())
                .or_else(|| {
                    row.try_get::<i64, _>(5)
                        .ok()
                        .flatten()
                        .map(|v| v.to_string())
                })
                .or_else(|| {
                    row.try_get::<i32, _>(5)
                        .ok()
                        .flatten()
                        .map(|v| v.to_string())
                })
                .unwrap_or_default();

            // Semicolon-separated format with \r\n line ending
            let line = format!(
                "{};{};{};{};{};{}\r\n",
                mfgnum, itmref, extqty_str, strdat_str, empty, yligneof
            );

            writer
                .write_all(line.as_bytes())
                .map_err(|e| e.to_string())?;
            row_count += 1;
        }

        writer.flush().map_err(|e| e.to_string())?;
        drop(writer);

        if out_path.exists() {
            fs::remove_file(out_path).map_err(|e| e.to_string())?;
        }
        fs::rename(&tmp_path, out_path).map_err(|e| e.to_string())?;

        Ok(ExportDatResult {
            output_path,
            rows: row_count,
        })
    }
    .await;

    match &result {
        Ok(res) => {
            app.emit("logitron-of-export-result", res)
                .unwrap_or_else(|e| eprintln!("Failed to emit result: {}", e));
        }
        Err(e) => {
            app.emit("logitron-of-export-error", e)
                .unwrap_or_else(|e| eprintln!("Failed to emit error: {}", e));
        }
    }

    result
}

#[tauri::command]
pub async fn export_ateis_produit_dat(
    state: State<'_, DbState>,
    output_path: String,
) -> Result<ExportDatResult, String> {
    if output_path.trim().is_empty() {
        return Err("Chemin de sortie manquant".to_string());
    }

    let cfg = get_sql_server_config(state.clone()).await?;
    let mut client = connect_sql_server(cfg).await?;

    let query = get_or_init_sql_query(
        &state.pool,
        "ATEIS_PRODUIT",
        crate::commands::sql_queries::DEFAULT_ATEIS_PRODUIT_QUERY,
    )
    .await?;

    let mut stream = client
        .query(query.as_str(), &[])
        .await
        .map_err(|e| e.to_string())?;

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

        // Extraction helper
        let get_str = |idx: usize| -> String {
            row.try_get::<&str, _>(idx)
                .ok()
                .flatten()
                .map(|s| s.to_string())
                .unwrap_or_default()
        };
        let get_dec = |idx: usize| -> String {
            row.try_get::<Decimal, _>(idx)
                .ok()
                .flatten()
                .map(|v| v.to_string())
                .or_else(|| {
                    row.try_get::<f64, _>(idx)
                        .ok()
                        .flatten()
                        .map(|v| v.to_string())
                })
                .or_else(|| {
                    row.try_get::<i64, _>(idx)
                        .ok()
                        .flatten()
                        .map(|v| v.to_string())
                })
                .unwrap_or_else(|| "0".to_string())
        };
        let get_int = |idx: usize| -> String {
            row.try_get::<i64, _>(idx)
                .ok()
                .flatten()
                .map(|v| v.to_string())
                .or_else(|| {
                    row.try_get::<i32, _>(idx)
                        .ok()
                        .flatten()
                        .map(|v| v.to_string())
                })
                .unwrap_or_else(|| "0".to_string())
        };

        let ean13 = get_str(0);
        let designation = get_str(1);
        let code_art = get_str(2);
        let poids = get_dec(3);
        let nb_fard_pal = get_int(4);
        let decal_dluo = get_str(5);
        let ean_pal = get_str(6);
        let des_arabe = get_str(7);
        let ean_export = get_str(8);

        // ATEIS uses semicolon separated format with UTF-8
        let line = format!(
            "{};{};{};{};{};{};{};{};{}\r\n",
            ean13,
            designation,
            code_art,
            poids,
            nb_fard_pal,
            decal_dluo,
            ean_pal,
            des_arabe,
            ean_export
        );

        writer
            .write_all(line.as_bytes())
            .map_err(|e| e.to_string())?;
        row_count += 1;
    }

    writer.flush().map_err(|e| e.to_string())?;
    drop(writer);

    if out_path.exists() {
        fs::remove_file(out_path).map_err(|e| e.to_string())?;
    }
    fs::rename(&tmp_path, out_path).map_err(|e| e.to_string())?;

    Ok(ExportDatResult {
        output_path,
        rows: row_count,
    })
}

#[tauri::command]
pub async fn export_ateis_of_dat(
    state: State<'_, DbState>,
    output_path: String,
) -> Result<ExportDatResult, String> {
    use chrono::NaiveDateTime;
    if output_path.trim().is_empty() {
        return Err("Chemin de sortie manquant".to_string());
    }

    let cfg = get_sql_server_config(state.clone()).await?;
    let mut client = connect_sql_server(cfg).await?;

    let query = get_or_init_sql_query(
        &state.pool,
        "ATEIS_OF",
        crate::commands::sql_queries::DEFAULT_ATEIS_OF_QUERY,
    )
    .await?;

    let mut stream = client
        .query(query.as_str(), &[])
        .await
        .map_err(|e| e.to_string())?;

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

        // Helper to format date as YYYYMMDD
        let get_date_fmt = |idx: usize| -> String {
            let dt_opt = row
                .try_get::<NaiveDateTime, _>(idx)
                .ok()
                .flatten()
                .map(|dt| dt.date())
                .or_else(|| row.try_get::<chrono::NaiveDate, _>(idx).ok().flatten());
            format_date_yyyymmdd(dt_opt)
        };

        let mfgnum: String = row
            .try_get::<&str, _>(0)
            .ok()
            .flatten()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let yligneof: String = row
            .try_get::<i32, _>(1)
            .ok()
            .flatten()
            .map(|v| v.to_string())
            .unwrap_or_default();
        let itmref: String = row
            .try_get::<&str, _>(2)
            .ok()
            .flatten()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let extqty: String = row
            .try_get::<Decimal, _>(3)
            .ok()
            .flatten()
            .map(|v| v.to_string())
            .or_else(|| {
                row.try_get::<f64, _>(3)
                    .ok()
                    .flatten()
                    .map(|v| v.to_string())
            })
            .unwrap_or_else(|| "0".to_string());
        let desc = ""; // Blank column 4
        let strdat = get_date_fmt(5);
        let enddat = get_date_fmt(6);

        // Output line: Numero;NumeroLigne;CodeArt;Quantite;Description;DateFin;DateDebut
        let line = format!(
            "{};{};{};{};{};{};{}\r\n",
            mfgnum, yligneof, itmref, extqty, desc, enddat, strdat
        );

        writer
            .write_all(line.as_bytes())
            .map_err(|e| e.to_string())?;
        row_count += 1;
    }

    writer.flush().map_err(|e| e.to_string())?;
    drop(writer);

    if out_path.exists() {
        fs::remove_file(out_path).map_err(|e| e.to_string())?;
    }
    fs::rename(&tmp_path, out_path).map_err(|e| e.to_string())?;

    Ok(ExportDatResult {
        output_path,
        rows: row_count,
    })
}
