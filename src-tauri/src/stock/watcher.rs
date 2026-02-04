use crate::stock::fs_utils::scan_existing_files;
use crate::stock::processor::StockProcessor;
use crate::stock::registry::{WatcherHandle, WatcherState};
use notify::{Config, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use tauri::{AppHandle, Manager};

pub fn start_watcher(
    app_handle: AppHandle,
    line_id: i64,
    path: String,
    prefix: String,
    archived_path: Option<String>,
) {
    let state = app_handle.state::<WatcherState>();

    {
        let watchers = state.watchers.lock().expect("watchers mutex poisoned");
        if watchers.contains_key(&line_id) {
            return;
        }
    }

    let pool = app_handle.state::<crate::db::DbState>().pool.clone();
    let processor = StockProcessor::new(pool);

    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let processed_files = Arc::new(Mutex::new(HashMap::<String, SystemTime>::new()));

    {
        let mut watchers = state.watchers.lock().expect("watchers mutex poisoned");
        watchers.insert(line_id, WatcherHandle { stop_tx });
    }

    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher =
            notify::RecommendedWatcher::new(tx, Config::default()).expect("failed to create watcher");

        let watch_path = Path::new(&path);
        if !watch_path.exists() {
            eprintln!("Watch path does not exist: {}", path);
            return;
        }

        for p in scan_existing_files(watch_path, &prefix) {
            let file_key = p.to_string_lossy().to_string();
            let mut processed = processed_files.lock().expect("processed_files mutex poisoned");
            if processed.contains_key(&file_key) {
                continue;
            }
            processed.insert(file_key, SystemTime::now());
            drop(processed);

            let pr = processor.pool_clone();
            let proc = StockProcessor::new(pr);
            let pref = prefix.clone();
            let arch = archived_path.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = proc.process_file(line_id, p, pref, arch).await {
                    eprintln!("Error processing existing file: {}", e);
                }
            });
        }

        watcher
            .watch(watch_path, RecursiveMode::NonRecursive)
            .expect("failed to watch path");

        let mut last_scan = Instant::now();

        loop {
            if stop_rx.try_recv().is_ok() {
                break;
            }

            if last_scan.elapsed() >= Duration::from_secs(5) {
                {
                    let mut processed =
                        processed_files.lock().expect("processed_files mutex poisoned");
                    let now = SystemTime::now();
                    processed.retain(|_, time| {
                        now.duration_since(*time)
                            .map(|d| d.as_secs() < 60)
                            .unwrap_or(false)
                    });
                }

                for p in scan_existing_files(watch_path, &prefix) {
                    let file_key = p.to_string_lossy().to_string();
                    let mut processed =
                        processed_files.lock().expect("processed_files mutex poisoned");
                    if processed.contains_key(&file_key) {
                        continue;
                    }
                    processed.insert(file_key, SystemTime::now());
                    drop(processed);

                    let pr = processor.pool_clone();
                    let proc = StockProcessor::new(pr);
                    let pref = prefix.clone();
                    let arch = archived_path.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = proc.process_file(line_id, p, pref, arch).await {
                            eprintln!("Error processing polled file: {}", e);
                        }
                    });
                }
                last_scan = Instant::now();
            }

            let recv = match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(v) => v,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(e) => {
                    eprintln!("watch channel error: {:?}", e);
                    break;
                }
            };

            match recv {
                Ok(event) => {
                    if matches!(
                        event.kind,
                        notify::EventKind::Create(_) | notify::EventKind::Modify(_)
                    ) {
                        for path_buf in event.paths {
                            let file_key = path_buf.to_string_lossy().to_string();
                            let mut processed =
                                processed_files.lock().expect("processed_files mutex poisoned");
                            if processed.contains_key(&file_key) {
                                continue;
                            }
                            processed.insert(file_key, SystemTime::now());
                            drop(processed);

                            let p = path_buf.clone();
                            let pr = processor.pool_clone();
                            let proc = StockProcessor::new(pr);
                            let pref = prefix.clone();
                            let arch = archived_path.clone();

                            tauri::async_runtime::spawn(async move {
                                if let Err(e) = proc.process_file(line_id, p, pref, arch).await {
                                    eprintln!("Error processing file: {}", e);
                                }
                            });
                        }
                    }
                }
                Err(e) => {
                    eprintln!("watch error: {:?}", e);
                }
            }
        }
    });
}

pub fn stop_watcher(app_handle: AppHandle, line_id: i64) {
    let state = app_handle.state::<WatcherState>();
    let handle = {
        let mut watchers = state.watchers.lock().expect("watchers mutex poisoned");
        watchers.remove(&line_id)
    };

    if let Some(h) = handle {
        let _ = h.stop_tx.send(());
    }
}
