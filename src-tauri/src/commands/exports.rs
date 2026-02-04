use crate::commands::sql_queries::{get_or_init_sql_query, DEFAULT_LOGITRON_PRODUIT_QUERY};
use crate::commands::sql_server::{connect_sql_server, get_sql_server_config};
use crate::db::DbState;
use futures_util::TryStreamExt;
use serde::Serialize;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;
use tauri::State;
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
    state: State<'_, DbState>,
    output_path: String,
) -> Result<ExportDatResult, String> {
    if output_path.trim().is_empty() {
        return Err("Chemin de sortie manquant".to_string());
    }

    let cfg = get_sql_server_config(state.clone()).await?;
    let mut client = connect_sql_server(cfg).await?;

    let query =
        get_or_init_sql_query(&state.pool, "LOGITRON_PRODUIT", DEFAULT_LOGITRON_PRODUIT_QUERY).await?;

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

    Ok(ExportDatResult {
        output_path,
        rows: row_count,
    })
}
