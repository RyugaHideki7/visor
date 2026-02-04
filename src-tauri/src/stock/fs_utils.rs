use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn is_file_locked(path: &Path) -> bool {
    match fs::OpenOptions::new().read(true).write(true).open(path) {
        Ok(_) => false,
        Err(_) => true,
    }
}

pub(crate) fn scan_existing_files(path: &Path, prefix: &str) -> Vec<PathBuf> {
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
            let allowed_ext = upper.ends_with(".TMP") || upper.ends_with(".CSV") || upper.ends_with(".TXT");
            if allowed_ext && upper.contains(&prefix.to_uppercase()) {
                matches.push(p);
            }
        }
    }

    matches
}
