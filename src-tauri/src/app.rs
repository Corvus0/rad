use crate::downloads::{DownloadItem, DownloadStatus};
use crate::file::download_audio;
use reqwest::Client;
use std::{collections::HashMap, env::current_dir, error::Error, path::PathBuf};
use tauri::{App, Emitter, Manager, Runtime, State, async_runtime::spawn as tauri_spawn};
use tokio::{
    spawn,
    sync::{
        Mutex, RwLock,
        mpsc::{Receiver, Sender},
    },
};

pub struct DownloadState {
    pub downloads: RwLock<HashMap<usize, DownloadItem>>,
    pub queue: Mutex<Sender<usize>>,
    pub url_id: Mutex<HashMap<String, usize>>,
    pub id: Mutex<usize>,
    pub directory: RwLock<PathBuf>,
}

impl DownloadState {
    pub fn new(tx: Sender<usize>) -> Self {
        Self {
            downloads: RwLock::default(),
            queue: Mutex::new(tx),
            url_id: Mutex::default(),
            id: Mutex::default(),
            directory: RwLock::new(current_dir().unwrap_or_default()),
        }
    }
}

pub fn setup_app(app: &App, mut rx: Receiver<usize>) -> Result<(), Box<dyn Error + 'static>> {
    let app_handle = app.handle().clone();
    let client = Client::new();
    tauri_spawn(async move {
        while let Some(id) = rx.recv().await {
            let app_handle = app_handle.clone();
            let client = client.clone();
            spawn(async move {
                let state = app_handle
                    .try_state::<DownloadState>()
                    .ok_or("Could not access application state".to_owned())?;
                emit_update(id, DownloadStatus::Downloading, None, &state, &app_handle).await?;
                let (status, failure) = if let Err(e) = download_audio(id, &client, &state).await {
                    (DownloadStatus::Failed, Some(e))
                } else {
                    (DownloadStatus::Completed, None)
                };
                emit_update(id, status, failure, &state, &app_handle).await?;
                Ok::<(), String>(())
            });
        }
    });
    Ok(())
}

async fn emit_update<R: Runtime>(
    id: usize,
    status: DownloadStatus,
    failure: Option<String>,
    state: &State<'_, DownloadState>,
    manager: &impl Emitter<R>,
) -> Result<(), String> {
    let mut downloads = state.downloads.write().await;
    let download = downloads.get_mut(&id).ok_or("Invalid download id")?;
    download.set_status(status);
    download.set_failure(failure);
    manager
        .emit::<DownloadItem>("update_downloads", download.clone())
        .map_err(|e| e.to_string())
}
