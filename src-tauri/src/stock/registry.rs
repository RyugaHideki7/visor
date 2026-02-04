use std::collections::HashMap;
use std::sync::{mpsc, Mutex};

pub struct WatcherState {
    pub(crate) watchers: Mutex<HashMap<i64, WatcherHandle>>,
}

pub(crate) struct WatcherHandle {
    pub(crate) stop_tx: mpsc::Sender<()>,
}

impl WatcherState {
    pub fn new() -> Self {
        Self {
            watchers: Mutex::new(HashMap::new()),
        }
    }
}
