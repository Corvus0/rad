use crate::{app::DownloadState, downloads::DownloadItem};
use ffmpeg_sidecar::{self, command::FfmpegCommand};
use lofty::{Accessor, ItemKey, Tag, TagExt, TaggedFileExt};
use reqwest::{Client, header::HeaderMap};
use std::{collections::HashMap, path::Path};
use tauri::State;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    spawn,
};

pub async fn download_audio(
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
    let filename = format!("{}.{}", download.filename()?, download.extension());
    let mut file_path = Path::new(&directory).join(&filename);
    let mut file = File::create_new(&file_path)
        .await
        .map_err(|e| format!("Failed to create file: {e}"))?;
    let bytes = {
        let chunks = download.chunks();
        if chunks.is_empty() {
            download_file(download.audio(), download.headers(), client).await?
        } else {
            download_chunks(chunks, client).await?
        }
    };
    file.write_all(&bytes)
        .await
        .map_err(|e| format!("Failed to write data to file: {e}"))?;
    if download.extension() == "ts" {
        let new_filename = format!("{}.mp3", download.filename()?);
        let new_file_path = Path::new(&directory).join(&new_filename);
        ts_to_mp3(&file_path, &new_file_path).await?;
        file_path = new_file_path;
    }
    tag_file(&file_path, &download)
}

async fn download_file(
    url: &str,
    headers: &HashMap<String, String>,
    client: &Client,
) -> Result<Vec<u8>, String> {
    let data = client
        .get(url)
        .headers(HeaderMap::try_from(headers).map_err(|e| e.to_string())?)
        .send()
        .await
        .map_err(|e| format!("Failed to download: {e}"))?;
    data.bytes()
        .await
        .map_err(|e| format!("Failed to parse to bytes: {e}"))
        .map(|bytes| bytes.to_vec())
}

async fn download_chunks(chunks: &[String], client: &Client) -> Result<Vec<u8>, String> {
    let handles: Vec<_> = chunks
        .iter()
        .map(|chunk| {
            let client = client.clone();
            let chunk = chunk.clone();
            spawn(async move {
                client
                    .get(&chunk)
                    .send()
                    .await
                    .map_err(|e| format!("Failed to download chunk {chunk}: {e}"))
            })
        })
        .collect();
    let mut data_chunks = Vec::new();
    for (i, handle) in handles.into_iter().enumerate() {
        let data_chunk = handle.await.map_err(|e| e.to_string())??;
        let chunk_bytes = data_chunk
            .bytes()
            .await
            .map_err(|e| format!("Failed to parse chunk to bytes: {e}"))?;
        data_chunks.push((i, chunk_bytes));
    }
    data_chunks.sort_unstable_by_key(|(i, _data)| *i);
    Ok(data_chunks
        .into_iter()
        .map(|(_i, data)| data)
        .collect::<Vec<_>>()
        .concat())
}

async fn ts_to_mp3(input: &Path, output: &Path) -> Result<(), String> {
    ffmpeg_sidecar::download::auto_download().map_err(|e| e.to_string())?;
    FfmpegCommand::new()
        .input(input.to_string_lossy())
        .output(output.to_string_lossy())
        .codec_audio("copy")
        .no_video()
        .no_overwrite()
        .spawn()
        .map_err(|e| e.to_string())?
        .wait()
        .map_err(|e| e.to_string())?
        .success()
        .then_some(())
        .ok_or("Ffmpeg command failed to run")?;
    fs::remove_file(input).await.map_err(|e| e.to_string())
}

fn tag_file(file_path: &Path, download: &DownloadItem) -> Result<(), String> {
    let tagged_file = lofty::read_from_path(file_path)
        .map_err(|e| format!("Failed to read tags from file: {e}"))?;
    let mut tag = Tag::new(tagged_file.primary_tag_type());
    tag.set_artist(download.op().to_owned());
    tag.set_album(download.op().to_owned());
    tag.insert_text(ItemKey::AlbumArtist, download.op().to_owned());
    tag.set_title(download.title().to_owned());
    tag.set_genre(download.sub().to_owned());
    tag.save_to_path(file_path)
        .map_err(|e| format!("Failed to write tags to file: {e}"))
}
