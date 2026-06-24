// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod commands;
mod downloads;
use app::{DownloadState, setup_app};
use commands::{
    add_download, clear_downloads, get_directory, get_downloads, queue_download, queue_downloads,
    remove_completed, remove_download, set_directory, update_download,
};
pub use downloads::*;
use tokio::sync::mpsc;

fn main() -> Result<(), tauri::Error> {
    let threads: usize = std::thread::available_parallelism()?.get();
    let (tx, rx) = mpsc::channel(threads);
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
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
            get_directory,
            set_directory,
        ])
        .setup(move |app| setup_app(app, rx))
        .run(tauri::generate_context!())
}
