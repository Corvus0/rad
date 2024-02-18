use crate::{DownloadItem, DownloadStatus};
use lofty::{Accessor, ItemKey, Tag, TagExt, TaggedFileExt};
use regex::Regex;
use reqwest::{header::HeaderMap, Client};
use std::{collections::HashMap, error::Error};
use tauri::{App, Manager};
use tokio::{
    self,
    fs::File,
    io::AsyncWriteExt,
    sync::{mpsc, Mutex},
};

pub struct DownloadState {
    pub downloads: Mutex<HashMap<usize, DownloadItem>>,
    pub queue: Mutex<mpsc::Sender<DownloadItem>>,
    pub url_id: Mutex<HashMap<String, usize>>,
    pub id: Mutex<usize>,
}

impl DownloadState {
    pub fn new(tx: mpsc::Sender<DownloadItem>) -> Self {
        Self {
            downloads: Default::default(),
            queue: Mutex::new(tx),
            url_id: Default::default(),
            id: Default::default(),
        }
    }
}

pub fn setup_app(
    app: &mut App,
    mut rx: mpsc::Receiver<DownloadItem>,
) -> Result<(), Box<dyn Error + 'static>> {
    let app_handle = app.handle();
    let client = Client::new();
    tauri::async_runtime::spawn(async move {
        while let Some(mut download) = rx.recv().await {
            let app_handle = app_handle.clone();
            let client = client.clone();
            tokio::spawn(async move {
                download.set_status(DownloadStatus::Downloading);
                update_downloads(download.clone(), &app_handle).await?;
                if let Err(e) = download_file(download.clone(), client).await {
                    download.set_status(DownloadStatus::Failed);
                    download.set_failure(e);
                } else {
                    download.set_status(DownloadStatus::Completed);
                }
                update_downloads(download, &app_handle).await?;
                Ok::<(), String>(())
            });
        }
    });
    Ok(())
}

async fn download_file(download: DownloadItem, client: Client) -> Result<(), String> {
    let extension = download
        .audio()
        .split(".")
        .last()
        .ok_or(format!("Audio URL contains no valid file extension"))?;
    let filename = format!(
        "[{}] [{}] {}.{}",
        download.sub(),
        download.op(),
        download.title(),
        extension,
    );
    let invalid_chars = Regex::new(r#"[<>:"/\\\?\*|]+"#)
        .map_err(|e| format!("Invalid regex pattern: {}", e.to_string()))?;
    let filename = invalid_chars.replace_all(&filename, "").to_string();
    if std::path::Path::new(&filename).exists() {
        return Err("File already exists".to_owned());
    }
    let mut file = File::create(&filename)
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
    let tagged_file = lofty::read_from_path(&filename)
        .map_err(|e| format!("Failed to read tags from file: {}", e.to_string()))?;
    let mut tag = Tag::new(tagged_file.primary_tag_type());
    tag.set_artist(download.op().to_owned());
    tag.set_album(download.op().to_owned());
    tag.insert_text(ItemKey::AlbumArtist, download.op().to_owned());
    tag.set_title(download.title().to_owned());
    tag.set_genre(download.sub().to_owned());
    tag.save_to_path(&filename)
        .map_err(|e| format!("Failed to write tags to file: {}", e.to_string()))
}

async fn update_downloads<R: tauri::Runtime>(
    download: DownloadItem,
    manager: &impl Manager<R>,
) -> Result<(), String> {
    if let Some(state) = manager.try_state::<DownloadState>() {
        let mut downloads = state.downloads.lock().await;
        downloads
            .insert(*download.id(), download.clone())
            .ok_or(format!("Invalid id: {}", download.id()))?;
        return manager
            .emit_all::<DownloadItem>("update_downloads", download)
            .map_err(|e| e.to_string());
    }
    Ok(())
}
