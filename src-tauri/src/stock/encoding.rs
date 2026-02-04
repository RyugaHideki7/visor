use encoding_rs::{UTF_8, WINDOWS_1252};
use std::fs;
use std::path::Path;

/// Read file with multiple encoding attempts (like Python's encoding fallback)
pub(crate) fn read_file_with_encoding_fallback(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;

    if let Ok(s) = std::str::from_utf8(&bytes) {
        return Ok(s.to_string());
    }

    let (cow, _, had_errors) = UTF_8.decode(&bytes);
    if !had_errors {
        return Ok(cow.into_owned());
    }

    let (cow, _, _) = WINDOWS_1252.decode(&bytes);
    Ok(cow.into_owned())
}
