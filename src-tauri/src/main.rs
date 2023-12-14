// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod downloads;
use std::collections::HashMap;

use downloads::*;
use reqwest::Client;
use tauri::{Manager, State};
use tokio::{
    self,
    fs::File,
    io::AsyncWriteExt,
    sync::{mpsc, Mutex},
};

fn main() -> Result<(), tauri::Error> {
    let (input_tx, input_rx) = mpsc::channel(12);
    let (output_tx, mut output_rx) = mpsc::channel(12);
    tauri::Builder::default()
        .manage::<Downloads>(Downloads {
            downloads: Default::default(),
            queue: Mutex::new(input_tx),
            url_id: Mutex::new(HashMap::new()),
            id: Mutex::new(0),
        })
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
        .setup(|app| {
            tauri::async_runtime::spawn(async move { process_queue(input_rx, output_tx).await });
            let app_handle = app.handle();
            let client = Client::new();
            tauri::async_runtime::spawn(async move {
                loop {
                    if let Some(mut download) = output_rx.recv().await {
                        let client = client.clone();
                        let app_handle = app_handle.clone();
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
                    };
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
}

struct Downloads {
    downloads: Mutex<Vec<DownloadItem>>,
    queue: Mutex<mpsc::Sender<DownloadItem>>,
    url_id: Mutex<HashMap<String, usize>>,
    id: Mutex<usize>,
}

#[tauri::command]
async fn get_downloads(state: State<'_, Downloads>) -> Result<Vec<DownloadItem>, String> {
    let downloads = state.downloads.lock().await.to_vec();
    Ok(downloads)
}

#[tauri::command]
async fn add_download(
    download_input: DownloadInput,
    state: State<'_, Downloads>,
) -> Result<Vec<DownloadItem>, String> {
    let mut url_id = state.url_id.lock().await;
    let input_url = &download_input.url().to_owned();
    if url_id.contains_key(input_url) {
        return Err(format!("URL already added: {}", input_url));
    }
    let mut id = state.id.lock().await;
    let download_item = download_input.parse_input(*id).await?;
    let mut downloads = state.downloads.lock().await;
    downloads.push(download_item);
    url_id.insert(input_url.clone(), *id);
    *id += 1;
    Ok(downloads.to_vec())
}

#[tauri::command]
async fn update_download(
    mut download: DownloadItem,
    state: State<'_, Downloads>,
) -> Result<Vec<DownloadItem>, String> {
    let mut url_id = state.url_id.lock().await;
    let input_url = &download.url().to_owned();
    if let Some(id) = url_id.get(input_url) {
        if id != download.id() {
            return Err(format!("URL already added: {}", input_url));
        }
    }
    let mut downloads = state.downloads.lock().await;
    let index = downloads
        .iter()
        .position(|d| d.id() == download.id())
        .ok_or(format!("Invalid id: {}", download.id()))?;
    let old_download = &downloads[index];
    if old_download.url() != *input_url {
        url_id.remove(old_download.url());
        download = download.parse_input().await?;
        url_id.insert(input_url.clone(), *download.id());
    }
    downloads[index] = download;
    Ok(downloads.to_vec())
}

#[tauri::command]
async fn remove_download(
    id: usize,
    state: State<'_, Downloads>,
) -> Result<Vec<DownloadItem>, String> {
    let mut downloads = state.downloads.lock().await;
    let mut url_id = state.url_id.lock().await;
    let index = downloads
        .iter()
        .position(|d| *d.id() == id)
        .ok_or(format!("Invalid id: {}", id))?;
    let download = downloads.remove(index);
    let url = download.url();
    url_id.remove(url);
    Ok(downloads.to_vec())
}

#[tauri::command]
async fn clear_downloads(state: State<'_, Downloads>) -> Result<Vec<DownloadItem>, String> {
    state.downloads.lock().await.clear();
    state.url_id.lock().await.clear();
    Ok(Vec::new())
}

#[tauri::command]
async fn remove_completed(state: State<'_, Downloads>) -> Result<Vec<DownloadItem>, String> {
    let mut downloads = state.downloads.lock().await;
    let mut url_id = state.url_id.lock().await;
    let filtered: Vec<DownloadItem> = downloads
        .to_vec()
        .into_iter()
        .filter(|d| {
            let completed = d.is_completed();
            if completed {
                url_id.remove(d.url());
            }
            !completed
        })
        .collect();
    *downloads = filtered.to_vec();
    Ok(filtered)
}

async fn download_file(download: DownloadItem, client: Client) -> Result<(), String> {
    let handle = tokio::spawn(async move {
        let filename = format!(
            "[{}] [{}] {}.m4a",
            download.sub(),
            download.op(),
            download.title()
        );
        if std::path::Path::new(&filename).exists() {
            return Err("File already exists".to_owned());
        }
        let mut file = match File::create(&filename).await {
            Ok(file) => file,
            Err(e) => return Err(format!("Failed to create file: {}", e.to_string())),
        };
        let data = match client.get(download.audio()).send().await {
            Ok(data) => data,
            Err(e) => return Err(format!("Failed to download: {}", e.to_string())),
        };
        let bytes = match data.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => return Err(format!("Failed to parse to bytes: {}", e.to_string())),
        };
        match file.write_all(&bytes).await {
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to write data to file: {}", e.to_string())),
        };
        Ok(())
    });
    match handle.await {
        Ok(res) => res,
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn queue_download(
    download: DownloadItem,
    state: tauri::State<'_, Downloads>,
) -> Result<(), String> {
    state
        .queue
        .lock()
        .await
        .send(download)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn queue_downloads(state: State<'_, Downloads>) -> Result<(), String> {
    let downloads = state.downloads.lock().await.to_vec();
    let queue = state.queue.lock().await;
    let handles: Vec<_> = downloads
        .into_iter()
        .map(|d| {
            let queue = queue.clone();
            tokio::spawn(async move { queue.send(d).await.map_err(|e| e.to_string()) })
        })
        .collect();
    for handle in handles {
        handle.await.map_err(|e| e.to_string())??;
    }
    Ok(())
}

async fn update_downloads<R: tauri::Runtime>(
    download: DownloadItem,
    manager: &impl Manager<R>,
) -> Result<(), String> {
    if let Some(state) = manager.try_state::<Downloads>() {
        let mut downloads = state.downloads.lock().await;
        let index = downloads
            .iter()
            .position(|d| d.id() == download.id())
            .ok_or(format!("Invalid id: {}", download.id()))?;
        downloads[index] = download.clone();
        return manager
            .emit_all::<DownloadItem>("update_downloads", download)
            .map_err(|e| e.to_string());
    }
    Ok(())
}

async fn process_queue(
    mut input_rx: mpsc::Receiver<DownloadItem>,
    output_tx: mpsc::Sender<DownloadItem>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    while let Some(input) = input_rx.recv().await {
        let output = input;
        output_tx.send(output).await?;
    }
    Ok(())
}
