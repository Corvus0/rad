// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod commands;
mod downloads;
use app::*;
use commands::*;
pub use downloads::*;
use tokio::sync::mpsc;

fn main() -> Result<(), tauri::Error> {
    const BUFFER_SIZE: usize = 12;
    let (tx, rx) = mpsc::channel(BUFFER_SIZE);
    tauri::Builder::default()
        .manage::<DownloadState>(DownloadState::new(tx))
        .invoke_handler(tauri::generate_handler![
            get_downloads,
            add_download,
            update_download,
            remove_download,
            clear_downloads,
            remove_completed,
            queue_download,
            queue_downloads,
        ])
        .setup(move |app| setup_app(app, rx))
        .run(tauri::generate_context!())
}
