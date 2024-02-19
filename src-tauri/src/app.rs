use crate::{DownloadItem, DownloadStatus};
use lofty::{Accessor, ItemKey, Tag, TagExt, TaggedFileExt};
use reqwest::{header::HeaderMap, Client};
use std::{
    collections::HashMap,
    env::current_dir,
    error::Error,
    path::{Path, PathBuf},
};
use tauri::{async_runtime::spawn as tauri_spawn, App, Manager, Runtime, State};
use tokio::{
    fs::File,
    io::AsyncWriteExt,
    spawn,
    sync::{
        mpsc::{Receiver, Sender},
        Mutex, RwLock,
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
            downloads: Default::default(),
            queue: Mutex::new(tx),
            url_id: Default::default(),
            id: Default::default(),
            directory: RwLock::new(current_dir().unwrap_or_default()),
        }
    }
}

pub fn setup_app(app: &mut App, mut rx: Receiver<usize>) -> Result<(), Box<dyn Error + 'static>> {
    let app_handle = app.handle();
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
                let (status, failure) = if let Err(e) = download_file(id, &client, &state).await {
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

async fn download_file(
    id: usize,
    client: &Client,
    state: &State<'_, DownloadState>,
) -> Result<(), String> {
    let download = state
        .downloads
        .read()
        .await
        .get(&id)
        .ok_or("Invalid download id")?
        .clone();
    let directory = state.directory.read().await.to_owned();
    let file_path = Path::new(&directory).join(download.filename()?);
    if file_path.exists() {
        return Err("File already exists".to_owned());
    }
    let mut file = File::create(&file_path)
        .await
        .map_err(|e| format!("Failed to create file: {}", e.to_string()))?;
    let data = client
        .get(download.audio())
        .headers(HeaderMap::try_from(download.headers()).map_err(|e| e.to_string())?)
        .send()
        .await
        .map_err(|e| format!("Failed to download: {}", e.to_string()))?;
    let bytes = data
        .bytes()
        .await
        .map_err(|e| format!("Failed to parse to bytes: {}", e.to_string()))?;
    file.write_all(&bytes)
        .await
        .map_err(|e| format!("Failed to write data to file: {}", e.to_string()))?;
    let tagged_file = lofty::read_from_path(&file_path)
        .map_err(|e| format!("Failed to read tags from file: {}", e.to_string()))?;
    let mut tag = Tag::new(tagged_file.primary_tag_type());
    tag.set_artist(download.op().to_owned());
    tag.set_album(download.op().to_owned());
    tag.insert_text(ItemKey::AlbumArtist, download.op().to_owned());
    tag.set_title(download.title().to_owned());
    tag.set_genre(download.sub().to_owned());
    tag.save_to_path(&file_path)
        .map_err(|e| format!("Failed to write tags to file: {}", e.to_string()))
}

async fn emit_update<R: Runtime>(
    id: usize,
    status: DownloadStatus,
    failure: Option<String>,
    state: &State<'_, DownloadState>,
    manager: &impl Manager<R>,
) -> Result<(), String> {
    let mut downloads = state.downloads.write().await;
    let download = downloads.get_mut(&id).ok_or("Invalid download id")?;
    download.set_status(status);
    download.set_failure(failure);
    manager
        .emit_all::<DownloadItem>("update_downloads", download.clone())
        .map_err(|e| e.to_string())
}
