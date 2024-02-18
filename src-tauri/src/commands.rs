use crate::{DownloadInput, DownloadItem, DownloadState};
use tauri::State;
use tokio::spawn;

#[tauri::command]
pub async fn get_downloads(state: State<'_, DownloadState>) -> Result<Vec<DownloadItem>, String> {
    let mut downloads_vec: Vec<DownloadItem> =
        state.downloads.lock().await.values().cloned().collect();
    downloads_vec.sort_unstable_by(|a, b| a.id().cmp(b.id()));
    Ok(downloads_vec)
}

#[tauri::command]
pub async fn add_download(
    download_input: DownloadInput,
    state: State<'_, DownloadState>,
) -> Result<DownloadItem, String> {
    let mut url_id = state.url_id.lock().await;
    let input_url = download_input.url().to_owned();
    if url_id.contains_key(&input_url) {
        return Err(format!("URL already added: {}", &input_url));
    }
    let mut id = state.id.lock().await;
    let download_item = download_input.parse_input(*id).await?;
    let mut downloads = state.downloads.lock().await;
    downloads.insert(*id, download_item.clone());
    url_id.insert(input_url, *id);
    *id += 1;
    Ok(download_item)
}

#[tauri::command]
pub async fn update_download(
    mut download: DownloadItem,
    state: State<'_, DownloadState>,
) -> Result<DownloadItem, String> {
    let mut url_id = state.url_id.lock().await;
    let input_url = download.url().to_owned();
    if let Some(id) = url_id.get(&input_url) {
        if id != download.id() {
            return Err(format!("URL already added: {}", input_url));
        }
    }
    let mut downloads = state.downloads.lock().await;
    let old_download = downloads
        .get(download.id())
        .ok_or(format!("Invalid id: {}", download.id()))?;
    if old_download.url() != input_url {
        url_id.remove(old_download.url());
        download = download.parse_input().await?;
        url_id.insert(input_url, *download.id());
    }
    downloads.insert(*download.id(), download.clone());
    Ok(download)
}

#[tauri::command]
pub async fn remove_download(id: usize, state: State<'_, DownloadState>) -> Result<(), String> {
    let mut downloads = state.downloads.lock().await;
    let mut url_id = state.url_id.lock().await;
    let download = downloads.remove(&id).ok_or(format!("Invalid id: {}", id))?;
    let url = download.url();
    url_id.remove(url);
    Ok(())
}

#[tauri::command]
pub async fn clear_downloads(state: State<'_, DownloadState>) -> Result<(), String> {
    state.downloads.lock().await.clear();
    state.url_id.lock().await.clear();
    Ok(())
}

#[tauri::command]
pub async fn remove_completed(state: State<'_, DownloadState>) -> Result<(), String> {
    let mut downloads = state.downloads.lock().await;
    let mut url_id = state.url_id.lock().await;
    downloads.retain(|_, d| !d.is_completed());
    url_id.retain(|_, id| downloads.contains_key(id));
    Ok(())
}

#[tauri::command]
pub async fn queue_download(
    download: DownloadItem,
    state: tauri::State<'_, DownloadState>,
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
pub async fn queue_downloads(state: State<'_, DownloadState>) -> Result<(), String> {
    let queue = state.queue.lock().await;
    let handles: Vec<_> = state
        .downloads
        .lock()
        .await
        .values()
        .cloned()
        .filter(|d| !d.is_completed())
        .map(|d| {
            let queue = queue.clone();
            spawn(async move { queue.send(d).await.map_err(|e| e.to_string()) })
        })
        .collect();
    for handle in handles {
        handle.await.map_err(|e| e.to_string())??;
    }
    Ok(())
}
