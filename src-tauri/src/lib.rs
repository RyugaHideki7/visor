mod app;
mod commands;
mod db;
mod logging;
pub mod scheduler;
mod stock;

pub fn run() {
    app::run_app();
}
