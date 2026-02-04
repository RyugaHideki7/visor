use crate::commands::mappings::{get_ateis_default_mappings, get_logitron_default_mappings, MappingRow};

#[tauri::command]
pub async fn get_default_mappings(format_name: String) -> Result<Vec<MappingRow>, String> {
    // This command is kept for backwards compatibility.
    // Default mappings are initialized lazily by model mappings API.
    // Here, we return the same defaults used when model mappings are missing.
    let mappings = match format_name.to_uppercase().as_str() {
        "ATEIS" => get_ateis_default_mappings(),
        "LOGITRON" => get_logitron_default_mappings(),
        _ => get_ateis_default_mappings(),
    };
    Ok(mappings)
}
